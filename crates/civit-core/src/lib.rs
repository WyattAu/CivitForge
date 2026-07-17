#![forbid(unsafe_code)]

pub use civit_types as shared_types;
pub use civit_security as security;

pub mod api;
pub mod audit;
pub mod auth;
pub mod backup;
pub mod cache;
pub mod circuit_breaker;
pub mod chaos;
pub mod config;
pub mod db;
pub mod deploy_keys;
pub mod docs;
pub mod error;
pub mod event_queues;
pub mod events;
pub mod feature_flags;
pub use civit_federation as federation;
pub mod git;
pub mod health;
pub mod ldap;
// license_scanner — moved to civit-security (re-exported via security::)
pub mod loadtest;
pub mod merge_queue;
pub mod middleware;
pub mod mirror;
pub mod static_assets;
pub mod notifications;
pub mod project_boards;
pub mod performance;
pub mod policy;
pub mod protection;
pub mod realtime;
pub mod provenance;
pub mod release;
pub mod release_manager;
pub mod resilience;
pub mod runner;
pub mod scaling;
pub mod scheduler;
pub mod search;
// secrets — moved to civit-security (re-exported via security::)
pub mod search_index_sync;
pub mod shutdown;
pub mod ssh;
pub mod storage;
pub use civit_telemetry as telemetry;
// vuln_scanner, security_scanner, compliance, audit_trail — moved to civit-security
pub mod deployment_strategy;
pub mod infrastructure;
pub mod service_mesh;
pub mod test_coverage;
pub mod code_quality;
pub mod performance_testing;
pub mod data_archival;
pub mod data_migration;
// network_policy, encryption, acl — moved to civit-security
pub use civit_workflow as workflow;
pub mod log_aggregation;
pub mod distributed_tracing_v2;
pub mod dashboard_reporting;
pub mod pipeline_secrets;
pub mod pipeline_runners;
pub mod environment_variables;
pub mod test_suite_management;
pub mod review_automation;
pub mod quality_gates;
// firewall, intrusion_detection, ddos_protection — moved to civit-security
pub mod object_storage;
pub mod plugins;
pub mod marketplace;
// backup_encryption — moved to civit-security
pub mod data_retention;
pub mod database_replication;
pub mod data_residency;
pub mod data_portability;
pub mod multi_region;
pub mod tenant_isolation;
pub mod compliance_config;
pub mod sla_monitoring;
pub mod compliance_reporting;

pub use telemetry::apm::{ApmConfig, ApmRecorder, ApmTransaction, ApmSpan, ApmDashboard, TransactionStats};
pub use telemetry::distributed_tracing::{
    DistributedTracer, DistributedTracingConfig, TraceSpan, TraceEvent,
    TRACEPARENT_HEADER, TRACESTATE_HEADER,
    parse_traceparent, format_traceparent, generate_trace_id, generate_span_id,
};
pub use telemetry::error_tracking::{ErrorTracker, ErrorTrackingConfig, ErrorRecord, ErrorSummary};
pub mod webhook;
pub mod webhooks;
pub mod wiki;

pub use config::AppConfig;
pub use db::{DatabasePool, DbRepository};
pub use error::{CoreError, Result};
pub use events::{Event, EventBus, EventCategory, EventPayload, EventSubscriber, EventPublisher, PublishedEvent, EventSubscription, WebSocketManager};
pub use ssh::{SshAuthService, SshConfig, SshServer};

pub use security::{
    security_scanner, compliance, audit_trail, vuln_scanner, license_scanner,
    acl, firewall, intrusion_detection, ddos_protection,
    encryption, backup_encryption, secrets, network_policy,
    audit_compliance,
};
