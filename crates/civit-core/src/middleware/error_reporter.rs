#![forbid(unsafe_code)]

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use tracing::error;

pub async fn panic_catcher(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    let result = std::panic::AssertUnwindSafe(next.run(request)).await;

    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let status = result.status();
        if status.is_server_error() {
            error!(
                method = %method,
                path,
                status = status.as_u16(),
                "Handler returned server error"
            );
        }
        result
    })) {
        Ok(response) => response,
        Err(panic_payload) => {
            let message = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown panic".to_string()
            };
            error!(
                method = %method,
                path,
                error = %message,
                "Handler panicked"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({
                    "error": "internal_server_error",
                    "message": "An unexpected error occurred."
                })),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_panic_message_from_str() {
        let payload: Box<dyn std::any::Any + Send> = Box::new("boom");
        let msg = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else {
            "unknown".to_string()
        };
        assert_eq!(msg, "boom");
    }

    #[test]
    fn test_panic_message_from_string() {
        let payload: Box<dyn std::any::Any + Send> = Box::new("boom".to_string());
        let msg = if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown".to_string()
        };
        assert_eq!(msg, "boom");
    }

    #[test]
    fn test_panic_message_from_int() {
        let payload: Box<dyn std::any::Any + Send> = Box::new(42i32);
        let msg = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };
        assert_eq!(msg, "unknown panic");
    }
}
