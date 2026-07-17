//! CI/CD Pipeline API endpoints.
//!
//! Route handlers that depend on AppState. Types and DB logic are in civit-ci.

#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, OptionalAuthUser};
use crate::error::CoreError;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use civit_ci::pipeline::{self, PipelineListParams, PipelineRunResponse, TriggerPipelineRequest};
use civit_pipeline::trigger::TriggerContext;
use civit_pipeline::{
    expand_matrix, matches_trigger, parse_pipeline, resolve_includes, validate_pipeline,
};
use uuid::Uuid;

pub fn pipeline_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{repo}/pipelines",
            get(list_pipelines).post(trigger_pipeline),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/pipelines/{pipeline_id}",
            get(get_pipeline).delete(cancel_pipeline),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/pipelines/{pipeline_id}/jobs",
            get(get_pipeline_jobs),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/pipelines/{pipeline_id}/graph",
            get(get_pipeline_graph),
        )
        .route("/api/v1/pipelines", get(list_all_pipelines))
}

async fn get_repo_id(
    pool: &sqlx::PgPool,
    owner: &str,
    repo_name: &str,
) -> std::result::Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_as::<_, (Uuid,)>(
        "SELECT r.id FROM repositories r JOIN users u ON r.owner_id = u.id WHERE u.username = $1 AND r.name = $2",
    )
    .bind(owner)
    .bind(repo_name)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}

async fn read_pipeline_yaml(
    state: &AppState,
    _repo_id: &Uuid,
    owner: &str,
    repo_name: &str,
    ref_name: &str,
    yaml_path: &str,
) -> std::result::Result<Option<String>, CoreError> {
    let base = &state.config.storage_path;
    let repo_path = std::path::Path::new(base)
        .join(owner)
        .join(format!("{repo_name}.git"));

    let output = tokio::process::Command::new("git")
        .arg("archive")
        .arg("--format=tar")
        .arg(ref_name)
        .arg(yaml_path)
        .current_dir(&repo_path)
        .output()
        .await;

    if let Ok(out) = output
        && out.status.success()
        && !out.stdout.is_empty()
    {
        let cursor = std::io::Cursor::new(&out.stdout);
        if let Ok(mut archive) = tar::Archive::new(cursor).entries() {
            while let Some(Ok(mut entry)) = archive.next() {
                let mut content = String::new();
                if std::io::Read::read_to_string(&mut entry, &mut content).is_ok() {
                    return Ok(Some(content));
                }
            }
        }
    }

    let file_path = repo_path.join(yaml_path);
    if file_path.exists() {
        tokio::fs::read_to_string(&file_path)
            .await
            .map(Some)
            .map_err(|e| CoreError::Internal(format!("read pipeline YAML: {e}")))
    } else {
        Ok(None)
    }
}

