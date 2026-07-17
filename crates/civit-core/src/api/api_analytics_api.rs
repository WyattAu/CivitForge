#![forbid(unsafe_code)]

//! Consolidated API analytics routes.
//!
//! All versioned API analytics handlers in a single module,
//! with each version in its own sub-module for namespace isolation.

use crate::api::AppState;
use axum::Router;

pub mod v1;
pub mod v2;
pub mod v3;
pub mod v5;
pub mod v6;
pub mod v9;
pub mod v11;
pub mod v12;
pub mod v13;
pub mod v14;
pub mod v15;
pub mod v16;
pub mod v17;
pub mod v18;
pub mod v19;
pub mod v20;
pub mod v21;
pub mod v23;
pub mod v24;

pub fn api_analytics_api_routes() -> Router<AppState> {
    v1::routes()
        .merge(v2::routes())
        .merge(v3::routes())
        .merge(v5::routes())
        .merge(v6::routes())
        .merge(v9::routes())
        .merge(v11::routes())
        .merge(v12::routes())
        .merge(v13::routes())
        .merge(v14::routes())
        .merge(v15::routes())
        .merge(v16::routes())
        .merge(v17::routes())
        .merge(v18::routes())
        .merge(v19::routes())
        .merge(v20::routes())
        .merge(v21::routes())
        .merge(v23::routes())
        .merge(v24::routes())
}
