#![forbid(unsafe_code)]

pub mod api;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod events;
pub mod federation;
pub mod git;
pub mod loadtest;
pub mod scaling;
pub mod ssh;
pub mod telemetry;

pub use config::AppConfig;
pub use db::{DatabasePool, DbRepository};
pub use error::{CoreError, Result};
pub use events::{Event, EventBus, EventCategory, EventPayload, EventSubscriber, WebSocketManager};
pub use ssh::{SshAuthService, SshConfig, SshServer};
