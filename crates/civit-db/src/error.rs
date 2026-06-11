#![forbid(unsafe_code)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Database(String),

    #[error("authentication error: {0}")]
    Auth(String),
}

pub type Result<T> = std::result::Result<T, DbError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = DbError::Database("connection refused".into());
        assert_eq!(err.to_string(), "database error: connection refused");

        let err = DbError::Auth("session expired".into());
        assert_eq!(err.to_string(), "authentication error: session expired");
    }

    #[test]
    fn test_result_type() {
        let res: Result<()> = Ok(());
        assert!(res.is_ok());
        let res: Result<()> = Err(DbError::Database("fail".into()));
        assert!(res.is_err());
    }
}
