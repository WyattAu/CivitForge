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

    #[error("search error: {0}")]
    Search(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("too many requests: {0}")]
    TooManyRequests(String),
}

impl From<civit_auth::error::AuthError> for CoreError {
    fn from(e: civit_auth::error::AuthError) -> Self {
        match e {
            civit_auth::error::AuthError::Auth(msg) => CoreError::Auth(msg),
            civit_auth::error::AuthError::Forbidden(msg) => CoreError::Forbidden(msg),
            civit_auth::error::AuthError::Config(msg) => CoreError::Config(msg),
            civit_auth::error::AuthError::Database(msg) => CoreError::Database(msg),
            civit_auth::error::AuthError::Jwt(e) => CoreError::Jwt(e),
            civit_auth::error::AuthError::Ldap(msg) => CoreError::Auth(msg),
            civit_auth::error::AuthError::Internal(msg) => CoreError::Internal(msg),
            civit_auth::error::AuthError::NotFound(msg) => CoreError::NotFound(msg),
            civit_auth::error::AuthError::BadRequest(msg) => CoreError::BadRequest(msg),
            civit_auth::error::AuthError::TooManyRequests(msg) => CoreError::TooManyRequests(msg),
        }
    }
}

/// Automatic conversion from civit-db errors to core errors.
/// This allows civit-core to use civit-db's DbRepository transparently.
impl From<civit_db::error::DbError> for CoreError {
    fn from(e: civit_db::error::DbError) -> Self {
        match e {
            civit_db::error::DbError::Database(msg) => CoreError::Database(msg),
            civit_db::error::DbError::Auth(msg) => CoreError::Auth(msg),
        }
    }
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
            Self::Search(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::TooManyRequests(_) => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    pub fn error_response(&self) -> ErrorResponse {
        let message = match self {
            Self::Database(detail) => {
                tracing::warn!("database error: {detail}");
                "internal server error".to_string()
            }
            Self::Internal(detail) => {
                tracing::warn!("internal error: {detail}");
                "internal server error".to_string()
            }
            Self::Io(e) => {
                tracing::warn!("io error: {e}");
                "internal server error".to_string()
            }
            Self::Config(msg) => {
                tracing::warn!("config error: {msg}");
                "internal server error".to_string()
            }
            Self::Git(msg) => {
                tracing::warn!("git error: {msg}");
                "internal server error".to_string()
            }
            Self::Search(msg) => {
                tracing::warn!("search error: {msg}");
                "internal server error".to_string()
            }
            other => other.to_string(),
        };
        ErrorResponse { error: message }
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

    #[test]
    fn test_all_status_codes() {
        assert_eq!(
            CoreError::Config("x".into()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            CoreError::Database("x".into()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            CoreError::Auth("x".into()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            CoreError::Forbidden("x".into()).status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            CoreError::NotFound("x".into()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            CoreError::Git("x".into()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            CoreError::Federation("x".into()).status_code(),
            StatusCode::BAD_GATEWAY
        );
        assert_eq!(
            CoreError::Internal("x".into()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            CoreError::Json(serde_json::from_str::<i32>("not a number").unwrap_err()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            CoreError::Jwt(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken
            ))
            .status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            CoreError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "not found"
            ))
            .status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_all_display_impls() {
        assert!(
            CoreError::Config("cfg".into())
                .to_string()
                .contains("configuration error: cfg")
        );
        assert!(
            CoreError::Database("db".into())
                .to_string()
                .contains("database error: db")
        );
        assert!(
            CoreError::Auth("auth".into())
                .to_string()
                .contains("authentication error: auth")
        );
        assert!(
            CoreError::Forbidden("forbid".into())
                .to_string()
                .contains("authorization error: forbid")
        );
        assert!(
            CoreError::NotFound("nf".into())
                .to_string()
                .contains("not found: nf")
        );
        assert!(
            CoreError::Git("git".into())
                .to_string()
                .contains("git error: git")
        );
        assert!(
            CoreError::Federation("fed".into())
                .to_string()
                .contains("federation error: fed")
        );
        assert!(
            CoreError::Internal("int".into())
                .to_string()
                .contains("internal error: int")
        );
        assert!(
            CoreError::Json(serde_json::from_str::<i32>("x").unwrap_err())
                .to_string()
                .contains("json error:")
        );
        assert!(
            CoreError::Jwt(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken
            ))
            .to_string()
            .contains("jwt error:")
        );
        assert!(
            CoreError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "io"))
                .to_string()
                .contains("io error: io")
        );
    }

    #[test]
    fn test_error_response() {
        let err = CoreError::Auth("unauthorized".into());
        let resp = err.error_response();
        assert_eq!(resp.error, "authentication error: unauthorized");
    }

    #[test]
    fn test_error_response_serialization() {
        let err = CoreError::NotFound("resource".into());
        let resp = err.error_response();
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("resource"));
        assert!(json.contains("error"));
    }

    #[test]
    fn test_error_response_for_all_variants() {
        let errors: Vec<CoreError> = vec![
            CoreError::Config("c".into()),
            CoreError::Database("d".into()),
            CoreError::Auth("a".into()),
            CoreError::Forbidden("f".into()),
            CoreError::NotFound("n".into()),
            CoreError::Git("g".into()),
            CoreError::Federation("fe".into()),
            CoreError::Internal("i".into()),
            CoreError::Json(serde_json::from_str::<i32>("x").unwrap_err()),
            CoreError::Jwt(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            )),
            CoreError::Io(std::io::Error::other("o")),
        ];
        for err in errors {
            let resp = err.error_response();
            // Sanitized errors (Database, Internal, Io, Config, Git) return
            // generic "internal server error" to clients — assert no leak
            assert!(!resp.error.contains("database error:"));
            assert!(!resp.error.contains("internal error:"));
            assert!(!resp.error.contains("io error:"));
            assert!(!resp.error.contains("configuration error:"));
            assert!(!resp.error.contains("git error:"));
        }
    }

    #[test]
    fn test_result_type_alias() {
        let res: Result<String> = Ok("ok".into());
        assert!(res.is_ok());
        let res: Result<String> = Err(CoreError::Internal("err".into()));
        assert!(res.is_err());
    }

    #[test]
    fn test_from_json_error() {
        let json_err = serde_json::from_str::<i32>("not a number").unwrap_err();
        let core_err: CoreError = json_err.into();
        assert!(matches!(core_err, CoreError::Json(_)));
    }

    #[test]
    fn test_from_jwt_error() {
        let jwt_err =
            jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::ExpiredSignature);
        let core_err: CoreError = jwt_err.into();
        assert!(matches!(core_err, CoreError::Jwt(_)));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broke");
        let core_err: CoreError = io_err.into();
        assert!(matches!(core_err, CoreError::Io(_)));
    }
}
