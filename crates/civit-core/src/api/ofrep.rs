#![forbid(unsafe_code)]

//! OpenFeature Remote Evaluation Protocol (OFREP) endpoints.
//!
//! ADR-0008 track 2. OFREP is the vendor-neutral protocol from the
//! OpenFeature project; serving it means any OpenFeature SDK can evaluate
//! CivitForge flags without a CivitForge-specific provider.
//!
//! Two shapes, per protocol version 0.4.0:
//!
//! - `POST /ofrep/v1/evaluate/flags/{key}` — one flag, context per call.
//!   Server-side evaluation, where the context changes between requests.
//! - `POST /ofrep/v1/evaluate/flags` — every flag at once, with an ETag the
//!   client revalidates. Client-side evaluation, where the whole flag set is
//!   fetched once and cached locally.
//!
//! Wire types come from `flag_kit::ofrep` so the shapes cannot drift from
//! the spec enums.

use axum::{
    Json, Router,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use std::collections::HashMap;
use uuid::Uuid;

use crate::api::AppState;
use crate::api::auth::OptionalAuthUser;

/// Reasons are derived from *why* the value is what it is, because a
/// consumer's debugging starts there. Reporting `DISABLED` for a subject
/// outside a 5% rollout sends the reader looking in the wrong place.
fn evaluate_row(
    row: &civit_db::models::FeatureFlag,
    user_id: Option<Uuid>,
    org_id: Option<Uuid>,
) -> flag_kit::ofrep::EvaluationSuccess {
    use flag_kit::ofrep::{EvaluationSuccess, Reason};
    use flag_kit::lifecycle::rollout_decision;

    let name = row.name.as_str();
    if !row.enabled {
        return EvaluationSuccess::off(name, Reason::Disabled);
    }

    // Explicit targeting wins over rollout: an allow-listed subject is in
    // regardless of the bucket, which is the whole point of an allow list.
    let explicitly_targeted = user_id.is_some_and(|u| row.enabled_for_users.contains(&u))
        || org_id.is_some_and(|o| row.enabled_for_orgs.contains(&o));
    if explicitly_targeted {
        return EvaluationSuccess::on(name, Reason::TargetingMatch);
    }

    // No targeting context at all: a boolean flag with a full rollout is
    // static, anything else is unknown rather than guessed.
    let Some(user) = user_id else {
        return if row.enabled_for_percentage >= 100 {
            EvaluationSuccess::on(name, Reason::Static)
        } else {
            EvaluationSuccess::off(name, Reason::Unknown)
        };
    };

    let decision = rollout_decision(
        name,
        &user.to_string(),
        row.enabled_for_percentage.clamp(0, 100) as u8,
        &row.salt,
    );
    if decision.enabled {
        EvaluationSuccess::on(name, Reason::Split)
    } else {
        EvaluationSuccess::off(name, Reason::Split)
    }
}

fn parse_uuid(raw: &str) -> Option<Uuid> {
    Uuid::parse_str(raw).ok()
}

/// `POST /ofrep/v1/evaluate/flags/{key}`
pub async fn evaluate_flag(
    State(state): State<AppState>,
    OptionalAuthUser(auth): OptionalAuthUser,
    Path(key): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    Json(req): Json<flag_kit::ofrep::EvaluationRequest>,
) -> Response {
    let flags = match state.db.list_feature_flags().await {
        Ok(f) => f,
        Err(e) => {
            return ofrep_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                flag_kit::ofrep::ErrorCode::General,
                &format!("could not read flags: {e}"),
            );
        }
    };

    let Some(row) = flags.iter().find(|f| f.name == key) else {
        // 404 with FLAG_NOT_FOUND, not an evaluation failure: providers are
        // expected to fall back to the code default for a missing flag, and
        // only do so when the two are distinguishable.
        return (
            StatusCode::NOT_FOUND,
            Json(flag_kit::ofrep::FlagNotFound::new(&key)),
        )
            .into_response();
    };

    let user_id = resolve_subject(&req, &params, auth.as_ref(), &headers);
    let org_id = req
        .context
        .attributes
        .get("orgId")
        .or_else(|| req.context.attributes.get("org_id"))
        .and_then(|v| v.as_str())
        .and_then(parse_uuid);

    let result = evaluate_row(row, user_id, org_id);
    record_evaluation(&state, std::slice::from_ref(&row.name)).await;
    // Json, not a pre-serialized String: axum renders a bare String as
    // text/plain, and a conformant provider checks the response MIME type
    // before parsing. Found by the real OFREP provider.
    (StatusCode::OK, Json(result)).into_response()
}

