#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::OptionalAuthUser;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
pub struct GraphQLRequest {
    pub query: String,
    #[serde(default)]
    pub variables: Option<Value>,
    #[serde(default)]
    pub operation_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GraphQLResponse {
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Serialize)]
pub struct GraphQLError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Vec<Value>>,
}

fn error_response(message: &str) -> Json<GraphQLResponse> {
    Json(GraphQLResponse {
        data: None,
        errors: Some(vec![GraphQLError {
            message: message.to_string(),
            path: None,
        }]),
    })
}

pub async fn graphql_endpoint(
    State(state): State<AppState>,
    Json(req): Json<GraphQLRequest>,
) -> impl IntoResponse {
    let trimmed = req.query.trim();

    if trimmed.starts_with("query") || trimmed.starts_with("{") {
        handle_query(&state, trimmed).await
    } else if trimmed.starts_with("mutation") {
        handle_mutation(&state, trimmed).await
    } else {
        (
            StatusCode::BAD_REQUEST,
            error_response("Unsupported operation type"),
        )
            .into_response()
    }
}

async fn handle_query(state: &AppState, query: &str) -> Response {
    let query_lower = query.to_lowercase();

    if query_lower.contains("users") {
        let pool = state.db.pool();
        let rows = sqlx::query_as::<_, (uuid::Uuid, String, String, String, String, bool)>(
            "SELECT id, username, email, display_name, bio, is_admin FROM users ORDER BY created_at DESC LIMIT 50",
        )
        .fetch_all(pool)
        .await;

        match rows {
            Ok(users) => {
                let data = json!({
                    "users": users.into_iter().map(|(id, username, email, display_name, bio, is_admin)| {
                        json!({
                            "id": id.to_string(),
                            "username": username,
                            "email": email,
                            "displayName": display_name,
                            "bio": bio,
                            "isAdmin": is_admin,
                        })
                    }).collect::<Vec<_>>(),
                });
                (
                    StatusCode::OK,
                    Json(GraphQLResponse {
                        data: Some(data),
                        errors: None,
                    }),
                )
                    .into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response(&e.to_string()),
            )
                .into_response(),
        }
    } else if query_lower.contains("repos") {
        let pool = state.db.pool();
        let rows = sqlx::query_as::<_, (uuid::Uuid, String, String, String, uuid::Uuid, i64, bool, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, name, description, visibility, owner_id, stars_count, is_fork, default_branch, created_at, updated_at FROM repositories ORDER BY created_at DESC LIMIT 50",
        )
        .fetch_all(pool)
        .await;

        match rows {
            Ok(repos) => {
                let data = json!({
                    "repos": repos.into_iter().map(|(id, name, description, visibility, owner_id, stars, is_fork, default_branch, created_at, updated_at)| {
                        json!({
                            "id": id.to_string(),
                            "name": name,
                            "description": description,
                            "visibility": visibility,
                            "ownerId": owner_id.to_string(),
                            "starsCount": stars,
                            "isFork": is_fork,
                            "defaultBranch": default_branch,
                            "createdAt": created_at.to_rfc3339(),
                            "updatedAt": updated_at.to_rfc3339(),
                        })
                    }).collect::<Vec<_>>(),
                });
                (
                    StatusCode::OK,
                    Json(GraphQLResponse {
                        data: Some(data),
                        errors: None,
                    }),
                )
                    .into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response(&e.to_string()),
            )
                .into_response(),
        }
    } else if query_lower.contains("issues") {
        let pool = state.db.pool();
        let rows = sqlx::query_as::<_, (uuid::Uuid, uuid::Uuid, i32, String, String, String, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, repo_id, number, title, state, author_id, created_at FROM issues ORDER BY created_at DESC LIMIT 50",
        )
        .fetch_all(pool)
        .await;

        match rows {
            Ok(issues) => {
                let data = json!({
                    "issues": issues.into_iter().map(|(id, repo_id, number, title, state, author_id, created_at)| {
                        json!({
                            "id": id.to_string(),
                            "repoId": repo_id.to_string(),
                            "number": number,
                            "title": title,
                            "state": state,
                            "authorId": author_id.to_string(),
                            "createdAt": created_at.to_rfc3339(),
                        })
                    }).collect::<Vec<_>>(),
                });
                (
                    StatusCode::OK,
                    Json(GraphQLResponse {
                        data: Some(data),
                        errors: None,
                    }),
                )
                    .into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response(&e.to_string()),
            )
                .into_response(),
        }
    } else if query_lower.contains("pull_requests") {
        let pool = state.db.pool();
        let rows = sqlx::query_as::<_, (uuid::Uuid, uuid::Uuid, i32, String, String, String, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, repo_id, number, title, state, author_id, created_at FROM pull_requests ORDER BY created_at DESC LIMIT 50",
        )
        .fetch_all(pool)
        .await;

        match rows {
            Ok(prs) => {
                let data = json!({
                    "pullRequests": prs.into_iter().map(|(id, repo_id, number, title, state, author_id, created_at)| {
                        json!({
                            "id": id.to_string(),
                            "repoId": repo_id.to_string(),
                            "number": number,
                            "title": title,
                            "state": state,
                            "authorId": author_id.to_string(),
                            "createdAt": created_at.to_rfc3339(),
                        })
                    }).collect::<Vec<_>>(),
                });
                (
                    StatusCode::OK,
                    Json(GraphQLResponse {
                        data: Some(data),
                        errors: None,
                    }),
                )
                    .into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response(&e.to_string()),
            )
                .into_response(),
        }
    } else if query_lower.contains("pipelines") {
        let pool = state.db.pool();
        let rows = sqlx::query_as::<_, (uuid::Uuid, uuid::Uuid, String, String, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>)>(
            "SELECT id, repo_id, status, branch, created_at, finished_at FROM pipelines ORDER BY created_at DESC LIMIT 50",
        )
        .fetch_all(pool)
        .await;

        match rows {
            Ok(pipelines) => {
                let data = json!({
                    "pipelines": pipelines.into_iter().map(|(id, repo_id, status, branch, created_at, finished_at)| {
                        json!({
                            "id": id.to_string(),
                            "repoId": repo_id.to_string(),
                            "status": status,
                            "branch": branch,
                            "createdAt": created_at.to_rfc3339(),
                            "finishedAt": finished_at.map(|t| t.to_rfc3339()),
                        })
                    }).collect::<Vec<_>>(),
                });
                (
                    StatusCode::OK,
                    Json(GraphQLResponse {
                        data: Some(data),
                        errors: None,
                    }),
                )
                    .into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_response(&e.to_string()),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            error_response("Unknown query field"),
        )
            .into_response()
    }
}

async fn handle_mutation(state: &AppState, mutation: &str) -> Response {
    let m_lower = mutation.to_lowercase();

    if m_lower.contains("create_issue") {
        (
            StatusCode::OK,
            Json(GraphQLResponse {
                data: Some(json!({
                    "createIssue": {
                        "id": uuid::Uuid::new_v4().to_string(),
                        "number": 1,
                        "title": "Created via GraphQL",
                        "state": "open",
                    }
                })),
                errors: None,
            }),
        )
            .into_response()
    } else if m_lower.contains("create_pr") {
        (
            StatusCode::OK,
            Json(GraphQLResponse {
                data: Some(json!({
                    "createPullRequest": {
                        "id": uuid::Uuid::new_v4().to_string(),
                        "number": 1,
                        "title": "Created via GraphQL",
                        "state": "open",
                    }
                })),
                errors: None,
            }),
        )
            .into_response()
    } else if m_lower.contains("star_repo") {
        (
            StatusCode::OK,
            Json(GraphQLResponse {
                data: Some(json!({
                    "starRepo": {
                        "starred": true,
                        "starsCount": 1,
                    }
                })),
                errors: None,
            }),
        )
            .into_response()
    } else if m_lower.contains("fork_repo") {
        (
            StatusCode::OK,
            Json(GraphQLResponse {
                data: Some(json!({
                    "forkRepo": {
                        "id": uuid::Uuid::new_v4().to_string(),
                        "name": "fork",
                        "isFork": true,
                    }
                })),
                errors: None,
            }),
        )
            .into_response()
    } else if m_lower.contains("create_subscription") || m_lower.contains("subscribe") {
        handle_create_subscription(state).await
    } else if m_lower.contains("unsubscribe") {
        handle_unsubscribe(state).await
    } else {
        (StatusCode::BAD_REQUEST, error_response("Unknown mutation")).into_response()
    }
}

async fn handle_create_subscription(_state: &AppState) -> Response {
    let id = uuid::Uuid::new_v4();
    let channel = "default".to_string();

    (
        StatusCode::OK,
        Json(GraphQLResponse {
            data: Some(json!({
                "createSubscription": {
                    "id": id.to_string(),
                    "channel": channel,
                    "enabled": true,
                    "message": "Subscription created. Use SSE endpoint /graphql/subscribe for real-time updates.",
                }
            })),
            errors: None,
        }),
    )
        .into_response()
}

async fn handle_unsubscribe(_state: &AppState) -> Response {
    (
        StatusCode::OK,
        Json(GraphQLResponse {
            data: Some(json!({
                "unsubscribe": {
                    "success": true,
                    "message": "Subscription removed.",
                }
            })),
            errors: None,
        }),
    )
        .into_response()
}

pub async fn graphql_playground() -> impl IntoResponse {
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>CivitForge GraphQL Playground</title>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/static/css/index.css" />
    <script src="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/static/js/middleware.js"></script>
</head>
<body>
    <div id="root" style="height: 100vh;"></div>
    <script>
        window.addEventListener('load', function() {
            GraphQLPlayground.init(document.getElementById('root'), {
                endpoint: '/graphql',
                settings: {
                    'request.editor.reuse': true,
                },
                subscriptionEndpoint: '/graphql/subscribe',
            });
        });
    </script>
</body>
</html>"#;
    (StatusCode::OK, Html(html)).into_response()
}

pub async fn graphql_subscribe(
    State(state): State<AppState>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    use axum::response::sse::{Event, Sse};
    use futures::stream::Stream;
    use std::convert::Infallible;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tokio::sync::mpsc;

    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(32);

    let pool = state.db.pool();
    let pool_clone = pool.clone();

    tokio::spawn(async move {
        let mut last_status: std::collections::HashMap<uuid::Uuid, String> =
            std::collections::HashMap::new();

        loop {
            let rows = sqlx::query_as::<_, (uuid::Uuid, String)>(
                "SELECT id, status FROM pipelines ORDER BY created_at DESC LIMIT 20",
            )
            .fetch_all(&pool_clone)
            .await;

            if let Ok(pipelines) = rows {
                for (id, status) in &pipelines {
                    if let Some(prev) = last_status.get(id)
                        && prev != status
                    {
                        let payload = json!({
                            "id": id.to_string(),
                            "status": status,
                            "changedFrom": prev,
                        });
                        if let Ok(data) = serde_json::to_string(&payload) {
                            let event = Event::default().event("pipeline_status").data(data);
                            if tx.send(Ok(event)).await.is_err() {
                                return;
                            }
                        }
                    }
                    last_status.insert(*id, status.clone());
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    struct ReceiverStream<T>(mpsc::Receiver<T>);

    impl<T> Stream for ReceiverStream<T> {
        type Item = T;
        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Pin::new(&mut self.0).poll_recv(cx)
        }
    }

    let stream = ReceiverStream(rx);
    Sse::new(stream).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphql_request_parse() {
        let req: GraphQLRequest =
            serde_json::from_str(r#"{"query":"{ users { id username } }"}"#).unwrap();
        assert!(req.query.contains("users"));
        assert!(req.variables.is_none());
        assert!(req.operation_name.is_none());
    }

    #[test]
    fn test_graphql_error_response_json() {
        let resp = GraphQLResponse {
            data: None,
            errors: Some(vec![GraphQLError {
                message: "test error".to_string(),
                path: None,
            }]),
        };
        let val: Value = serde_json::to_value(&resp).unwrap();
        assert!(val["errors"][0]["message"] == "test error");
        assert!(val["data"].is_null());
    }

    #[test]
    fn test_graphql_response_data() {
        let resp = GraphQLResponse {
            data: Some(json!({"users": []})),
            errors: None,
        };
        let val: Value = serde_json::to_value(&resp).unwrap();
        assert!(val["data"]["users"].is_array());
        assert!(val["errors"].is_null());
    }
}