pub async fn list_pipelines(
    State(state): State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    Query(params): Query<PipelineListParams>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    match get_repo_id(pool, &owner, &repo_name).await {
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response(),
        Ok(Some(repo_id)) => {
            match pipeline::list_pipelines_by_repo(
                pool,
                repo_id,
                params.limit,
                params.offset,
                params.status.as_deref(),
            )
            .await
            {
                Ok(runs) => {
                    let resp: Vec<PipelineRunResponse> =
                        runs.into_iter().map(|r| r.into()).collect();
                    (axum::http::StatusCode::OK, Json(resp)).into_response()
                }
                Err(e) => (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Database(e.to_string()).error_response()),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_all_pipelines(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    match pipeline::list_all_pipelines_db(pool, 50, 0).await {
        Ok(runs) => {
            let resp: Vec<PipelineRunResponse> = runs.into_iter().map(|r| r.into()).collect();
            (axum::http::StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn trigger_pipeline(
    State(state): State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<TriggerPipelineRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();

    let repo_id = match get_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let yaml_content = match read_pipeline_yaml(
        &state,
        &repo_id,
        &owner,
        &repo_name,
        &req.ref_name,
        &req.yaml_path,
    )
    .await
    {
        Ok(Some(content)) => content,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(
                    CoreError::NotFound(format!(
                        "pipeline YAML not found at '{}' in ref '{}'",
                        req.yaml_path, req.ref_name
                    ))
                    .error_response(),
                ),
            )
                .into_response();
        }
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let mut pl = match parse_pipeline(&yaml_content) {
        Ok(p) => p,
        Err(e) => {
            return (
                axum::http::StatusCode::UNPROCESSABLE_ENTITY,
                Json(CoreError::NotFound(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    // Resolve includes for v2 pipelines
    if pl.version == "2"
        && let Some(includes) = &pl.include {
            let mut included_files = Vec::new();
            for inc in includes {
                let ref_name = inc.ref_name.as_deref().unwrap_or(&req.ref_name);
                match read_pipeline_yaml(&state, &repo_id, &owner, &repo_name, ref_name, &inc.source).await {
                    Ok(Some(content)) => {
                        included_files.push((inc.clone(), content));
                    }
                    Ok(None) => {
                        return (
                            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
                            Json(
                                CoreError::NotFound(format!(
                                    "included file '{}' not found",
                                    inc.source
                                ))
                                .error_response(),
                            ),
                        )
                            .into_response();
                    }
                    Err(e) => {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            Json(CoreError::Database(e.to_string()).error_response()),
                        )
                            .into_response();
                    }
                }
            }

            if let Err(e) = resolve_includes(&mut pl, &included_files) {
                return (
                    axum::http::StatusCode::UNPROCESSABLE_ENTITY,
                    Json(CoreError::NotFound(e.to_string()).error_response()),
                )
                    .into_response();
            }
        }

    if let Err(e) = validate_pipeline(&pl) {
        return (
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            Json(CoreError::NotFound(e.to_string()).error_response()),
        )
            .into_response();
    }

    let pl = match expand_matrix(&pl) {
        Ok(p) => p,
        Err(e) => {
            return (
                axum::http::StatusCode::UNPROCESSABLE_ENTITY,
                Json(CoreError::NotFound(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let event_type = req.event_type.as_deref().unwrap_or("push");
    let ctx = TriggerContext::push(&req.ref_name, req.changed_files.clone());
    let _ = ctx;

    match pipeline::create_pipeline_run(
        pool,
        repo_id,
        &req.yaml_path,
        &req.ref_name,
        &req.commit_sha,
        event_type,
        &yaml_content,
        &pl,
    )
    .await
    {
        Ok(run) => {
            let dispatcher = crate::webhooks::WebhookDispatcher::new();
            let pool_clone = state.db.pool().clone();
            let rid = repo_id;
            let evt = crate::webhooks::WebhookEvent::Pipeline;
            let payload = serde_json::json!({
                "action": "started",
                "pipeline_id": run.id,
                "repo_id": rid.to_string(),
                "trigger": run.trigger,
                "ref_name": run.ref_name,
                "commit_sha": run.commit_sha,
                "status": run.status,
            });
            tokio::spawn(async move { dispatcher.dispatch(&pool_clone, rid, &evt, payload).await });

            (axum::http::StatusCode::CREATED, Json(run)).into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_pipeline(
    State(state): State<AppState>,
    Path((_owner, _repo_name, pipeline_id)): Path<(String, String, String)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let id = match Uuid::parse_str(&pipeline_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid pipeline ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match pipeline::get_pipeline_detail(pool, id).await {
        Ok(Some(detail)) => (axum::http::StatusCode::OK, Json(detail)).into_response(),
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("pipeline run not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn cancel_pipeline(
    State(state): State<AppState>,
    Path((owner, repo_name, pipeline_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let id = match Uuid::parse_str(&pipeline_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid pipeline ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let repo_id_opt = get_repo_id(pool, &owner, &repo_name).await.ok().flatten();

    match pipeline::cancel_pipeline_run(pool, id).await {
        Ok(Some(run)) => {
            if let Some(rid) = repo_id_opt {
                let dispatcher = crate::webhooks::WebhookDispatcher::new();
                let pool_clone = state.db.pool().clone();
                let evt = crate::webhooks::WebhookEvent::Pipeline;
                let payload = serde_json::json!({
                    "action": "canceled",
                    "pipeline_id": run.id,
                    "repo_id": rid.to_string(),
                    "status": run.status,
                });
                tokio::spawn(
                    async move { dispatcher.dispatch(&pool_clone, rid, &evt, payload).await },
                );
            }
            (axum::http::StatusCode::OK, Json(run)).into_response()
        }
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("pipeline run not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_pipeline_jobs(
    State(state): State<AppState>,
    Path((_owner, _repo_name, pipeline_id)): Path<(String, String, String)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let id = match Uuid::parse_str(&pipeline_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid pipeline ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match pipeline::get_pipeline_jobs_db(pool, id).await {
        Ok(jobs) => (axum::http::StatusCode::OK, Json(jobs)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_pipeline_graph(
    State(state): State<AppState>,
    Path((_owner, _repo_name, pipeline_id)): Path<(String, String, String)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let id = match Uuid::parse_str(&pipeline_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid pipeline ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match pipeline::get_pipeline_graph_db(pool, id).await {
        Ok(graph) => (axum::http::StatusCode::OK, Json(graph)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn trigger_pipelines_on_push(state: &AppState, owner: &str, repo_name: &str) {
    let pool = state.db.pool();

    let repo_id = match get_repo_id(pool, owner, repo_name).await {
        Ok(Some(id)) => id,
        _ => return,
    };

    let (ref_name, commit_sha) = match detect_push_ref(state, owner, repo_name).await {
        Some(r) => r,
        None => return,
    };

    let yaml_path = ".civit/pipeline.yaml";
    let yaml_content =
        match read_pipeline_yaml(state, &repo_id, owner, repo_name, &ref_name, yaml_path).await {
            Ok(Some(content)) => content,
            _ => return,
        };

    let mut pl = match parse_pipeline(&yaml_content) {
        Ok(p) => p,
        _ => return,
    };

    // Resolve includes for v2 pipelines
    if pl.version == "2"
        && let Some(includes) = &pl.include {
            let mut included_files = Vec::new();
            for inc in includes {
                let ref_name = inc.ref_name.as_deref().unwrap_or(&ref_name);
                match read_pipeline_yaml(state, &repo_id, owner, repo_name, ref_name, &inc.source)
                    .await
                {
                    Ok(Some(content)) => {
                        included_files.push((inc.clone(), content));
                    }
                    _ => continue,
                }
            }

            if resolve_includes(&mut pl, &included_files).is_err() {
                return;
            }
        }

    if validate_pipeline(&pl).is_err() {
        return;
    }

    let pl = match expand_matrix(&pl) {
        Ok(p) => p,
        _ => return,
    };

    let ctx = TriggerContext::push(&ref_name, vec![]);
    if !matches_trigger(&pl, &ctx) {
        return;
    }

    let _ = pipeline::create_pipeline_run(
        pool,
        repo_id,
        yaml_path,
        &ref_name,
        &commit_sha,
        "push",
        &yaml_content,
        &pl,
    )
    .await;
}

async fn detect_push_ref(
    state: &AppState,
    owner: &str,
    repo_name: &str,
) -> Option<(String, String)> {
    let base = &state.config.storage_path;
    let repo_path = std::path::Path::new(base)
        .join(owner)
        .join(format!("{repo_name}.git"));

    let head_ref = std::process::Command::new("git")
        .arg("--git-dir")
        .arg(&repo_path)
        .arg("symbolic-ref")
        .arg("--short")
        .arg("HEAD")
        .output()
        .ok()?;

    let ref_name = String::from_utf8_lossy(&head_ref.stdout).trim().to_string();
    if ref_name.is_empty() {
        return None;
    }

    let sha_output = std::process::Command::new("git")
        .arg("--git-dir")
        .arg(&repo_path)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .ok()?;

    let commit_sha = String::from_utf8_lossy(&sha_output.stdout)
        .trim()
        .to_string();
    if commit_sha.is_empty() {
        return None;
    }

    Some((ref_name, commit_sha))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_pipeline_routes_compiled() {
        let _ = super::pipeline_routes();
    }
}
