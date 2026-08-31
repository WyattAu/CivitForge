use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("authentication error: {0}")]
    Auth(String),

    #[error("authorization error: {0}")]
    Forbidden(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("ldap error: {0}")]
    Ldap(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("too many requests: {0}")]
    TooManyRequests(String),
}

pub type Result<T> = std::result::Result<T, AuthError>;
