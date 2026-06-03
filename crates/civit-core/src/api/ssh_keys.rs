#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct SshKeyResponse {
    pub id: String,
    pub user_id: String,
    pub key_type: String,
    pub public_key: String,
    pub fingerprint: String,
    pub label: String,
    pub created_at: String,
}

impl From<crate::db::SshKey> for SshKeyResponse {
    fn from(k: crate::db::SshKey) -> Self {
        Self {
            id: k.id.to_string(),
            user_id: k.user_id.to_string(),
            key_type: k.key_type,
            public_key: k.public_key,
            fingerprint: k.fingerprint,
            label: k.label,
            created_at: k.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AddSshKeyRequest {
    pub key_type: String,
    pub public_key: String,
    pub fingerprint: String,
    pub label: Option<String>,
}

pub async fn list_ssh_keys(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid user id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.list_ssh_keys(user_uuid).await {
        Ok(keys) => {
            let out: Vec<SshKeyResponse> = keys.into_iter().map(Into::into).collect();
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn add_ssh_key(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    _auth: AuthUser,
    Json(req): Json<AddSshKeyRequest>,
) -> impl IntoResponse {
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid user id".into()).error_response()),
            )
                .into_response();
        }
    };

    if req.public_key.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::Config("public_key required".into()).error_response()),
        )
            .into_response();
    }

    if req.fingerprint.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::Config("fingerprint required".into()).error_response()),
        )
            .into_response();
    }

    let label = req.label.as_deref().unwrap_or("");

    match state
        .db
        .add_ssh_key(
            user_uuid,
            &req.key_type,
            &req.public_key,
            &req.fingerprint,
            label,
        )
        .await
    {
        Ok(key) => (StatusCode::CREATED, Json(SshKeyResponse::from(key))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_ssh_key(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let key_uuid = match Uuid::parse_str(&key_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid key id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.delete_ssh_key(key_uuid).await {
        Ok(()) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("ssh key not found".into()).error_response()),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_ssh_key() -> crate::db::SshKey {
        crate::db::SshKey {
            id: Uuid::nil(),
            user_id: Uuid::nil(),
            key_type: "ssh-ed25519".into(),
            public_key: "AAAAC3NzaC1lZDI1NTE5AAAAI...".into(),
            fingerprint: "SHA256:abc123def456".into(),
            label: "my-laptop".into(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_ssh_key_response_from_db() {
        let key = make_ssh_key();
        let resp = SshKeyResponse::from(key);
        assert_eq!(resp.id, Uuid::nil().to_string());
        assert_eq!(resp.key_type, "ssh-ed25519");
        assert_eq!(resp.fingerprint, "SHA256:abc123def456");
        assert_eq!(resp.label, "my-laptop");
    }

    #[test]
    fn test_ssh_key_response_serialization() {
        let key = make_ssh_key();
        let resp = SshKeyResponse::from(key);
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"key_type\":\"ssh-ed25519\""));
        assert!(json.contains("\"fingerprint\":\"SHA256:abc123def456\""));
        assert!(json.contains("\"label\":\"my-laptop\""));
    }

    #[test]
    fn test_add_ssh_key_request_parse() {
        let json = r#"{"key_type":"ssh-ed25519","public_key":"AAAAC3NzaC1lZDI1NTE5AAAAI...","fingerprint":"SHA256:abc123"}"#;
        let req: AddSshKeyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.key_type, "ssh-ed25519");
        assert_eq!(req.fingerprint, "SHA256:abc123");
        assert!(req.label.is_none());
    }

    #[test]
    fn test_add_ssh_key_request_with_label() {
        let json = r#"{"key_type":"ssh-rsa","public_key":"ssh-rsa AAAA...","fingerprint":"SHA256:xyz","label":"work-desktop"}"#;
        let req: AddSshKeyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.label.as_deref(), Some("work-desktop"));
    }

    #[test]
    fn test_add_ssh_key_request_missing_fields() {
        let json = r#"{"key_type":"ssh-ed25519"}"#;
        let result = serde_json::from_str::<AddSshKeyRequest>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_ssh_key_response_empty_label() {
        let mut key = make_ssh_key();
        key.label = String::new();
        let resp = SshKeyResponse::from(key);
        assert_eq!(resp.label, "");
    }
}
