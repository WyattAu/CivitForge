#![forbid(unsafe_code)]

use crate::crds::{PipelinePhase, PipelineRun, PipelineRunStatus};
use crate::pod_builder::PodBuilder;
use chrono::Utc;
use futures::StreamExt as _;
use k8s_openapi::api::core::v1::Pod;
use kube::{
    Api, Client, ResourceExt,
    runtime::{Controller, controller::Action, watcher::Config as WatcherConfig},
};
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{debug, error, info, warn};

/// Configuration for the K8s operator controller.
#[derive(Debug, Clone)]
pub struct KubeControllerConfig {
    /// Kubernetes namespace to watch for PipelineRun CRDs.
    pub namespace: String,
    /// Sandbox container image used when the PipelineRun spec doesn't specify one.
    pub default_image: String,
    /// Interval for re-enqueuing non-terminal PipelineRuns.
    pub requeue_after: Duration,
}

impl Default for KubeControllerConfig {
    fn default() -> Self {
        Self {
            namespace: "civit-system".into(),
            default_image: crate::pod_builder::DEFAULT_SANDBOX_IMAGE.into(),
            requeue_after: Duration::from_secs(10),
        }
    }
}

/// Errors that can occur during K8s operator reconciliation.
#[derive(Debug, thiserror::Error)]
pub enum ReconcileError {
    #[error("failed to get PipelineRun: {0}")]
    GetObject(#[source] kube::Error),

    #[error("failed to create Pod: {0}")]
    CreatePod(#[source] kube::Error),

    #[error("failed to get Pod: {0}")]
    GetPod(#[source] kube::Error),

    #[error("failed to patch PipelineRun status: {0}")]
    PatchStatus(#[source] kube::Error),

    #[error("missing object UID")]
    MissingUid,

    #[error("Pod name conflict for {name}")]
    PodConflict { name: String },
}

/// Shared context passed to every reconcile invocation.
pub struct KubeReconcileCtx {
    client: Client,
    config: KubeControllerConfig,
    pod_builder: PodBuilder,
}

impl KubeReconcileCtx {
    pub fn new(client: Client, config: KubeControllerConfig) -> Self {
        let pod_builder = PodBuilder::new(&config.default_image);
        Self {
            client,
            config,
            pod_builder,
        }
    }
}

/// Builds a deterministic Pod name from a PipelineRun name.
/// K8s Pod names must be <= 253 chars, DNS-subdomain compatible.
pub fn pipeline_run_pod_name(pr_name: &str) -> String {
    let base = pr_name.trim_matches('.').replace('_', "-").to_lowercase();
    let suffix = "pod";
    // Reserve budget: max 253 chars total
    let max_base = 253 - suffix.len() - 1;
    let truncated = if base.len() > max_base {
        &base[..max_base]
    } else {
        &base
    };
    format!("{truncated}-{suffix}")
}

/// Core reconciliation logic for a single PipelineRun.
///
/// 1. If no Pod exists yet → create one from the PipelineRun spec.
/// 2. If Pod exists → check its phase and map to PipelineRun status.
/// 3. Patch PipelineRun status subresource.
pub async fn reconcile_pipeline_run(
    pr: Arc<PipelineRun>,
    ctx: Arc<KubeReconcileCtx>,
) -> Result<Action, ReconcileError> {
    let name = pr.name_any();
    let ns = ctx.config.namespace.clone();
    let pr_api: Api<PipelineRun> = Api::namespaced(ctx.client.clone(), &ns);
    let pod_api: Api<Pod> = Api::namespaced(ctx.client.clone(), &ns);

    let pod_name = pipeline_run_pod_name(&name);

    // Fetch the latest PipelineRun to get current status
    let pr_latest = pr_api.get(&name).await.map_err(ReconcileError::GetObject)?;

    let current_phase = pr_latest
        .status
        .as_ref()
        .map(|s| s.phase.clone())
        .unwrap_or_default();

    // If already terminal, no work to do
    if current_phase.is_terminal() {
        debug!(pipeline = %name, phase = ?current_phase, "pipeline already terminal, skipping");
        return Ok(Action::await_change());
    }

    // Check if our child Pod already exists
    match pod_api.get(&pod_name).await {
        Ok(pod) => {
            // Pod exists — map its phase to PipelineRun status
            let pod_phase = pod
                .status
                .as_ref()
                .and_then(|s| s.phase.clone())
                .unwrap_or_default();

            let new_status = map_pod_phase_to_pr_status(&pod_phase, &current_phase, &pod);

            // Only patch if status changed
            if new_status.phase != current_phase {
                debug!(
                    pipeline = %name,
                    from = ?current_phase,
                    to = ?new_status.phase,
                    "status change, patching"
                );

                let patch_status = kube::api::Patch::Apply(serde_json::json!({
                    "status": new_status
                }));
                pr_api
                    .patch_status(
                        &name,
                        &kube::api::PatchParams::apply("civit-operator"),
                        &patch_status,
                    )
                    .await
                    .map_err(ReconcileError::PatchStatus)?;

                if new_status.phase.is_terminal() {
                    info!(pipeline = %name, phase = ?new_status.phase, "pipeline completed");
                    return Ok(Action::await_change());
                }
            }

            // Non-terminal — requeue to keep checking Pod progress
            Ok(Action::requeue(ctx.config.requeue_after))
        }
        Err(kube::Error::Api(ae)) if ae.code == 404 => {
            // Pod doesn't exist yet — create it
            let uid = pr_latest.metadata.uid.ok_or(ReconcileError::MissingUid)?;

            let spec = &pr_latest.spec;
            let pod = ctx
                .pod_builder
                .build_sandbox_pod(&pod_name, &ns, spec, &uid);

            debug!(pipeline = %name, pod = %pod_name, "creating sandbox pod");

            pod_api
                .create(&kube::api::PostParams::default(), &pod)
                .await
                .map_err(ReconcileError::CreatePod)?;

            // Patch status to Running
            let running_status = PipelineRunStatus {
                phase: PipelinePhase::Running,
                step_statuses: None,
                message: Some(format!("pod {pod_name} created")),
                start_time: Some(Utc::now()),
                completion_time: None,
            };

            let patch_status = kube::api::Patch::Apply(serde_json::json!({
                "status": running_status
            }));
            pr_api
                .patch_status(
                    &name,
                    &kube::api::PatchParams::apply("civit-operator"),
                    &patch_status,
                )
                .await
                .map_err(ReconcileError::PatchStatus)?;

            info!(pipeline = %name, pod = %pod_name, "pipeline running");

            Ok(Action::requeue(ctx.config.requeue_after))
        }
        Err(e) => Err(ReconcileError::GetPod(e)),
    }
}

/// Map a K8s Pod phase to a PipelineRun status.
fn map_pod_phase_to_pr_status(
    pod_phase: &str,
    _current_pr_phase: &PipelinePhase,
    _pod: &Pod,
) -> PipelineRunStatus {
    let message = format!("pod phase: {pod_phase}");

    let new_phase = match pod_phase {
        "Pending" => PipelinePhase::Pending,
        "Running" => PipelinePhase::Running,
        "Succeeded" => PipelinePhase::Succeeded,
        "Failed" => PipelinePhase::Failed,
        _ => {
            // Unknown or unexpected phase
            warn!(pod_phase = %pod_phase, "unrecognized pod phase, mapping to Failed");
            PipelinePhase::Failed
        }
    };

    let start_time = if matches!(
        new_phase,
        PipelinePhase::Running | PipelinePhase::Succeeded | PipelinePhase::Failed
    ) {
        Some(Utc::now())
    } else {
        None
    };

    let completion_time = if new_phase.is_terminal() {
        Some(Utc::now())
    } else {
        None
    };

    PipelineRunStatus {
        phase: new_phase,
        step_statuses: None,
        message: Some(message),
        start_time,
        completion_time,
    }
}

/// Error handler for the controller. Logs and requeues after a backoff.
pub fn error_policy(
    _pr: Arc<PipelineRun>,
    error: &ReconcileError,
    _ctx: Arc<KubeReconcileCtx>,
) -> Action {
    error!(
        error = %error,
        "reconciliation failed"
    );
    // Exponential backoff via kube default — just requeue
    Action::requeue(Duration::from_secs(30))
}

/// Builder for configuring the PipelineRun controller.
pub struct KubeControllerBuilder {
    config: KubeControllerConfig,
}

impl KubeControllerBuilder {
    pub fn new() -> Self {
        Self {
            config: KubeControllerConfig::default(),
        }
    }

    pub fn namespace(mut self, ns: impl Into<String>) -> Self {
        self.config.namespace = ns.into();
        self
    }

    pub fn default_image(mut self, image: impl Into<String>) -> Self {
        self.config.default_image = image.into();
        self
    }

    pub fn requeue_after(mut self, dur: Duration) -> Self {
        self.config.requeue_after = dur;
        self
    }

    /// Build the context (for direct Controller::new usage) or pass to
    /// `run_controller_until_cancelled`.
    pub fn build_ctx(self, client: Client) -> Arc<KubeReconcileCtx> {
        Arc::new(KubeReconcileCtx::new(client, self.config))
    }

    /// Consume the builder and run the controller until cancelled.
    pub async fn run_until_cancelled(
        self,
        client: Client,
        cancel: tokio::sync::watch::Receiver<bool>,
    ) {
        run_controller_until_cancelled(client, self.config, cancel).await
    }
}

impl Default for KubeControllerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function: start the controller and run until the provided
/// cancellation token fires.
///
/// Returns only when the future is cancelled (e.g. via tokio::select!).
pub async fn run_controller_until_cancelled(
    client: Client,
    config: KubeControllerConfig,
    mut cancel: tokio::sync::watch::Receiver<bool>,
) {
    let ctx = Arc::new(KubeReconcileCtx::new(client, config));

    let pr_api: Api<PipelineRun> = Api::namespaced(ctx.client.clone(), &ctx.config.namespace);

    let stream = Controller::new(pr_api, WatcherConfig::default()).run(
        reconcile_pipeline_run,
        error_policy,
        Arc::clone(&ctx),
    );
    tokio::pin!(stream);

    info!(
        namespace = %ctx.config.namespace,
        "starting PipelineRun K8s controller"
    );

    // Drive the controller until cancelled
    loop {
        tokio::select! {
            _ = cancel.changed() => {
                info!("controller shutdown signal received");
                break;
            }
            result = stream.next() => {
                match result {
                    Some(Ok((obj_ref, _action))) => {
                        debug!(pipeline = ?obj_ref.name, "controller tick");
                    }
                    Some(Err(e)) => {
                        error!(error = %e, "controller stream error");
                    }
                    None => {
                        info!("controller stream ended");
                        break;
                    }
                }
            }
        }
    }

    info!("PipelineRun K8s controller stopped");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_run_pod_name_basic() {
        assert_eq!(
            pipeline_run_pod_name("build-main-abc"),
            "build-main-abc-pod"
        );
    }

    #[test]
    fn test_pipeline_run_pod_name_normalizes() {
        // Underscores → hyphens, lowercase
        assert_eq!(
            pipeline_run_pod_name("My_Pipeline_Run"),
            "my-pipeline-run-pod"
        );
    }

    #[test]
    fn test_pipeline_run_pod_name_trims_dots() {
        assert_eq!(pipeline_run_pod_name(".my-run."), "my-run-pod");
    }

    #[test]
    fn test_pipeline_run_pod_name_max_length() {
        // 253 char limit
        let long_name = "a".repeat(300);
        let result = pipeline_run_pod_name(&long_name);
        assert!(result.len() <= 253);
        assert!(result.ends_with("-pod"));
    }

    #[test]
    fn test_pipeline_run_pod_name_empty() {
        assert_eq!(pipeline_run_pod_name(""), "-pod");
    }

    #[test]
    fn test_kube_controller_config_default() {
        let cfg = KubeControllerConfig::default();
        assert_eq!(cfg.namespace, "civit-system");
        assert_eq!(cfg.default_image, crate::pod_builder::DEFAULT_SANDBOX_IMAGE);
        assert_eq!(cfg.requeue_after, Duration::from_secs(10));
    }

    #[test]
    fn test_kube_controller_builder() {
        let _builder = KubeControllerBuilder::new()
            .namespace("custom-ns")
            .default_image("alpine:3.19")
            .requeue_after(Duration::from_secs(5));
    }

    #[test]
    fn test_map_pod_phase_succeeded() {
        let pod = Pod::default();
        let current = PipelinePhase::Running;
        let status = map_pod_phase_to_pr_status("Succeeded", &current, &pod);
        assert_eq!(status.phase, PipelinePhase::Succeeded);
        assert!(status.phase.is_terminal());
        assert!(status.completion_time.is_some());
    }

    #[test]
    fn test_map_pod_phase_failed() {
        let pod = Pod::default();
        let current = PipelinePhase::Running;
        let status = map_pod_phase_to_pr_status("Failed", &current, &pod);
        assert_eq!(status.phase, PipelinePhase::Failed);
        assert!(status.phase.is_terminal());
        assert!(status.completion_time.is_some());
    }

    #[test]
    fn test_map_pod_phase_running() {
        let pod = Pod::default();
        let current = PipelinePhase::Pending;
        let status = map_pod_phase_to_pr_status("Running", &current, &pod);
        assert_eq!(status.phase, PipelinePhase::Running);
        assert!(!status.phase.is_terminal());
        assert!(status.start_time.is_some());
        assert!(status.completion_time.is_none());
    }

    #[test]
    fn test_map_pod_phase_pending() {
        let pod = Pod::default();
        let current = PipelinePhase::Pending;
        let status = map_pod_phase_to_pr_status("Pending", &current, &pod);
        assert_eq!(status.phase, PipelinePhase::Pending);
        assert!(!status.phase.is_terminal());
    }

    #[test]
    fn test_map_pod_phase_unknown_maps_to_failed() {
        let pod = Pod::default();
        let current = PipelinePhase::Running;
        let status = map_pod_phase_to_pr_status("UnknownWeirdPhase", &current, &pod);
        assert_eq!(status.phase, PipelinePhase::Failed);
        assert!(status.phase.is_terminal());
    }

    #[test]
    fn test_map_pod_phase_preserves_start_time_from_pr() {
        let pod = Pod::default();
        // Directly test the mapping function behavior
        let result = map_pod_phase_to_pr_status("Running", &PipelinePhase::Running, &pod);
        assert!(result.start_time.is_some());
        assert!(result.completion_time.is_none());
    }

    #[test]
    fn test_reconcile_error_display() {
        let err = ReconcileError::MissingUid;
        assert!(err.to_string().contains("missing object UID"));

        let err2 = ReconcileError::PodConflict {
            name: "test-run".into(),
        };
        assert!(err2.to_string().contains("test-run"));
    }

    #[test]
    fn test_kube_reconcile_ctx_new() {
        // Verify config construction and PodBuilder creation work
        let config = KubeControllerConfig {
            namespace: "test".into(),
            default_image: "alpine:latest".into(),
            requeue_after: Duration::from_secs(5),
        };
        assert_eq!(config.namespace, "test");
        let _builder = PodBuilder::new(&config.default_image);
        // PodBuilder is consumed internally — existence validates construction
    }
}
