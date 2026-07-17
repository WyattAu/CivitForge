#![forbid(unsafe_code)]

pub mod shared_types;

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
pub mod federation;
pub mod git;
pub mod health;
pub mod ldap;
pub mod license_scanner;
pub mod loadtest;
pub mod merge_queue;
pub mod middleware;
pub mod mirror;
pub mod notifications;
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
pub mod secrets;
pub mod shutdown;
pub mod ssh;
pub mod storage;
pub mod telemetry;
pub mod vuln_scanner;
pub mod security_scanner;
pub mod compliance;
pub mod audit_trail;
pub mod deployment_strategy;
pub mod infrastructure;
pub mod service_mesh;
pub mod test_coverage;
pub mod code_quality;
pub mod performance_testing;
pub mod data_archival;
pub mod data_migration;
pub mod network_policy;
pub mod encryption;
pub mod acl;
pub mod workflow_engine;
pub mod automation_rules;
pub mod scheduled_tasks;
pub mod log_aggregation;
pub mod distributed_tracing_v2;
pub mod dashboard_reporting;
pub mod pipeline_secrets;
pub mod pipeline_runners;
pub mod environment_variables;
pub mod test_suite_management;
pub mod review_automation;
pub mod quality_gates;
pub mod firewall;
pub mod intrusion_detection;
pub mod ddos_protection;
pub mod object_storage;
pub mod backup_encryption;
pub mod data_retention;
pub mod database_replication;
pub mod data_residency;
pub mod pipeline_action_reviews;
pub mod environment_deployment;
pub mod cache_hit_analysis;

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

#[cfg(test)]
mod workflow_engine_tests;
#[cfg(test)]
mod automation_rules_tests;
#[cfg(test)]
mod security_scanner_tests;
#[cfg(test)]
mod compliance_tests;
#[cfg(test)]
mod audit_trail_tests;
