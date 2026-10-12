#![forbid(unsafe_code)]

pub use civit_security as security;
pub use civit_types as shared_types;

pub mod api;
pub mod audit;
pub mod auth;
pub mod backup;
pub mod cache;
pub mod chaos;
pub mod circuit_breaker;
pub mod config;
pub mod db;
pub mod deploy_keys;
pub mod docs;
pub mod error;
pub mod event_queues;
pub mod events;
pub mod feature_flags;
pub mod flags_store;
pub use civit_federation as federation;
pub mod git;
pub mod health;
pub mod ldap;
// license_scanner — moved to civit-security (re-exported via security::)
pub mod loadtest;
pub mod merge_queue;
pub mod middleware;
pub mod mirror;
pub mod notifications;
pub mod performance;
pub mod policy;
pub mod project_boards;
pub mod protection;
pub mod provenance;
pub mod realtime;
pub mod release;
pub mod release_manager;
pub mod resilience;
pub mod rollout_controller;
pub mod runner;
pub mod scaling;
pub mod scheduler;
pub mod search;
pub mod static_assets;
// secrets — moved to civit-security (re-exported via security::)
pub mod search_index_sync;
pub mod shutdown;
pub mod ssh;
pub mod storage;
pub use civit_telemetry as telemetry;
// vuln_scanner, security_scanner, compliance, audit_trail — moved to civit-security
pub mod data_archival;
pub mod data_migration;
pub mod deployment_strategy;
pub mod infrastructure;
pub mod service_mesh;
// network_policy, encryption, acl — moved to civit-security
pub use civit_workflow as workflow;
pub mod environment_variables;
pub mod pipeline_runners;
pub mod pipeline_secrets;
// firewall, intrusion_detection, ddos_protection — moved to civit-security
pub mod marketplace;
pub mod object_storage;
// backup_encryption — moved to civit-security
pub mod compliance_config;
pub mod compliance_reporting;
pub mod data_portability;
pub mod data_residency;
pub mod data_retention;
pub mod database_replication;
pub mod multi_region;
pub mod sla_monitoring;

pub use telemetry::apm::{
    ApmConfig, ApmDashboard, ApmRecorder, ApmSpan, ApmTransaction, TransactionStats,
};
pub use telemetry::distributed_tracing::{
    DistributedTracer, DistributedTracingConfig, TRACEPARENT_HEADER, TRACESTATE_HEADER, TraceEvent,
    TraceSpan, format_traceparent, generate_span_id, generate_trace_id, parse_traceparent,
};
pub use telemetry::error_tracking::{ErrorRecord, ErrorSummary, ErrorTracker, ErrorTrackingConfig};
pub mod webhook;
pub mod webhooks;
pub mod wiki;

pub use config::AppConfig;
pub use db::{DatabasePool, DbRepository};
pub use error::{CoreError, Result};
pub use events::{
    Event, EventCategory, EventPayload, EventPublisher, EventSubscription, PublishedEvent,
    WebSocketManager,
};
pub use ssh::{SshAuthService, SshConfig, SshServer};

pub use security::{
    acl, audit_compliance, audit_trail, backup_encryption, compliance, ddos_protection, encryption,
    firewall, intrusion_detection, license_scanner, network_policy, secrets, security_scanner,
    vuln_scanner,
};
