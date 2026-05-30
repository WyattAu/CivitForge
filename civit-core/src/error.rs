#![forbid(unsafe_code)]

use axum::http::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("authentication error: {0}")]
    Auth(String),

    #[error("authorization error: {0}")]
    Forbidden(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("git error: {0}")]
    Git(String),

    #[error("federation error: {0}")]
    Federation(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("jwt error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    error: String,
}

impl CoreError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Auth(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Git(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Federation(_) => StatusCode::BAD_GATEWAY,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Json(_) => StatusCode::BAD_REQUEST,
            Self::Jwt(_) => StatusCode::UNAUTHORIZED,
            Self::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_response(&self) -> ErrorResponse {
        ErrorResponse {
            error: self.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        let err = CoreError::Auth("bad token".into());
        assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);

        let err = CoreError::NotFound("repo".into());
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);

        let err = CoreError::Forbidden("no access".into());
        assert_eq!(err.status_code(), StatusCode::FORBIDDEN);

        let err = CoreError::Config("bad".into());
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_display() {
        let err = CoreError::Git("clone failed".into());
        assert_eq!(err.to_string(), "git error: clone failed");
    }
}
