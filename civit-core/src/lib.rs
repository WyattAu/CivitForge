#![forbid(unsafe_code)]

pub mod api;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod federation;
pub mod git;

pub use config::AppConfig;
pub use error::{CoreError, Result};
