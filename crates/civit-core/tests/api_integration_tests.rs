#![forbid(unsafe_code)]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use civit_core::api::create_router;
use civit_core::config::{AppConfig, SecurityConfig};
use http_body_util::BodyExt;
use sqlx::postgres::PgPool;
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_config() -> AppConfig {
    let storage = tempfile::tempdir().unwrap().keep();
    AppConfig {
        host: "127.0.0.1".into(),
        port: 8080,
        database_url: "postgres://localhost/test".into(),
        redis_url: "redis://localhost:6379".into(),
        jwt_secret: "test-secret-key-32bytes-minimums".into(),
        jwt_expiry_hours: 24,
        federation_enabled: false,
        federation_instance_id: "test".into(),
        federation_instance_domain: "localhost".into(),
        storage_path: storage.to_str().unwrap().to_string(),
        cors_allowed_origins: Vec::new(),
        rate_limit_max_requests: None,
        rate_limit_window_secs: None,
        security: SecurityConfig::default(),
        tls_cert_path: None,
        tls_key_path: None,
        ui_assets_path: "/tmp/nonexistent-ui".into(),
        debug_mode: false,
    }
}

async fn register_user(
    app: &axum::Router,
    username: &str,
    email: &str,
) -> (StatusCode, serde_json::Value) {
    let reg_body = serde_json::json!({
        "username": username,
        "email": email,
        "display_name": username,
        "password": "Test1234!"
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&reg_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    (status, json)
}

async fn register_and_login(
    app: &axum::Router,
    username: &str,
    email: &str,
) -> (String, serde_json::Value) {
    let reg_body = serde_json::json!({
        "username": username,
        "email": email,
        "display_name": username,
        "password": "Test1234!"
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&reg_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = json["token"].as_str().unwrap().to_string();
    (token, json)
}

async fn login(app: &axum::Router, username: &str, password: &str) -> (StatusCode, serde_json::Value) {
    let body = serde_json::json!({"username": username, "password": password});
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    (status, json)
}

fn auth_header(token: &str) -> String {
    format!("Bearer {token}")
}

async fn get_json(app: &axum::Router, uri: &str, token: Option<&str>) -> (StatusCode, serde_json::Value) {
    let mut builder = Request::builder().method("GET").uri(uri);
    if let Some(t) = token {
        builder = builder.header("authorization", auth_header(t));
    }
    let resp = app
        .clone()
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
    (status, json)
}

async fn post_json(app: &axum::Router, uri: &str, body: serde_json::Value, token: Option<&str>) -> (StatusCode, serde_json::Value) {
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(t) = token {
        builder = builder.header("authorization", auth_header(t));
    }
    let resp = app
        .clone()
        .oneshot(builder.body(Body::from(serde_json::to_string(&body).unwrap())).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
    (status, json)
}

async fn put_json(app: &axum::Router, uri: &str, body: serde_json::Value, token: Option<&str>) -> (StatusCode, serde_json::Value) {
    let mut builder = Request::builder()
        .method("PUT")
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(t) = token {
        builder = builder.header("authorization", auth_header(t));
    }
    let resp = app
        .clone()
        .oneshot(builder.body(Body::from(serde_json::to_string(&body).unwrap())).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
    (status, json)
}

async fn patch_json(app: &axum::Router, uri: &str, body: serde_json::Value, token: Option<&str>) -> (StatusCode, serde_json::Value) {
    let mut builder = Request::builder()
        .method("PATCH")
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(t) = token {
        builder = builder.header("authorization", auth_header(t));
    }
    let resp = app
        .clone()
        .oneshot(builder.body(Body::from(serde_json::to_string(&body).unwrap())).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
    (status, json)
}

async fn delete_request(app: &axum::Router, uri: &str, token: Option<&str>) -> StatusCode {
    let mut builder = Request::builder().method("DELETE").uri(uri);
    if let Some(t) = token {
        builder = builder.header("authorization", auth_header(t));
    }
    let resp = app
        .clone()
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap();
    resp.status()
}

async fn create_repo(app: &axum::Router, token: &str, name: &str, owner: &str) -> serde_json::Value {
    let body = serde_json::json!({
        "name": name,
        "owner": owner,
        "description": format!("Test repo {name}"),
        "visibility": "public"
    });
    let (_, json) = post_json(app, "/api/v1/repos", body, Some(token)).await;
    json
}

// ===========================================================================
// Auth tests
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_register_success(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (_, json) = register_and_login(&app, "alice", "alice@example.com").await;
    assert!(!json["token"].as_str().unwrap().is_empty());
    assert_eq!(json["user"]["username"], "alice");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_register_duplicate(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    register_user(&app, "alice", "alice@example.com").await;
    let (status, _) = register_user(&app, "alice", "alice2@example.com").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_login_success(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    register_and_login(&app, "bob", "bob@example.com").await;
    let (status, json) = login(&app, "bob", "Test1234!").await;
    assert_eq!(status, StatusCode::OK);
    assert!(!json["token"].as_str().unwrap().is_empty());
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_login_wrong_password(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    register_and_login(&app, "carol", "carol@example.com").await;
    let (status, _) = login(&app, "carol", "WrongPassword1!").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_login_locked_account(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    register_and_login(&app, "dave", "dave@example.com").await;
    for _ in 0..6 {
        login(&app, "dave", "WrongPassword1!").await;
    }
    let (status, _) = login(&app, "dave", "Test1234!").await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_me(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "eve", "eve@example.com").await;
    let (status, json) = get_json(&app, "/api/v1/auth/me", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["username"], "eve");
}

// ===========================================================================
// Repos tests
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_create_repo(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "owner", "owner@example.com").await;
    let json = create_repo(&app, &token, "myrepo", "owner").await;
    assert_eq!(json["name"], "myrepo");
    assert_eq!(json["visibility"], "public");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_list_repos(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "lister", "lister@example.com").await;
    create_repo(&app, &token, "repo1", "lister").await;
    create_repo(&app, &token, "repo2", "lister").await;
    let (status, json) = get_json(&app, "/api/v1/repos", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let data = json["data"].as_array().unwrap();
    assert!(data.len() >= 2);
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_get_repo(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "fetcher", "fetcher@example.com").await;
    create_repo(&app, &token, "target", "fetcher").await;
    let (status, json) = get_json(&app, "/api/v1/repos/fetcher/target", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["name"], "target");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_update_repo(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "updater", "updater@example.com").await;
    create_repo(&app, &token, "mutable", "updater").await;
    let body = serde_json::json!({"description": "Updated description"});
    let (status, json) = patch_json(&app, "/api/v1/repos/updater/mutable", body, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["description"], "Updated description");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_delete_repo(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "deleter", "deleter@example.com").await;
    create_repo(&app, &token, "todelete", "deleter").await;
    let status = delete_request(&app, "/api/v1/repos/deleter/todelete", Some(&token)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = get_json(&app, "/api/v1/repos/deleter/todelete", Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_star_toggle(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "staruser", "star@example.com").await;
    create_repo(&app, &token, "starrepo", "staruser").await;
    let (status, json) = post_json(&app, "/api/v1/repos/staruser/starrepo/star", serde_json::json!({}), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(json["starred"].as_bool().unwrap());
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_watch_toggle(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "watcher", "watcher@example.com").await;
    create_repo(&app, &token, "watchrepo", "watcher").await;
    let (status, json) = post_json(&app, "/api/v1/repos/watcher/watchrepo/watch", serde_json::json!({}), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(json["watched"].as_bool().unwrap());
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_set_topics(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "topicuser", "topic@example.com").await;
    create_repo(&app, &token, "topicrepo", "topicuser").await;
    let body = serde_json::json!({"topics": ["rust", "web"]});
    let (status, json) = put_json(&app, "/api/v1/repos/topicuser/topicrepo/topics", body, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let topics = json["topics"].as_array().unwrap();
    assert!(topics.contains(&serde_json::json!("rust")));
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_archive_toggle(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "archuser", "arch@example.com").await;
    create_repo(&app, &token, "archrepo", "archuser").await;
    let body = serde_json::json!({"archived": true});
    let (status, _json) = post_json(&app, "/api/v1/repos/archuser/archrepo/archive-toggle", body, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
}

// ===========================================================================
// Issues tests
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_create_issue(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "issuer", "issuer@example.com").await;
    create_repo(&app, &token, "issuerepo", "issuer").await;
    let body = serde_json::json!({"title": "Bug report", "description": "Something is broken"});
    let (status, json) = post_json(&app, "/api/v1/repos/issuer/issuerepo/issues", body, Some(&token)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["title"], "Bug report");
    assert_eq!(json["status"], "open");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_list_issues(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "listissuer", "listissuer@example.com").await;
    create_repo(&app, &token, "listissuerepo", "listissuer").await;
    let body = serde_json::json!({"title": "Issue 1"});
    post_json(&app, "/api/v1/repos/listissuer/listissuerepo/issues", body, Some(&token)).await;
    let body = serde_json::json!({"title": "Issue 2"});
    post_json(&app, "/api/v1/repos/listissuer/listissuerepo/issues", body, Some(&token)).await;
    let (status, json) = get_json(&app, "/api/v1/repos/listissuer/listissuerepo/issues", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let data = json["data"].as_array().unwrap();
    assert!(data.len() >= 2);
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_get_issue(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "getissuer", "getissuer@example.com").await;
    create_repo(&app, &token, "getissuerepo", "getissuer").await;
    let body = serde_json::json!({"title": "My issue"});
    let (_, created) = post_json(&app, "/api/v1/repos/getissuer/getissuerepo/issues", body, Some(&token)).await;
    let number = created["number"].as_i64().unwrap();
    let (status, json) = get_json(&app, &format!("/api/v1/repos/getissuer/getissuerepo/issues/{number}"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["title"], "My issue");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_update_issue(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "updissuer", "updissuer@example.com").await;
    create_repo(&app, &token, "updissuerepo", "updissuer").await;
    let body = serde_json::json!({"title": "Original"});
    let (_, created) = post_json(&app, "/api/v1/repos/updissuer/updissuerepo/issues", body, Some(&token)).await;
    let number = created["number"].as_i64().unwrap();
    let body = serde_json::json!({"title": "Updated title", "state": "in_progress"});
    let (status, json) = patch_json(&app, &format!("/api/v1/repos/updissuer/updissuerepo/issues/{number}"), body, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["title"], "Updated title");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_add_comment(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "commenter", "commenter@example.com").await;
    create_repo(&app, &token, "commentrepo", "commenter").await;
    let body = serde_json::json!({"title": "Issue"});
    let (_, created) = post_json(&app, "/api/v1/repos/commenter/commentrepo/issues", body, Some(&token)).await;
    let number = created["number"].as_i64().unwrap();
    let body = serde_json::json!({"body": "Nice work!"});
    let (status, json) = post_json(&app, &format!("/api/v1/repos/commenter/commentrepo/issues/{number}/comments"), body, Some(&token)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["body"], "Nice work!");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_toggle_pin(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "pinuser", "pin@example.com").await;
    create_repo(&app, &token, "pinrepo", "pinuser").await;
    let body = serde_json::json!({"title": "Pinnable"});
    let (_, created) = post_json(&app, "/api/v1/repos/pinuser/pinrepo/issues", body, Some(&token)).await;
    let number = created["number"].as_i64().unwrap();
    let (status, json) = post_json(&app, &format!("/api/v1/repos/pinuser/pinrepo/issues/{number}/pin"), serde_json::json!({}), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(json["is_pinned"].as_bool().unwrap());
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_toggle_lock(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "lockuser", "lock@example.com").await;
    create_repo(&app, &token, "lockrepo", "lockuser").await;
    let body = serde_json::json!({"title": "Lockable"});
    let (_, created) = post_json(&app, "/api/v1/repos/lockuser/lockrepo/issues", body, Some(&token)).await;
    let number = created["number"].as_i64().unwrap();
    let (status, json) = post_json(&app, &format!("/api/v1/repos/lockuser/lockrepo/issues/{number}/lock"), serde_json::json!({}), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(json["is_locked"].as_bool().unwrap());
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_log_time(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "timeuser", "time@example.com").await;
    create_repo(&app, &token, "timerepo", "timeuser").await;
    let body = serde_json::json!({"title": "Timed"});
    let (_, created) = post_json(&app, "/api/v1/repos/timeuser/timerepo/issues", body, Some(&token)).await;
    let number = created["number"].as_i64().unwrap();
    let body = serde_json::json!({"hours": 2.5, "description": "Debugging"});
    let (status, json) = post_json(&app, &format!("/api/v1/repos/timeuser/timerepo/issues/{number}/time"), body, Some(&token)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["hours"], 2.5);
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_get_time(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "gettime", "gettime@example.com").await;
    create_repo(&app, &token, "gettimerepo", "gettime").await;
    let body = serde_json::json!({"title": "Timed issue"});
    let (_, created) = post_json(&app, "/api/v1/repos/gettime/gettimerepo/issues", body, Some(&token)).await;
    let number = created["number"].as_i64().unwrap();
    let body = serde_json::json!({"hours": 3.0});
    post_json(&app, &format!("/api/v1/repos/gettime/gettimerepo/issues/{number}/time"), body, Some(&token)).await;
    let (status, json) = get_json(&app, &format!("/api/v1/repos/gettime/gettimerepo/issues/{number}/time"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["total_hours"], 3.0);
}

// ===========================================================================
// Pull Requests tests
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_create_pull_request(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "prcreator", "prcreator@example.com").await;
    create_repo(&app, &token, "prrepo", "prcreator").await;
    let body = serde_json::json!({
        "title": "Add feature",
        "body": "Implements new feature",
        "source_branch": "feature-x",
        "target_branch": "main"
    });
    let (status, json) = post_json(&app, "/api/v1/repos/prcreator/prrepo/pulls", body, Some(&token)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["title"], "Add feature");
    assert_eq!(json["status"], "open");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_list_pull_requests(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "prlister", "prlister@example.com").await;
    create_repo(&app, &token, "prlistrepo", "prlister").await;
    let body = serde_json::json!({
        "title": "PR 1",
        "source_branch": "b1",
        "target_branch": "main"
    });
    post_json(&app, "/api/v1/repos/prlister/prlistrepo/pulls", body, Some(&token)).await;
    let body = serde_json::json!({
        "title": "PR 2",
        "source_branch": "b2",
        "target_branch": "main"
    });
    post_json(&app, "/api/v1/repos/prlister/prlistrepo/pulls", body, Some(&token)).await;
    let (status, json) = get_json(&app, "/api/v1/repos/prlister/prlistrepo/pulls", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let items = json["items"].as_array().unwrap();
    assert!(items.len() >= 2);
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_get_pull_request(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "prgetter", "prgetter@example.com").await;
    create_repo(&app, &token, "prgetrepo", "prgetter").await;
    let body = serde_json::json!({
        "title": "My PR",
        "source_branch": "feature",
        "target_branch": "main"
    });
    let (_, created) = post_json(&app, "/api/v1/repos/prgetter/prgetrepo/pulls", body, Some(&token)).await;
    let number = created["number"].as_i64().unwrap();
    let (status, json) = get_json(&app, &format!("/api/v1/repos/prgetter/prgetrepo/pulls/{number}"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["title"], "My PR");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_pr_patch(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "prpatcher", "prpatcher@example.com").await;
    create_repo(&app, &token, "prpatchrepo", "prpatcher").await;
    let body = serde_json::json!({
        "title": "Patch PR",
        "source_branch": "feat",
        "target_branch": "main"
    });
    let (_, created) = post_json(&app, "/api/v1/repos/prpatcher/prpatchrepo/pulls", body, Some(&token)).await;
    let number = created["number"].as_i64().unwrap();
    let builder = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/repos/prpatcher/prpatchrepo/pulls/{number}/patch"))
        .header("authorization", auth_header(&token));
    let resp = app
        .clone()
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ===========================================================================
// Labels tests
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_create_label(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "labeluser", "label@example.com").await;
    create_repo(&app, &token, "labelrepo", "labeluser").await;
    let body = serde_json::json!({"name": "bug", "color": "#ff0000"});
    let (status, json) = post_json(&app, "/api/v1/repos/labeluser/labelrepo/labels", body, Some(&token)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["name"], "bug");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_list_labels(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "labellister", "labellister@example.com").await;
    create_repo(&app, &token, "labellistrepo", "labellister").await;
    let body = serde_json::json!({"name": "feature"});
    post_json(&app, "/api/v1/repos/labellister/labellistrepo/labels", body, Some(&token)).await;
    let (status, json) = get_json(&app, "/api/v1/repos/labellister/labellistrepo/labels", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let arr = json.as_array().unwrap();
    assert!(arr.len() >= 1);
}

// ===========================================================================
// Milestones tests
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_create_milestone(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "mileuser", "mile@example.com").await;
    create_repo(&app, &token, "milerepo", "mileuser").await;
    let body = serde_json::json!({"title": "v1.0 Release"});
    let (status, json) = post_json(&app, "/api/v1/repos/mileuser/milerepo/milestones", body, Some(&token)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["title"], "v1.0 Release");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_list_milestones(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "milelister", "milelister@example.com").await;
    create_repo(&app, &token, "milelistrepo", "milelister").await;
    let body = serde_json::json!({"title": "v1.0"});
    post_json(&app, "/api/v1/repos/milelister/milelistrepo/milestones", body, Some(&token)).await;
    let (status, json) = get_json(&app, "/api/v1/repos/milelister/milelistrepo/milestones", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let arr = json.as_array().unwrap();
    assert!(arr.len() >= 1);
}

// ===========================================================================
// Wiki tests
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_create_wiki(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "wikiuser", "wiki@example.com").await;
    create_repo(&app, &token, "wikirepo", "wikiuser").await;
    let body = serde_json::json!({
        "slug": "home",
        "title": "Home Page",
        "content": "# Welcome\n\nThis is the wiki."
    });
    let (status, json) = post_json(&app, "/api/v1/repos/wikiuser/wikirepo/wiki", body, Some(&token)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["slug"], "home");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_list_wiki(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "wikilister", "wikilister@example.com").await;
    create_repo(&app, &token, "wikilistrepo", "wikilister").await;
    let body = serde_json::json!({
        "slug": "getting-started",
        "title": "Getting Started",
        "content": "# Getting Started\n\nFollow these steps."
    });
    post_json(&app, "/api/v1/repos/wikilister/wikilistrepo/wiki", body, Some(&token)).await;
    let (status, json) = get_json(&app, "/api/v1/repos/wikilister/wikilistrepo/wiki", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let arr = json.as_array().unwrap();
    assert!(arr.len() >= 1);
}

// ===========================================================================
// Webhooks tests
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_create_webhook(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "whuser", "wh@example.com").await;
    create_repo(&app, &token, "whrepo", "whuser").await;
    let body = serde_json::json!({
        "url": "https://example.com/hook",
        "events": ["push"]
    });
    let (status, json) = post_json(&app, "/api/v1/repos/whuser/whrepo/webhooks", body, Some(&token)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["url"], "https://example.com/hook");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_list_webhooks(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "whlister", "whlister@example.com").await;
    create_repo(&app, &token, "whlistrepo", "whlister").await;
    let body = serde_json::json!({"url": "https://example.com/hook"});
    post_json(&app, "/api/v1/repos/whlister/whlistrepo/webhooks", body, Some(&token)).await;
    let (status, json) = get_json(&app, "/api/v1/repos/whlister/whlistrepo/webhooks", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let data = json["data"].as_array().unwrap();
    assert!(data.len() >= 1);
}

// ===========================================================================
// Releases tests
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_create_release(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "reluser", "rel@example.com").await;
    create_repo(&app, &token, "relrepo", "reluser").await;
    let body = serde_json::json!({
        "tag_name": "v1.0.0",
        "name": "Release 1.0",
        "body": "First release"
    });
    let (status, json) = post_json(&app, "/api/v1/repos/reluser/relrepo/releases", body, Some(&token)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["tag_name"], "v1.0.0");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_list_releases(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "rellister", "rellister@example.com").await;
    create_repo(&app, &token, "rellistrepo", "rellister").await;
    let body = serde_json::json!({"tag_name": "v1.0", "name": "v1.0"});
    post_json(&app, "/api/v1/repos/rellister/rellistrepo/releases", body, Some(&token)).await;
    let (status, json) = get_json(&app, "/api/v1/repos/rellister/rellistrepo/releases", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let data = json["data"].as_array().unwrap();
    assert!(data.len() >= 1);
}

// ===========================================================================
// Branch Protection tests
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_set_branch_protection(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "bpuser", "bp@example.com").await;
    create_repo(&app, &token, "bprepo", "bpuser").await;
    let body = serde_json::json!({
        "branch_pattern": "main",
        "require_pull_request": true,
        "required_approving_reviews": 2,
        "enforce_admins": true
    });
    let (status, json) = put_json(&app, "/api/v1/repos/bpuser/bprepo/branch-protection", body, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["branch_pattern"], "main");
    assert!(json["require_pull_request"].as_bool().unwrap());
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_get_branch_protection(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "bpgetter", "bpgetter@example.com").await;
    create_repo(&app, &token, "bpgetrepo", "bpgetter").await;
    let body = serde_json::json!({"branch_pattern": "main", "require_pull_request": true});
    put_json(&app, "/api/v1/repos/bpgetter/bpgetrepo/branch-protection", body, Some(&token)).await;
    let (status, json) = get_json(&app, "/api/v1/repos/bpgetter/bpgetrepo/branch-protection", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let arr = json.as_array().unwrap();
    assert!(!arr.is_empty());
}

// ===========================================================================
// Search tests
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_global_search(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "searcher", "searcher@example.com").await;
    let (status, _) = get_json(&app, "/api/v1/search?q=test", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
}

// ===========================================================================
// Environments tests
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_create_environment(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "envuser", "env@example.com").await;
    create_repo(&app, &token, "envrepo", "envuser").await;
    let body = serde_json::json!({"name": "production"});
    let (status, json) = post_json(&app, "/api/v1/repos/envuser/envrepo/environments", body, Some(&token)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["name"], "production");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_list_environments(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "envlister", "envlister@example.com").await;
    create_repo(&app, &token, "envlistrepo", "envlister").await;
    let body = serde_json::json!({"name": "staging"});
    post_json(&app, "/api/v1/repos/envlister/envlistrepo/environments", body, Some(&token)).await;
    let (status, json) = get_json(&app, "/api/v1/repos/envlister/envlistrepo/environments", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let arr = json.as_array().unwrap();
    assert!(arr.len() >= 1);
}

// ===========================================================================
// Deployments tests
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_create_deployment(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "depuser", "dep@example.com").await;
    create_repo(&app, &token, "deprepo", "depuser").await;
    let body = serde_json::json!({"sha": "abc123def456"});
    let (status, json) = post_json(&app, "/api/v1/repos/depuser/deprepo/deployments", body, Some(&token)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["sha"], "abc123def456");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_list_deployments(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "deplister", "deplister@example.com").await;
    create_repo(&app, &token, "deplistrepo", "deplister").await;
    let body = serde_json::json!({"sha": "sha1"});
    post_json(&app, "/api/v1/repos/deplister/deplistrepo/deployments", body, Some(&token)).await;
    let (status, json) = get_json(&app, "/api/v1/repos/deplister/deplistrepo/deployments", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let data = json["data"].as_array().unwrap();
    assert!(data.len() >= 1);
}

// ===========================================================================
// Merge Queue tests
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_merge_queue_add_and_list(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "mquser", "mq@example.com").await;
    create_repo(&app, &token, "mqrepo", "mquser").await;
    let body = serde_json::json!({
        "title": "PR for queue",
        "source_branch": "q-branch",
        "target_branch": "main"
    });
    let (_, created) = post_json(&app, "/api/v1/repos/mquser/mqrepo/pulls", body, Some(&token)).await;
    let pr_number = created["number"].as_i64().unwrap();
    let body = serde_json::json!({"pr_number": pr_number});
    let (status, _) = post_json(&app, "/api/v1/repos/mquser/mqrepo/merge-queue", body, Some(&token)).await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, json) = get_json(&app, "/api/v1/repos/mquser/mqrepo/merge-queue", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let items = json["items"].as_array().unwrap();
    assert!(!items.is_empty());
}

// ===========================================================================
// Users tests
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_list_users(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "ulister", "ulister@example.com").await;
    let (status, json) = get_json(&app, "/api/v1/users", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let data = json["data"].as_array().unwrap();
    assert!(!data.is_empty());
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_get_user(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, user_json) = register_and_login(&app, "ugetter", "ugetter@example.com").await;
    let user_id = user_json["user"]["id"].as_str().unwrap();
    let (status, json) = get_json(&app, &format!("/api/v1/users/{user_id}"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["username"], "ugetter");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_update_profile(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "profuser", "prof@example.com").await;
    let body = serde_json::json!({
        "display_name": "Professor",
        "bio": "I teach Rust",
        "location": "Remote"
    });
    let (status, json) = patch_json(&app, "/api/v1/user/profile", body, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["display_name"], "Professor");
    assert_eq!(json["bio"], "I teach Rust");
}

// ===========================================================================
// Notifications tests
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_list_notifications(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "notifuser", "notif@example.com").await;
    let (status, _) = get_json(&app, "/api/v1/notifications", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
}

// ===========================================================================
// Activity tests
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_list_activity(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "actuser", "act@example.com").await;
    let (status, _) = get_json(&app, "/api/v1/activity", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
}

// ===========================================================================
// Health check
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_health(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let resp = app
        .oneshot(Request::builder().uri("/healthz").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ===========================================================================
// OIDC token exchange
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_oidc_exchange(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "oidcuser", "oidc@example.com").await;
    let body = serde_json::json!({
        "provider": "github",
        "id_token": "eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ1c2VyMTIzIn0.signature"
    });
    let (status, json) = post_json(&app, "/api/v1/auth/oidc/exchange", body, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["provider"], "github");
    assert!(json["linked"].as_bool().unwrap());
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_oidc_exchange_missing_provider(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "oidcuser2", "oidc2@example.com").await;
    let body = serde_json::json!({"id_token": "some.token.here"});
    let (status, _) = post_json(&app, "/api/v1/auth/oidc/exchange", body, Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_oidc_exchange_no_auth(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let body = serde_json::json!({"provider": "github", "id_token": "abc"});
    let (status, _) = post_json(&app, "/api/v1/auth/oidc/exchange", body, None).await;
    assert!(status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN);
}

// ===========================================================================
// Site settings CRUD
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_get_site_settings_default(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (status, json) = get_json(&app, "/api/v1/admin/settings", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["site_name"], "CivitForge");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_update_site_settings(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "admin", "admin@example.com").await;
    let body = serde_json::json!({
        "site_name": "My Custom Forge",
        "site_description": "A private forge",
        "footer_text": "(c) 2025"
    });
    let (status, json) = put_json(&app, "/api/v1/admin/settings", body, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["site_name"], "My Custom Forge");
    assert_eq!(json["footer_text"], "(c) 2025");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_get_site_settings_after_update(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "admin2", "admin2@example.com").await;
    let body = serde_json::json!({"site_name": "Updated Name"});
    put_json(&app, "/api/v1/admin/settings", body, Some(&token)).await;
    let (status, json) = get_json(&app, "/api/v1/admin/settings", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["site_name"], "Updated Name");
}

// ===========================================================================
// Issue analytics
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_issue_analytics(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "analytics", "analytics@example.com").await;
    create_repo(&app, &token, "analyticsrepo", "analytics").await;
    let body = serde_json::json!({"title": "Issue for analytics"});
    post_json(&app, "/api/v1/repos/analytics/analyticsrepo/issues", body, Some(&token)).await;
    let (status, json) = get_json(&app, "/api/v1/repos/analytics/analyticsrepo/issues/analytics", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(json["total"].as_i64().unwrap() >= 1);
    assert!(json["open_count"].as_i64().unwrap() >= 1);
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_issue_analytics_repo_not_found(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "anuser", "anuser@example.com").await;
    let (status, _) = get_json(&app, "/api/v1/repos/anuser/nonexistent/issues/analytics", Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ===========================================================================
// Merge queue remove
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_merge_queue_remove(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "mqremove", "mqremove@example.com").await;
    create_repo(&app, &token, "mqremoverepo", "mqremove").await;
    let body = serde_json::json!({
        "title": "PR for removal",
        "source_branch": "rm-branch",
        "target_branch": "main"
    });
    let (_, created) = post_json(&app, "/api/v1/repos/mqremove/mqremoverepo/pulls", body, Some(&token)).await;
    let pr_number = created["number"].as_i64().unwrap();
    let body = serde_json::json!({"pr_number": pr_number});
    let (_, queue_resp) = post_json(&app, "/api/v1/repos/mqremove/mqremoverepo/merge-queue", body, Some(&token)).await;
    let entry_id = queue_resp["id"].as_str().unwrap();
    let status = delete_request(&app, &format!("/api/v1/repos/mqremove/mqremoverepo/merge-queue/{entry_id}"), Some(&token)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

// ===========================================================================
// Environments update and delete
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_update_environment(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "envupdate", "envupdate@example.com").await;
    create_repo(&app, &token, "envupdaterepo", "envupdate").await;
    let body = serde_json::json!({"name": "staging"});
    let (_, created) = post_json(&app, "/api/v1/repos/envupdate/envupdaterepo/environments", body, Some(&token)).await;
    let env_id = created["id"].as_str().unwrap();
    let body = serde_json::json!({"name": "staging-updated"});
    let (status, json) = put_json(&app, &format!("/api/v1/repos/envupdate/envupdaterepo/environments/{env_id}"), body, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["name"], "staging-updated");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_delete_environment(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "envdel", "envdel@example.com").await;
    create_repo(&app, &token, "envdelrepo", "envdel").await;
    let body = serde_json::json!({"name": "to-delete"});
    let (_, created) = post_json(&app, "/api/v1/repos/envdel/envdelrepo/environments", body, Some(&token)).await;
    let env_id = created["id"].as_str().unwrap();
    let status = delete_request(&app, &format!("/api/v1/repos/envdel/envdelrepo/environments/{env_id}"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_delete_environment_not_found(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "envdel2", "envdel2@example.com").await;
    create_repo(&app, &token, "envdel2repo", "envdel2").await;
    let fake_id = uuid::Uuid::new_v4();
    let status = delete_request(&app, &format!("/api/v1/repos/envdel2/envdel2repo/environments/{fake_id}"), Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ===========================================================================
// Deployments update status
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_update_deployment_status(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "depupdate", "depupdate@example.com").await;
    create_repo(&app, &token, "depupdaterepo", "depupdate").await;
    let body = serde_json::json!({"sha": "sha_for_update"});
    let (_, created) = post_json(&app, "/api/v1/repos/depupdate/depupdaterepo/deployments", body, Some(&token)).await;
    let dep_id = created["id"].as_str().unwrap();
    let body = serde_json::json!({"status": "success"});
    let (status, json) = patch_json(&app, &format!("/api/v1/repos/depupdate/depupdaterepo/deployments/{dep_id}/status"), body, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "success");
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_update_deployment_status_invalid(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "depinv", "depinv@example.com").await;
    create_repo(&app, &token, "depinvrepo", "depinv").await;
    let body = serde_json::json!({"sha": "sha_invalid"});
    let (_, created) = post_json(&app, "/api/v1/repos/depinv/depinvrepo/deployments", body, Some(&token)).await;
    let dep_id = created["id"].as_str().unwrap();
    let body = serde_json::json!({"status": "invalid_status"});
    let (status, _) = patch_json(&app, &format!("/api/v1/repos/depinv/depinvrepo/deployments/{dep_id}/status"), body, Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_update_deployment_status_not_found(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "depnf", "depnf@example.com").await;
    create_repo(&app, &token, "depnfrepo", "depnf").await;
    let fake_id = uuid::Uuid::new_v4();
    let body = serde_json::json!({"status": "success"});
    let (status, _) = patch_json(&app, &format!("/api/v1/repos/depnf/depnfrepo/deployments/{fake_id}/status"), body, Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ===========================================================================
// Branch protection toggle (set then update)
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_branch_protection_update(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "bptoggle", "bptoggle@example.com").await;
    create_repo(&app, &token, "bptogglerepo", "bptoggle").await;
    let body = serde_json::json!({
        "branch_pattern": "main",
        "require_pull_request": true,
        "required_approving_reviews": 1
    });
    put_json(&app, "/api/v1/repos/bptoggle/bptogglerepo/branch-protection", body, Some(&token)).await;
    let body = serde_json::json!({
        "branch_pattern": "main",
        "require_pull_request": false,
        "required_approving_reviews": 0
    });
    let (status, json) = put_json(&app, "/api/v1/repos/bptoggle/bptogglerepo/branch-protection", body, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(!json["require_pull_request"].as_bool().unwrap());
}

// ===========================================================================
// Code search
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_global_code_search_empty(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "codesearch", "codesearch@example.com").await;
    let (status, _) = get_json(&app, "/api/v1/search/code?q=test", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_global_code_search_missing_q(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "csnoq", "csnoq@example.com").await;
    let (status, _) = get_json(&app, "/api/v1/search/code", Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_repo_search_languages(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "languser", "lang@example.com").await;
    create_repo(&app, &token, "langrepo", "languser").await;
    let (status, _) = get_json(&app, "/api/v1/repos/languser/langrepo/search/languages", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
}

// ===========================================================================
// Notifications read/unread
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_notifications_unread_count(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "notifunread", "notifunread@example.com").await;
    let (status, json) = get_json(&app, "/api/v1/notifications/unread-count", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 0);
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_notifications_mark_read_not_found(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "notifmr", "notifmr@example.com").await;
    let fake_id = uuid::Uuid::new_v4();
    let (status, _) = patch_json(&app, &format!("/api/v1/notifications/{fake_id}/read"), serde_json::json!({}), Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_notifications_list_with_read_filter(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "notifread", "notifread@example.com").await;
    let (status, _) = get_json(&app, "/api/v1/notifications?read=true", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
}

// ===========================================================================
// Activity feed
// ===========================================================================

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_activity_with_pagination(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (token, _) = register_and_login(&app, "actpag", "actpag@example.com").await;
    let (status, json) = get_json(&app, "/api/v1/activity?per_page=5&page=1", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let pagination = &json["pagination"];
    assert!(pagination.get("page").is_some());
    assert!(pagination.get("per_page").is_some());
}

#[sqlx::test(migrations = "../civit-db/src/migrations")]
async fn test_activity_unauthenticated(pool: PgPool) {
    let config = test_config();
    let app = create_router(config, pool).unwrap();
    let (status, _) = get_json(&app, "/api/v1/activity", None).await;
    assert_eq!(status, StatusCode::OK);
}
