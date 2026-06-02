//! API error types.

use serde::{Deserialize, Serialize};

/// Structured API error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Machine-readable error code.
    pub code: ApiErrorCode,
    /// Human-readable error message.
    pub message: String,
    /// Additional details (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Machine-readable error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApiErrorCode {
    #[serde(rename = "UNAUTHORIZED")]
    Unauthorized,
    #[serde(rename = "FORBIDDEN")]
    Forbidden,
    #[serde(rename = "NOT_FOUND")]
    NotFound,
    #[serde(rename = "CONFLICT")]
    Conflict,
    #[serde(rename = "VALIDATION_ERROR")]
    ValidationError,
    #[serde(rename = "RATE_LIMITED")]
    RateLimited,
    #[serde(rename = "INTERNAL_ERROR")]
    InternalError,
    #[serde(rename = "SERVICE_UNAVAILABLE")]
    ServiceUnavailable,
}

impl ApiErrorCode {
    /// Returns the HTTP status code for this error.
    pub fn http_status(&self) -> u16 {
        match self {
            ApiErrorCode::Unauthorized => 401,
            ApiErrorCode::Forbidden => 403,
            ApiErrorCode::NotFound => 404,
            ApiErrorCode::Conflict => 409,
            ApiErrorCode::ValidationError => 422,
            ApiErrorCode::RateLimited => 429,
            ApiErrorCode::InternalError => 500,
            ApiErrorCode::ServiceUnavailable => 503,
        }
    }
}

impl std::fmt::Display for ApiErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).unwrap_or_else(|_| "UNKNOWN".into());
        write!(f, "{s}")
    }
}
