#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::error::CoreError;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct TreeEntry {
    pub path: String,
    pub entry_type: String,
    pub size: u64,
    pub last_commit: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BlobResponse {
    pub path: String,
    pub content: String,
    pub size: u64,
    pub encoding: String,
}

#[derive(Debug, Deserialize)]
pub struct TreeQueryParams {
    pub path: Option<String>,
    #[serde(default = "default_ref")]
    pub ref_: Option<String>,
}

fn default_ref() -> Option<String> {
    None
}

pub fn code_browser_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/repos/{owner}/{name}/tree", get(list_tree))
        .route("/api/v1/repos/{owner}/{name}/blob", get(read_blob))
}

/// Convert a gix object error into our CoreError.
fn git_err(e: impl std::fmt::Display) -> CoreError {
    CoreError::Git(e.to_string())
}

/// List files and directories at a given path in a repo's default branch.
pub async fn list_tree(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(params): Query<TreeQueryParams>,
) -> impl IntoResponse {
    let repo_path = state.git_service.repo_path(&owner, &name);

    if !repo_path.join("HEAD").exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response();
    }

    let repo = match gix::open(&repo_path) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let head_id = match repo.head_id() {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let commit_obj = match head_id.object() {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let commit = match commit_obj.try_into_commit() {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let tree_id = match commit.tree_id() {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let tree_obj = match tree_id.object() {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let tree = match tree_obj.try_into_tree() {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let mut entries = Vec::new();
    let prefix = params.path.as_deref().unwrap_or("");
    for entry_result in tree.iter() {
        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue,
        };
        let entry_path = if prefix.is_empty() {
            entry.filename().to_string()
        } else {
            format!("{prefix}/{}", entry.filename())
        };

        let mode = entry.mode();
        let (entry_type, size) = if mode.is_tree() {
            ("dir".to_string(), 0u64)
        } else if mode.is_blob() {
            let sz = entry
                .object()
                .ok()
                .and_then(|o| o.try_into_blob().ok())
                .map(|b| b.data.len() as u64)
                .unwrap_or(0);
            ("file".to_string(), sz)
        } else if mode.is_link() {
            let sz = entry
                .object()
                .ok()
                .and_then(|o| o.try_into_blob().ok())
                .map(|b| b.data.len() as u64)
                .unwrap_or(0);
            ("symlink".to_string(), sz)
        } else if mode.is_commit() {
            ("submodule".to_string(), 0u64)
        } else {
            ("unknown".to_string(), 0u64)
        };

        entries.push(TreeEntry {
            path: entry_path,
            entry_type,
            size,
            last_commit: None,
        });
    }

    entries.sort_by(|a, b| match (&a.entry_type, &b.entry_type) {
        (t, o) if t == "dir" && o != "dir" => std::cmp::Ordering::Less,
        (t, o) if t != "dir" && o == "dir" => std::cmp::Ordering::Greater,
        _ => a.path.cmp(&b.path),
    });

    (StatusCode::OK, Json(entries)).into_response()
}

/// Read file content at a given path in a repo's default branch.
pub async fn read_blob(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(params): Query<TreeQueryParams>,
) -> impl IntoResponse {
    let file_path = match params.path {
        Some(ref p) if !p.is_empty() => p.clone(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("path query parameter is required".into()).error_response()),
            )
                .into_response();
        }
    };

    let repo_path = state.git_service.repo_path(&owner, &name);

    if !repo_path.join("HEAD").exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response();
    }

    let repo = match gix::open(&repo_path) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let head_id = match repo.head_id() {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let commit_obj = match head_id.object() {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let commit = match commit_obj.try_into_commit() {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let tree_id = match commit.tree_id() {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let tree_obj = match tree_id.object() {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let tree = match tree_obj.try_into_tree() {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let entry = match tree.lookup_entry_by_path(&file_path) {
        Ok(Some(e)) => e,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound(format!("file not found: {file_path}")).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let mode = entry.mode();
    if mode.is_tree() {
        let entry_obj = match entry.object() {
            Ok(o) => o,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(git_err(e).error_response()),
                )
                    .into_response();
            }
        };
        let subtree = match entry_obj.try_into_tree() {
            Ok(t) => t,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(git_err(e).error_response()),
                )
                    .into_response();
            }
        };

        let mut entries = Vec::new();
        for sub_result in subtree.iter() {
            let sub_entry = match sub_result {
                Ok(e) => e,
                Err(_) => continue,
            };
            let sub_mode = sub_entry.mode();
            let (entry_type, size) = if sub_mode.is_tree() {
                ("dir".to_string(), 0u64)
            } else if sub_mode.is_blob() {
                let sz = sub_entry
                    .object()
                    .ok()
                    .and_then(|o| o.try_into_blob().ok())
                    .map(|b| b.data.len() as u64)
                    .unwrap_or(0);
                ("file".to_string(), sz)
            } else if sub_mode.is_link() {
                ("symlink".to_string(), 0u64)
            } else if sub_mode.is_commit() {
                ("submodule".to_string(), 0u64)
            } else {
                ("unknown".to_string(), 0u64)
            };

            entries.push(TreeEntry {
                path: sub_entry.filename().to_string(),
                entry_type,
                size,
                last_commit: None,
            });
        }

        entries.sort_by(|a, b| match (&a.entry_type, &b.entry_type) {
            (t, o) if t == "dir" && o != "dir" => std::cmp::Ordering::Less,
            (t, o) if t != "dir" && o == "dir" => std::cmp::Ordering::Greater,
            _ => a.path.cmp(&b.path),
        });

        return (StatusCode::OK, Json(entries)).into_response();
    }

    let blob_obj = match entry.object() {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let blob = match blob_obj.try_into_blob() {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let data = &blob.data;
    let (content, encoding) = match String::from_utf8(data.to_vec()) {
        Ok(s) => (s, "utf-8".to_string()),
        Err(_) => (base64_encode(data), "base64".to_string()),
    };

    let resp = BlobResponse {
        path: file_path,
        content,
        size: data.len() as u64,
        encoding,
    };

    (StatusCode::OK, Json(resp)).into_response()
}

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_entry_serialization() {
        let entry = TreeEntry {
            path: "src/main.rs".into(),
            entry_type: "file".into(),
            size: 1024,
            last_commit: Some("abc123".into()),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"entry_type\":\"file\""));
        assert!(json.contains("\"size\":1024"));
    }

    #[test]
    fn test_tree_entry_dir() {
        let entry = TreeEntry {
            path: "src".into(),
            entry_type: "dir".into(),
            size: 0,
            last_commit: None,
        };
        assert_eq!(entry.entry_type, "dir");
    }

    #[test]
    fn test_blob_response_serialization() {
        let resp = BlobResponse {
            path: "README.md".into(),
            content: "Hello World".into(),
            size: 11,
            encoding: "utf-8".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"encoding\":\"utf-8\""));
        assert!(json.contains("\"content\":\"Hello World\""));
    }

    #[test]
    fn test_tree_query_params_defaults() {
        let params = TreeQueryParams {
            path: None,
            ref_: None,
        };
        assert!(params.path.is_none());
        assert!(params.ref_.is_none());
    }

    #[test]
    fn test_tree_query_params_parse() {
        let json = r#"{"path":"src/main.rs"}"#;
        let params: TreeQueryParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.path.as_deref(), Some("src/main.rs"));
    }

    #[test]
    fn test_tree_sorting_dirs_first() {
        let mut entries = vec![
            TreeEntry {
                path: "a.rs".into(),
                entry_type: "file".into(),
                size: 1,
                last_commit: None,
            },
            TreeEntry {
                path: "src".into(),
                entry_type: "dir".into(),
                size: 0,
                last_commit: None,
            },
            TreeEntry {
                path: "b.rs".into(),
                entry_type: "file".into(),
                size: 2,
                last_commit: None,
            },
        ];
        entries.sort_by(|a, b| match (&a.entry_type, &b.entry_type) {
            (t, o) if t == "dir" && o != "dir" => std::cmp::Ordering::Less,
            (t, o) if t != "dir" && o == "dir" => std::cmp::Ordering::Greater,
            _ => a.path.cmp(&b.path),
        });
        assert_eq!(entries[0].path, "src");
        assert_eq!(entries[1].path, "a.rs");
        assert_eq!(entries[2].path, "b.rs");
    }

    #[test]
    fn test_base64_encode() {
        use base64::Engine;
        let encoded = base64_encode(b"hello");
        assert_eq!(
            encoded,
            base64::engine::general_purpose::STANDARD.encode(b"hello")
        );
    }

    #[test]
    fn test_code_browser_routes_type() {
        fn _assert_routes() -> Router<AppState> {
            code_browser_routes()
        }
    }

    #[test]
    fn test_tree_entry_unknown_type() {
        let entry = TreeEntry {
            path: "special".into(),
            entry_type: "unknown".into(),
            size: 0,
            last_commit: None,
        };
        assert_eq!(entry.entry_type, "unknown");
    }
}
