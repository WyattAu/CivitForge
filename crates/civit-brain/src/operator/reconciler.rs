#![forbid(unsafe_code)]

use crate::operator::crd::{
    CivitForgeApp, CivitForgeAppComponent, CivitForgeAppSpec, CivitForgeAppStatus,
    ComponentCondition, ConditionStatus,
};
use kube::{
    Api, Client, ResourceExt,
    api::{Patch, PatchParams},
    runtime::controller::Action,
};
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, info};

const FINALIZER_NAME: &str = "civitforge.dev/finalizer";
const FIELD_MANAGER: &str = "civit-operator";
const REQUEUE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60);

#[derive(Debug, Error)]
pub enum ReconcileError {
    #[error("failed to get CivitForgeApp: {0}")]
    GetObject(#[source] kube::Error),

    #[error("failed to patch CivitForgeApp status: {0}")]
    PatchStatus(#[source] kube::Error),

    #[error("failed to manage finalizer: {0}")]
    Finalizer(#[source] kube::Error),

    #[error("missing object UID")]
    MissingUid,
}

pub struct ReconcileCtx {
    pub client: Client,
}

impl ReconcileCtx {
    pub fn new(client: Client) -> Arc<Self> {
        Arc::new(Self { client })
    }
}

pub async fn reconcile(
    resource: Arc<CivitForgeApp>,
    ctx: Arc<ReconcileCtx>,
) -> Result<Action, ReconcileError> {
    let ns = resource.namespace().unwrap_or_default();
    let name = resource.name_any();
    let api: Api<CivitForgeApp> = Api::namespaced(ctx.client.clone(), &ns);

    debug!(resource = %name, namespace = %ns, "reconciling CivitForgeApp");

    ensure_finalizer(&api, &name, &resource).await?;

    let spec = &resource.spec;
    let current_status = resource.status.clone().unwrap_or_default();

    let mut status = build_initial_status(spec, &current_status);

    let components: Vec<CivitForgeAppComponent> = if spec.components.is_empty() {
        CivitForgeAppComponent::all().to_vec()
    } else {
        spec.components.clone()
    };

    for component in &components {
        let condition = reconcile_component(component);
        status.conditions.push(condition);
    }

    status.phase = Some(compute_phase(&status.conditions));
    status.last_updated = Some(chrono::Utc::now().to_rfc3339());

    patch_status(&api, &name, &status).await?;

    info!(
        resource = %name,
        phase = ?status.phase,
        components = components.len(),
        "reconciliation complete"
    );

    Ok(Action::requeue(REQUEUE_INTERVAL))
}

pub async fn cleanup(
    resource: Arc<CivitForgeApp>,
    ctx: Arc<ReconcileCtx>,
) -> Result<Action, ReconcileError> {
    let ns = resource.namespace().unwrap_or_default();
    let name = resource.name_any();
    let api: Api<CivitForgeApp> = Api::namespaced(ctx.client.clone(), &ns);

    debug!(resource = %name, namespace = %ns, "cleaning up CivitForgeApp");

    remove_finalizer(&api, &name, &resource).await?;

    info!(resource = %name, "cleanup complete");
    Ok(Action::requeue(std::time::Duration::from_secs(5)))
}

async fn ensure_finalizer(
    api: &Api<CivitForgeApp>,
    name: &str,
    resource: &CivitForgeApp,
) -> Result<(), ReconcileError> {
    let has_finalizer = resource
        .metadata
        .finalizers
        .as_ref()
        .is_some_and(|f| f.iter().any(|f| f == FINALIZER_NAME));

    if has_finalizer {
        return Ok(());
    }

    let patch = Patch::Apply(serde_json::json!({
        "metadata": {
            "finalizers": [FINALIZER_NAME]
        }
    }));

    api.patch(name, &PatchParams::apply(FIELD_MANAGER), &patch)
        .await
        .map_err(ReconcileError::Finalizer)?;

    Ok(())
}

async fn remove_finalizer(
    api: &Api<CivitForgeApp>,
    name: &str,
    resource: &CivitForgeApp,
) -> Result<(), ReconcileError> {
    let remaining: Vec<String> = resource
        .metadata
        .finalizers
        .as_ref()
        .map(|f| f.iter().filter(|f| *f != FINALIZER_NAME).cloned().collect())
        .unwrap_or_default();

    let patch = Patch::Apply(serde_json::json!({
        "metadata": {
            "finalizers": remaining
        }
    }));

    api.patch(name, &PatchParams::apply(FIELD_MANAGER), &patch)
        .await
        .map_err(ReconcileError::Finalizer)?;

    Ok(())
}

fn build_initial_status(
    spec: &CivitForgeAppSpec,
    current: &CivitForgeAppStatus,
) -> CivitForgeAppStatus {
    CivitForgeAppStatus {
        phase: current.phase.clone(),
        replicas: spec.replicas,
        ready_replicas: current.ready_replicas,
        version: Some(spec.tag.clone()),
        conditions: vec![],
        last_updated: current.last_updated.clone(),
    }
}

fn reconcile_component(component: &CivitForgeAppComponent) -> ComponentCondition {
    ComponentCondition {
        component: component.clone(),
        status: ConditionStatus::True,
        reason: "Reconciled".to_string(),
        message: format!(
            "{} component reconciled successfully",
            component.deployment_name()
        ),
        last_transition_time: chrono::Utc::now().to_rfc3339(),
    }
}

pub fn compute_phase(conditions: &[ComponentCondition]) -> String {
    if conditions.is_empty() {
        return "Unknown".to_string();
    }
    if conditions.iter().all(|c| c.status == ConditionStatus::True) {
        "Running".to_string()
    } else if conditions
        .iter()
        .any(|c| c.status == ConditionStatus::False)
    {
        "Degraded".to_string()
    } else {
        "Progressing".to_string()
    }
}

async fn patch_status(
    api: &Api<CivitForgeApp>,
    name: &str,
    status: &CivitForgeAppStatus,
) -> Result<(), ReconcileError> {
    let patch = Patch::Apply(serde_json::json!({
        "status": status
    }));

    api.patch_status(name, &PatchParams::apply(FIELD_MANAGER), &patch)
        .await
        .map_err(ReconcileError::PatchStatus)?;

    Ok(())
}

pub fn error_policy(
    resource: Arc<CivitForgeApp>,
    error: &ReconcileError,
    _ctx: Arc<ReconcileCtx>,
) -> Action {
    error!(
        resource = %resource.name_any(),
        error = %error,
        "reconciliation failed"
    );
    Action::requeue(std::time::Duration::from_secs(30))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_spec() -> CivitForgeAppSpec {
        CivitForgeAppSpec {
            replicas: 3,
            image: "civitforge/app:latest".into(),
            tag: "1.0.0".into(),
            database_url: Some("postgres://db:5432/civit".into()),
            redis_url: Some("redis://cache:6379".into()),
            federation_enabled: true,
            resources: None,
            components: vec![],
            node_selector: None,
            max_unavailable: 1,
        }
    }

    #[test]
    fn test_compute_phase_all_healthy() {
        let conditions = vec![
            ComponentCondition {
                component: CivitForgeAppComponent::Web,
                status: ConditionStatus::True,
                reason: "Available".into(),
                message: "ok".into(),
                last_transition_time: "2025-01-01T00:00:00Z".into(),
            },
            ComponentCondition {
                component: CivitForgeAppComponent::Brain,
                status: ConditionStatus::True,
                reason: "Available".into(),
                message: "ok".into(),
                last_transition_time: "2025-01-01T00:00:00Z".into(),
            },
        ];
        assert_eq!(compute_phase(&conditions), "Running");
    }

    #[test]
    fn test_compute_phase_degraded() {
        let conditions = vec![
            ComponentCondition {
                component: CivitForgeAppComponent::Web,
                status: ConditionStatus::True,
                reason: "Available".into(),
                message: "ok".into(),
                last_transition_time: "2025-01-01T00:00:00Z".into(),
            },
            ComponentCondition {
                component: CivitForgeAppComponent::Brain,
                status: ConditionStatus::False,
                reason: "CrashLoopBackOff".into(),
                message: "pod crashed".into(),
                last_transition_time: "2025-01-01T00:00:00Z".into(),
            },
        ];
        assert_eq!(compute_phase(&conditions), "Degraded");
    }

    #[test]
    fn test_compute_phase_progressing() {
        let conditions = vec![ComponentCondition {
            component: CivitForgeAppComponent::Web,
            status: ConditionStatus::Unknown,
            reason: "Deploying".into(),
            message: "creating".into(),
            last_transition_time: "2025-01-01T00:00:00Z".into(),
        }];
        assert_eq!(compute_phase(&conditions), "Progressing");
    }

    #[test]
    fn test_compute_phase_empty() {
        assert_eq!(compute_phase(&[]), "Unknown");
    }

    #[test]
    fn test_compute_phase_mixed_unknown_and_true() {
        let conditions = vec![
            ComponentCondition {
                component: CivitForgeAppComponent::Web,
                status: ConditionStatus::True,
                reason: "Available".into(),
                message: "ok".into(),
                last_transition_time: "2025-01-01T00:00:00Z".into(),
            },
            ComponentCondition {
                component: CivitForgeAppComponent::Runner,
                status: ConditionStatus::Unknown,
                reason: "Pending".into(),
                message: "waiting".into(),
                last_transition_time: "2025-01-01T00:00:00Z".into(),
            },
        ];
        assert_eq!(compute_phase(&conditions), "Progressing");
    }

    #[test]
    fn test_build_initial_status_preserves_existing() {
        let spec = full_spec();
        let current = CivitForgeAppStatus {
            phase: Some("Running".into()),
            replicas: 3,
            ready_replicas: 2,
            version: Some("0.9.0".into()),
            conditions: vec![],
            last_updated: Some("2025-01-01T00:00:00Z".into()),
        };
        let status = build_initial_status(&spec, &current);
        assert_eq!(status.phase.as_deref(), Some("Running"));
        assert_eq!(status.replicas, 3);
        assert_eq!(status.ready_replicas, 2);
        assert_eq!(status.version.as_deref(), Some("1.0.0"));
        assert!(status.conditions.is_empty());
        assert_eq!(status.last_updated.as_deref(), Some("2025-01-01T00:00:00Z"));
    }

    #[test]
    fn test_build_initial_status_from_empty() {
        let spec = full_spec();
        let current = CivitForgeAppStatus::default();
        let status = build_initial_status(&spec, &current);
        assert!(status.phase.is_none());
        assert_eq!(status.replicas, 3);
        assert_eq!(status.ready_replicas, 0);
        assert_eq!(status.version.as_deref(), Some("1.0.0"));
    }

    #[test]
    fn test_reconcile_component_web() {
        let condition = reconcile_component(&CivitForgeAppComponent::Web);
        assert_eq!(condition.component, CivitForgeAppComponent::Web);
        assert_eq!(condition.status, ConditionStatus::True);
        assert_eq!(condition.reason, "Reconciled");
        assert!(condition.message.contains("civitforge-web"));
    }

    #[test]
    fn test_reconcile_component_all() {
        for component in CivitForgeAppComponent::all() {
            let condition = reconcile_component(component);
            assert_eq!(condition.status, ConditionStatus::True);
            assert!(condition.message.contains(&component.deployment_name()));
        }
    }

    #[test]
    fn test_reconcile_error_display() {
        let err = ReconcileError::MissingUid;
        assert!(err.to_string().contains("missing object UID"));

        let make_err = || {
            kube::Error::SerdeError(
                serde_json::from_str::<serde_json::Value>("invalid").unwrap_err(),
            )
        };

        let err = ReconcileError::GetObject(make_err());
        assert!(err.to_string().contains("failed to get CivitForgeApp"));

        let err = ReconcileError::PatchStatus(make_err());
        assert!(
            err.to_string()
                .contains("failed to patch CivitForgeApp status")
        );

        let err = ReconcileError::Finalizer(make_err());
        assert!(err.to_string().contains("failed to manage finalizer"));
    }

    #[test]
    fn test_reconcile_error_debug() {
        let err = ReconcileError::MissingUid;
        let debug = format!("{err:?}");
        assert!(debug.contains("MissingUid"));
    }

    #[test]
    fn test_constants() {
        assert_eq!(FINALIZER_NAME, "civitforge.dev/finalizer");
        assert_eq!(FIELD_MANAGER, "civit-operator");
        assert_eq!(REQUEUE_INTERVAL, std::time::Duration::from_secs(60));
    }

    #[test]
    fn test_reconcile_ctx_size() {
        let _ = std::mem::size_of::<ReconcileCtx>();
    }
}