/// `POST /ofrep/v1/evaluate/flags` — bulk, with ETag revalidation.
pub async fn evaluate_flags_bulk(
    State(state): State<AppState>,
    OptionalAuthUser(auth): OptionalAuthUser,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    Json(req): Json<flag_kit::ofrep::EvaluationRequest>,
) -> Response {
    let flags = match state.db.list_feature_flags().await {
        Ok(f) => f,
        Err(e) => {
            return ofrep_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                flag_kit::ofrep::ErrorCode::General,
                &format!("could not read flags: {e}"),
            );
        }
    };

    // The ETag identifies the flag *set*, so it must change when any flag's
    // value or targeting changes. Names alone would miss a rollout change.
    let etag = flag_set_etag(&flags);
    if let Some(inm) = headers.get(header::IF_NONE_MATCH).and_then(|v| v.to_str().ok()) {
        if flag_kit::ofrep::if_none_match_hits(inm, &etag) {
            return (
                StatusCode::NOT_MODIFIED,
                [(header::ETAG, HeaderValue::from_str(&etag).unwrap_or(HeaderValue::from_static("W/\"0\"")))],
                Body::empty(),
            )
                .into_response();
        }
    }

    let user_id = resolve_subject(&req, &params, auth.as_ref(), &headers);
    let org_id = req
        .context
        .attributes
        .get("orgId")
        .or_else(|| req.context.attributes.get("org_id"))
        .and_then(|v| v.as_str())
        .and_then(parse_uuid);

    let entries: Vec<flag_kit::ofrep::BulkEntry> = flags
        .iter()
        .map(|row| {
            flag_kit::ofrep::BulkEntry::Success(Box::new(evaluate_row(row, user_id, org_id)))
        })
        .collect();

    let body = flag_kit::ofrep::BulkEvaluationSuccess::new(entries);
    record_evaluation(&state, &flags.iter().map(|f| f.name.clone()).collect::<Vec<_>>()).await;

    (
        StatusCode::OK,
        [(header::ETAG, HeaderValue::from_str(&etag).unwrap_or(HeaderValue::from_static("W/\"0\"")))],
        Json(body),
    )
        .into_response()
}

/// Resolves the evaluation subject.
///
/// OFREP puts the subject in the context; CivitForge's own auth carries it on
/// the token. Authenticated context wins, because a caller must not be able
/// to evaluate another user's flags by putting their id in the body.
fn resolve_subject(
    req: &flag_kit::ofrep::EvaluationRequest,
    params: &HashMap<String, String>,
    auth: Option<&crate::api::auth::AuthUser>,
    _headers: &HeaderMap,
) -> Option<Uuid> {
    if let Some(a) = auth {
        return parse_uuid(&a.user_id);
    }
    req.context
        .targeting_key()
        .and_then(parse_uuid)
        .or_else(|| params.get("targetingKey").and_then(|v| parse_uuid(v)))
}

async fn record_evaluation(state: &AppState, names: &[String]) {
    if let Err(e) = state.db.touch_feature_flags_evaluated(names).await {
        tracing::warn!("could not record OFREP evaluation: {e}");
    }
}

/// Weak ETag over the flag set's observable state.
fn flag_set_etag(flags: &[civit_db::models::FeatureFlag]) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(flags.len());
    for f in flags {
        parts.push(format!(
            "{}:{}:{}:{}:{}:{}",
            f.name,
            u8::from(f.enabled),
            f.enabled_for_percentage,
            f.enabled_for_users.len(),
            f.enabled_for_orgs.len(),
            f.updated_at.timestamp_millis()
        ));
    }
    let refs: Vec<&str> = parts.iter().map(String::as_str).collect();
    flag_kit::ofrep::weak_etag(&refs)
}

fn ofrep_error(status: StatusCode, code: flag_kit::ofrep::ErrorCode, detail: &str) -> Response {
    let body = serde_json::json!({
        "errorCode": code.as_str(),
        "errorDetails": detail,
    });
    (status, Json(body)).into_response()
}

/// OFREP routes.
///
/// Auth is optional on purpose: OFREP clients are provider libraries that
/// send `Authorization` when configured, and a self-hosted deployment may
/// front the endpoint with its own gateway. When no credential is present
/// the context's `targetingKey` identifies the subject, and rollout
/// decisions still work; only allow-list targeting is unavailable.
pub fn ofrep_routes() -> Router<AppState> {
    Router::new()
        .route("/ofrep/v1/evaluate/flags", post(evaluate_flags_bulk))
        .route("/ofrep/v1/evaluate/flags/{key}", post(evaluate_flag))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    fn row(name: &str, enabled: bool, pct: i32) -> civit_db::models::FeatureFlag {
        let now = chrono::Utc::now();
        civit_db::models::FeatureFlag {
            id: Uuid::nil(),
            name: name.into(),
            description: "d".into(),
            enabled,
            enabled_for_users: Vec::new(),
            enabled_for_percentage: pct,
            enabled_for_orgs: Vec::new(),
            created_at: now,
            updated_at: now,
            kind: "release".into(),
            owner: "team".into(),
            ticket: String::new(),
            salt: String::new(),
            last_evaluated_at: Some(now),
            last_changed_at: now,
        }
    }

    #[test]
    fn disabled_flag_is_off_with_the_disabled_reason() {
        let r = evaluate_row(&row("f", false, 100), None, None);
        assert_eq!(r.value.as_bool(), Some(false));
        assert_eq!(r.reason.as_str(), "DISABLED");
    }

    #[test]
    fn full_rollout_without_context_is_static_on() {
        let r = evaluate_row(&row("f", true, 100), None, None);
        assert_eq!(r.value.as_bool(), Some(true));
        assert_eq!(r.reason.as_str(), "STATIC");
    }

    /// Without a subject a partial rollout cannot be decided; reporting
    /// `false` with a rollout reason would be a lie about why.
    #[test]
    fn partial_rollout_without_context_is_unknown() {
        let r = evaluate_row(&row("f", true, 5), None, None);
        assert_eq!(r.reason.as_str(), "UNKNOWN");
        assert_eq!(r.value.as_bool(), Some(false));
    }

    #[test]
    fn percentage_rollout_reports_split_in_both_directions() {
        let mut on = 0;
        let mut off = 0;
        // Distinct subjects: a rollout partitions *people*, so re-evaluating
        // one user 200 times proves nothing about the split.
        for _ in 0..400 {
            let r = evaluate_row(&row("f", true, 50), Some(Uuid::new_v4()), None);
            assert_eq!(r.reason.as_str(), "SPLIT");
            match r.value.as_bool() {
                Some(true) => on += 1,
                _ => off += 1,
            }
        }
        // Bucketing is 0..100 so a 50% rollout lands near half; a wide band
        // keeps this from flaking on hash distribution.
        assert!((120..=280).contains(&on), "expected ~half in rollout, got {on}");
        assert!(
            (120..=280).contains(&off),
            "expected ~half out of rollout, got {off}"
        );
    }

    /// Within a cycle the same subject must keep the same answer, or users
    /// flicker between variants across requests.
    #[test]
    fn a_cycle_is_sticky_for_a_subject() {
        let user = Uuid::new_v4();
        let first = evaluate_row(&row("f", true, 50), Some(user), None).value;
        for _ in 0..20 {
            assert_eq!(
                evaluate_row(&row("f", true, 50), Some(user), None).value,
                first,
                "a rollout must be sticky within its cycle"
            );
        }
    }

    /// A new cycle re-draws the cohort without a deploy — the reason salted
    /// bucketing exists at all.
    #[test]
    fn a_new_cycle_can_move_a_subject() {
        let before = row("f", true, 50);
        let mut after = before.clone();
        after.salt = "cycle-2".into();
        // Across many subjects, not repeatedly for one: re-comparing the same
        // two salts proves nothing, and a single subject is a coin flip.
        let moved = (0..50)
            .filter(|_| {
                let u = Uuid::new_v4();
                evaluate_row(&before, Some(u), None).value != evaluate_row(&after, Some(u), None).value
            })
            .count();
        assert!(moved > 0, "changing the salt must be able to re-draw a cohort");
    }

    #[test]
    fn allow_list_beats_the_rollout_bucket() {
        let user = Uuid::new_v4();
        let mut f = row("f", true, 1);
        f.enabled_for_users = vec![user];
        let r = evaluate_row(&f, Some(user), None);
        assert_eq!(r.value.as_bool(), Some(true));
        assert_eq!(
            r.reason.as_str(),
            "TARGETING_MATCH",
            "an allow-listed subject is in regardless of bucket"
        );
    }

    #[test]
    fn org_allow_list_also_targets() {
        let org = Uuid::new_v4();
        let mut f = row("f", true, 0);
        f.enabled_for_orgs = vec![org];
        let r = evaluate_row(&f, Some(Uuid::new_v4()), Some(org));
        assert_eq!(r.value.as_bool(), Some(true));
        assert_eq!(r.reason.as_str(), "TARGETING_MATCH");
    }

    #[test]
    fn out_of_range_percentage_is_clamped_not_wrapped() {
        let user = Uuid::new_v4();
        let r = evaluate_row(&row("f", true, 250), Some(user), None);
        assert_eq!(r.value.as_bool(), Some(true), "250 means fully rolled out");
        let r = evaluate_row(&row("f", true, -5), Some(user), None);
        assert_eq!(r.value.as_bool(), Some(false));
    }

    /// The ETag must track rollout changes, not just names, or clients will
    /// cache a stale flag set forever.
    #[test]
    fn etag_changes_when_a_rollout_changes() {
        let a = row("f", true, 5);
        let mut b = row("f", true, 50);
        assert_ne!(flag_set_etag(&[a.clone()]), flag_set_etag(&[b.clone()]));
        b.enabled = false;
        assert_ne!(flag_set_etag(&[a]), flag_set_etag(&[b]));
    }

    #[test]
    fn etag_is_stable_for_an_unchanged_set() {
        let flags = vec![row("a", true, 10), row("b", false, 0)];
        assert_eq!(flag_set_etag(&flags), flag_set_etag(&flags));
    }

    #[test]
    fn etag_changes_when_a_flag_is_added() {
        let one = vec![row("a", true, 10)];
        let two = vec![row("a", true, 10), row("b", false, 0)];
        assert_ne!(flag_set_etag(&one), flag_set_etag(&two));
    }

    #[test]
    fn authenticated_context_wins_over_the_body() {
        let auth = crate::api::auth::AuthUser {
            user_id: Uuid::new_v4().to_string(),
            username: "u".into(),
            role: crate::auth::rbac::Role::Member,
            org_id: None,
        };
        let req = flag_kit::ofrep::EvaluationRequest {
            context: flag_kit::ofrep::Context::new(Uuid::new_v4().to_string()),
        };
        let resolved = resolve_subject(&req, &HashMap::new(), Some(&auth), &HeaderMap::new());
        assert_eq!(resolved, parse_uuid(&auth.user_id));
    }

    #[test]
    fn unauthenticated_falls_back_to_targeting_key() {
        let id = Uuid::new_v4();
        let req = flag_kit::ofrep::EvaluationRequest {
            context: flag_kit::ofrep::Context::new(id.to_string()),
        };
        assert_eq!(
            resolve_subject(&req, &HashMap::new(), None, &HeaderMap::new()),
            Some(id)
        );
    }
}