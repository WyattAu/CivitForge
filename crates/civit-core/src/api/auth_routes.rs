#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use crate::api::users::UserResponse;
use crate::error::CoreError;
use crate::ldap::LdapAuth;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Redirect},
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub email: String,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyEmailResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct LdapSyncRequest {
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct LdapSyncResponse {
    pub status: String,
    pub groups: Vec<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub user_id: String,
    pub username: String,
    pub role: String,
    pub org_id: Option<String>,
}

// ── OAuth2/PKCE Types ──

#[derive(Debug, Deserialize)]
pub struct OAuthAuthorizeParams {
    pub client_id: String,
    pub redirect_uri: String,
    pub response_type: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub state: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthTokenRequest {
    pub grant_type: String,
    pub code: String,
    pub redirect_uri: String,
    pub code_verifier: String,
    pub client_id: String,
}

#[derive(Debug, Deserialize)]
pub struct OAuthRefreshRequest {
    pub grant_type: String,
    pub refresh_token: String,
    pub client_id: String,
}

#[derive(Debug, Deserialize)]
pub struct OAuthRegisterClientRequest {
    pub name: String,
    pub redirect_uris: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct OAuthRegisterClientResponse {
    pub client_id: String,
    pub client_secret: String,
}

/// Hash a token for storage (SHA-256)
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generate a random string of specified length
fn generate_random_string(length: usize) -> String {
    let mut random_bytes = vec![0u8; length];
    rand::fill(&mut random_bytes);
    // Convert to alphanumeric string
    random_bytes
        .iter()
        .map(|b| {
            let idx = (b % 62) as usize;
            const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            CHARS[idx] as char
        })
        .collect()
}

impl From<AuthUser> for MeResponse {
    fn from(u: AuthUser) -> Self {
        Self {
            user_id: u.user_id,
            username: u.username,
            role: u.role.as_str().to_string(),
            org_id: u.org_id,
        }
    }
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    match do_register(&state, req).await {
        Ok(resp) => (StatusCode::CREATED, Json(resp)).into_response(),
        Err(e) => (e.status_code(), Json(e.error_response())).into_response(),
    }
}

async fn do_register(
    state: &AppState,
    req: RegisterRequest,
) -> crate::error::Result<LoginResponse> {
    if req.username.is_empty() {
        return Err(CoreError::Auth("Username is required".into()));
    }
    if req.email.is_empty() {
        return Err(CoreError::Auth("Email is required".into()));
    }
    let violations =
        crate::api::password::validate_password_policy(&req.password, &state.config.security);
    if !violations.is_empty() {
        return Err(CoreError::Auth(violations.join("; ")));
    }

    // Check if username or email already exists
    if state.db.get_user_by_username(&req.username).await.is_ok() {
        return Err(CoreError::Auth("Username already taken".into()));
    }
    if state.db.get_user_by_email(&req.email).await.is_ok() {
        return Err(CoreError::Auth("Email already registered".into()));
    }

    let password_hash = civit_auth::password::hash_password(&req.password)?;

    let user = state
        .db
        .create_user(
            &req.username,
            &req.email,
            if req.display_name.is_empty() {
                &req.username
            } else {
                &req.display_name
            },
            "member",
            &password_hash,
        )
        .await?;

    // Generate and store a 6-digit verification code
    let verification_code = generate_verification_code();
    let expires_at = Utc::now() + Duration::minutes(15);
    if let Err(e) = state
        .db
        .store_verification_code(user.id, &req.email, &verification_code, expires_at)
        .await
    {
        tracing::warn!("Failed to store verification code: {e}");
    }

    if state.config.debug_mode {
        tracing::info!(
            email = %req.email,
            code = %verification_code,
            "Email verification code (dev mode)"
        );
    } else {
        tracing::info!(
            email = %req.email,
            "Email verification code generated (email sending not yet implemented)"
        );
    }

    let token =
        state
            .jwt_service
            .generate_token(&user.id.to_string(), &user.username, &user.role, None)?;

    Ok(LoginResponse {
        token,
        user: UserResponse::from(user),
    })
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    match do_login(&state, req).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => (e.status_code(), Json(e.error_response())).into_response(),
    }
}

async fn do_login(state: &AppState, req: LoginRequest) -> crate::error::Result<LoginResponse> {
    if req.username.is_empty() {
        return Err(CoreError::Auth("Username is required".into()));
    }
    if req.password.is_empty() {
        return Err(CoreError::Auth("Password is required".into()));
    }

    // Check lockout: count recent failures
    let sec = &state.config.security;
    let recent_failures = state
        .db
        .count_recent_failed_logins(&req.username, sec.login_lockout_secs)
        .await
        .unwrap_or(0);

    if recent_failures >= sec.login_max_attempts as i64 {
        return Err(CoreError::TooManyRequests(format!(
            "Account temporarily locked due to too many failed login attempts. Try again in {} seconds.",
            sec.login_lockout_secs
        )));
    }

    // Try LDAP authentication first if enabled
    if sec.ldap_enabled {
        let ldap_config: crate::ldap::LdapConfig = sec.into();
        match LdapAuth::authenticate(&ldap_config, &req.username, &req.password).await {
            Ok(ldap_info) => {
                // Auto-provision or fetch the user
                let user = match state.db.get_user_by_username(&ldap_info.username).await {
                    Ok(u) => u,
                    Err(_) => {
                        // Create new user from LDAP
                        state
                            .db
                            .create_user(
                                &ldap_info.username,
                                &ldap_info.email,
                                &ldap_info.display_name,
                                "member",
                                "", // No local password for LDAP users
                            )
                            .await?
                    }
                };

                let _ = state
                    .db
                    .record_login_attempt(&req.username, "ldap", true)
                    .await;
                let _ = state.db.clear_login_attempts(&req.username).await;

                let token = state.jwt_service.generate_token(
                    &user.id.to_string(),
                    &user.username,
                    &user.role,
                    None,
                )?;

                return Ok(LoginResponse {
                    token,
                    user: UserResponse::from(user),
                });
            }
            Err(_) => {
                // Fall through to local auth on LDAP failure
            }
        }
    }

    let user = match state.db.get_user_by_username(&req.username).await {
        Ok(u) => u,
        Err(_) => {
            let _ = state
                .db
                .record_login_attempt(&req.username, "unknown", false)
                .await;
            return Err(CoreError::Auth("Invalid username or password".into()));
        }
    };

    let stored_hash = match state.db.get_password_hash(user.id).await {
        Ok(Some(h)) if !h.is_empty() => h,
        _ => {
            return Err(CoreError::Auth("Invalid username or password".into()));
        }
    };

    if !civit_auth::password::verify_password(&req.password, &stored_hash) {
        let _ = state
            .db
            .record_login_attempt(&req.username, "unknown", false)
            .await;
        return Err(CoreError::Auth("Invalid username or password".into()));
    }

    // Record success and clear failures
    let _ = state
        .db
        .record_login_attempt(&req.username, "unknown", true)
        .await;
    let _ = state.db.clear_login_attempts(&req.username).await;

    let token =
        state
            .jwt_service
            .generate_token(&user.id.to_string(), &user.username, &user.role, None)?;

    Ok(LoginResponse {
        token,
        user: UserResponse::from(user),
    })
}

pub async fn me(auth: AuthUser) -> impl IntoResponse {
    (StatusCode::OK, Json(MeResponse::from(auth))).into_response()
}

pub async fn refresh(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let auth_header = match headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| crate::auth::jwt::JwtService::extract_bearer(v))
    {
        Some(h) => h.to_string(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(CoreError::Forbidden("missing authorization header".into()).error_response()),
            )
                .into_response();
        }
    };

    let claims = match state.jwt_service.validate_token(&auth_header) {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(CoreError::Forbidden("invalid or expired token".into()).error_response()),
            )
                .into_response();
        }
    };

    let user_id = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(
                    CoreError::Forbidden("invalid user id in token claims".into()).error_response(),
                ),
            )
                .into_response();
        }
    };

    let user = match state.db.get_user_by_id(user_id).await {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("user not found".into()).error_response()),
            )
                .into_response();
        }
    };

    let token = match state.jwt_service.generate_token(
        &user.id.to_string(),
        &user.username,
        &user.role,
        None,
    ) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Internal(format!("token generation failed: {e}")).error_response()),
            )
                .into_response();
        }
    };

    (StatusCode::OK, Json(RefreshResponse { token })).into_response()
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct ChangePasswordResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct LdapAdminStatusResponse {
    pub enabled: bool,
    pub connected: bool,
    pub server_url: String,
    pub bind_dn: String,
    pub search_base: String,
    pub group_search_base: String,
}

#[derive(Debug, Deserialize)]
pub struct LdapAdminTestRequest {}

#[derive(Debug, Serialize)]
pub struct LdapAdminSyncResponse {
    pub groups_synced: i32,
    pub users_mapped: i32,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct OidcTestRequest {}

#[derive(Debug, Serialize)]
pub struct OidcTestResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct OidcUsersCountResponse {
    pub count: i64,
}

pub async fn ldap_sync(
    State(state): State<AppState>,
    Json(req): Json<LdapSyncRequest>,
) -> impl IntoResponse {
    match do_ldap_sync(&state, req).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => (e.status_code(), Json(e.error_response())).into_response(),
    }
}

async fn do_ldap_sync(
    state: &AppState,
    req: LdapSyncRequest,
) -> crate::error::Result<LdapSyncResponse> {
    if req.username.is_empty() {
        return Err(CoreError::Auth("Username is required".into()));
    }

    let sec = &state.config.security;
    if !sec.ldap_enabled {
        return Err(CoreError::Auth("LDAP is not enabled".into()));
    }

    let ldap_config: crate::ldap::LdapConfig = sec.into();
    let groups = LdapAuth::sync_groups(&ldap_config, &req.username).await?;

    Ok(LdapSyncResponse {
        status: "ok".into(),
        groups: groups.clone(),
        message: format!("Synced {} LDAP groups", groups.len()),
    })
}

pub async fn change_password_auth(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    match do_change_password_auth(&state, &auth, req).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => (e.status_code(), Json(e.error_response())).into_response(),
    }
}

async fn do_change_password_auth(
    state: &AppState,
    auth: &AuthUser,
    req: ChangePasswordRequest,
) -> crate::error::Result<ChangePasswordResponse> {
    if req.current_password.is_empty() || req.new_password.is_empty() {
        return Err(CoreError::Auth(
            "Current and new password are required".into(),
        ));
    }

    let user_uuid = uuid::Uuid::parse_str(&auth.user_id)
        .map_err(|_| CoreError::Config("invalid user ID".into()))?;

    let violations =
        crate::api::password::validate_password_policy(&req.new_password, &state.config.security);
    if !violations.is_empty() {
        return Err(CoreError::Auth(violations.join("; ")));
    }

    if req.current_password == req.new_password {
        return Err(CoreError::Auth(
            "New password must differ from current password".into(),
        ));
    }

    let stored_hash = state
        .db
        .get_password_hash(user_uuid)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?
        .ok_or_else(|| CoreError::Auth("user has no password set".into()))?;

    if !civit_auth::password::verify_password(&req.current_password, &stored_hash) {
        return Err(CoreError::Auth("Current password is incorrect".into()));
    }

    let new_hash = civit_auth::password::hash_password(&req.new_password)?;

    state
        .db
        .change_password(user_uuid, &new_hash)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

    Ok(ChangePasswordResponse {
        status: "ok".into(),
        message: "Password changed successfully".into(),
    })
}

pub async fn ldap_admin_status(State(state): State<AppState>, auth: AuthUser) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let sec = &state.config.security;
    let ldap_config: crate::ldap::LdapConfig = sec.into();

    let connected = if sec.ldap_enabled {
        crate::ldap::LdapAuth::test_connection(&ldap_config)
            .await
            .unwrap_or_default()
    } else {
        false
    };

    let resp = LdapAdminStatusResponse {
        enabled: sec.ldap_enabled,
        connected,
        server_url: sec.ldap_url.clone(),
        bind_dn: sec.ldap_bind_dn.clone(),
        search_base: sec.ldap_user_search_base.clone(),
        group_search_base: sec.ldap_group_search_base.clone(),
    };

    (StatusCode::OK, Json(resp)).into_response()
}

pub async fn ldap_admin_test(State(state): State<AppState>, auth: AuthUser) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let sec = &state.config.security;
    let ldap_config: crate::ldap::LdapConfig = sec.into();

    match crate::ldap::LdapAuth::test_connection(&ldap_config).await {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "ok", "message": "LDAP connection successful"})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"status": "error", "message": "LDAP connection failed"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                serde_json::json!({"status": "error", "message": format!("LDAP test error: {e}")}),
            ),
        )
            .into_response(),
    }
}

pub async fn ldap_admin_sync(State(state): State<AppState>, auth: AuthUser) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let sec = &state.config.security;
    if !sec.ldap_enabled {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"status": "error", "message": "LDAP is not enabled"})),
        )
            .into_response();
    }

    let ldap_config: crate::ldap::LdapConfig = sec.into();

    match crate::ldap::LdapAuth::sync_all_groups(&ldap_config).await {
        Ok((groups_synced, users_mapped)) => (
            StatusCode::OK,
            Json(LdapAdminSyncResponse {
                groups_synced: groups_synced as i32,
                users_mapped: users_mapped as i32,
                message: format!("Synced {groups_synced} LDAP groups, mapped {users_mapped} users"),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"status": "error", "message": format!("Sync failed: {e}")})),
        )
            .into_response(),
    }
}

pub async fn oidc_admin_test(
    State(state): State<AppState>,
    Path(id): Path<String>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let provider_id = match uuid::Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid provider id".into()).error_response()),
            )
                .into_response();
        }
    };

    let pool = state.db.pool();
    let result = sqlx::query_as::<_, crate::api::oidc::OidcProviderRow>(
        "SELECT id, name, issuer, client_id, jwks_uri, client_secret, enabled, created_at, updated_at FROM oidc_providers WHERE id = $1",
    )
    .bind(provider_id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(provider)) => {
            let issuer_url = &provider.issuer;
            match reqwest::get(format!("{issuer_url}/.well-known/openid-configuration")).await {
                Ok(resp) if resp.status().is_success() => (
                    StatusCode::OK,
                    Json(OidcTestResponse {
                        status: "ok".into(),
                        message: format!("Connection to {} successful", provider.name),
                    }),
                )
                    .into_response(),
                _ => (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(OidcTestResponse {
                        status: "error".into(),
                        message: format!("Failed to connect to {}", provider.name),
                    }),
                )
                    .into_response(),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("provider not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn oidc_admin_users_count(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let pool = state.db.pool();
    let result =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(DISTINCT user_id) FROM oidc_identities")
            .fetch_one(pool)
            .await;

    match result {
        Ok(count) => (StatusCode::OK, Json(OidcUsersCountResponse { count })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Generate a random 6-digit verification code.
fn generate_verification_code() -> String {
    let code: u32 = rand::random::<u32>() % 1_000_000;
    format!("{code:06}")
}

/// Verify email with a code sent during registration.
pub async fn verify_email(
    State(state): State<AppState>,
    Json(req): Json<VerifyEmailRequest>,
) -> impl IntoResponse {
    match do_verify_email(&state, req).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => (e.status_code(), Json(e.error_response())).into_response(),
    }
}

async fn do_verify_email(
    state: &AppState,
    req: VerifyEmailRequest,
) -> crate::error::Result<VerifyEmailResponse> {
    if req.email.is_empty() || req.code.is_empty() {
        return Err(CoreError::Auth("Email and code are required".into()));
    }

    let code_record = state
        .db
        .validate_verification_code(&req.email, &req.code)
        .await?;

    // Mark the code as used
    state.db.mark_verification_code_used(code_record.id).await?;

    // Set email_verified = true
    state.db.set_email_verified(code_record.user_id).await?;

    Ok(VerifyEmailResponse {
        status: "ok".into(),
        message: "Email verified successfully".into(),
    })
}

// ── OAuth2/PKCE Endpoints ──

/// OAuth2 Authorization endpoint (GET /api/v1/oauth/authorize)
pub async fn oauth_authorize(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<OAuthAuthorizeParams>,
) -> impl IntoResponse {
    // Validate response_type
    if params.response_type != "code" {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest("response_type must be 'code'".into()).error_response()),
        )
            .into_response();
    }

    // Validate code_challenge_method
    if params.code_challenge_method != "S256" {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest("code_challenge_method must be 'S256'".into()).error_response()),
        )
            .into_response();
    }

    // Validate client_id
    let client = match sqlx::query_as::<_, (String, String, serde_json::Value)>(
        "SELECT client_id, name, redirect_uris FROM oauth_clients WHERE client_id = $1",
    )
    .bind(&params.client_id)
    .fetch_optional(state.db.pool())
    .await
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid client_id".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    // Validate redirect_uri
    let redirect_uris: Vec<String> = serde_json::from_value(client.2).unwrap_or_default();
    if !redirect_uris.contains(&params.redirect_uri) {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest("invalid redirect_uri".into()).error_response()),
        )
            .into_response();
    }

    // Generate authorization code
    let code = generate_random_string(64);
    let expires_at = Utc::now() + Duration::minutes(10);
    let user_id = uuid::Uuid::parse_str(&auth.user_id).unwrap_or(uuid::Uuid::nil());

    // Store the authorization code
    if let Err(e) = sqlx::query(
        "INSERT INTO oauth_codes (code, code_challenge, code_challenge_method, client_id, user_id, redirect_uri, state, expires_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(&code)
    .bind(&params.code_challenge)
    .bind(&params.code_challenge_method)
    .bind(&params.client_id)
    .bind(user_id)
    .bind(&params.redirect_uri)
    .bind(&params.state)
    .bind(expires_at)
    .execute(state.db.pool())
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response();
    }

    // Redirect to client with code and state
    let mut redirect_url = format!("{}?code={}", params.redirect_uri, code);
    if let Some(ref state_param) = params.state {
        redirect_url = format!("{}&state={}", redirect_url, state_param);
    }

    Redirect::temporary(&redirect_url).into_response()
}

/// OAuth2 Token endpoint (POST /api/v1/oauth/token)
pub async fn oauth_token(
    State(state): State<AppState>,
    Json(req): Json<OAuthTokenRequest>,
) -> impl IntoResponse {
    // Validate grant_type
    if req.grant_type != "authorization_code" {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "unsupported_grant_type",
                "error_description": "grant_type must be 'authorization_code'"
            })),
        )
            .into_response();
    }

    // Find and validate the authorization code
    let code_record = match sqlx::query_as::<_, (uuid::Uuid, String, String, uuid::Uuid, String, bool)>(
        "SELECT id, code_challenge, client_id, user_id, redirect_uri, used FROM oauth_codes WHERE code = $1 AND expires_at > NOW()",
    )
    .bind(&req.code)
    .fetch_optional(state.db.pool())
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_grant",
                    "error_description": "invalid or expired authorization code"
                })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    // Check if code was already used
    if code_record.5 {
        // Mark as compromised - delete all codes for this client
        let _ = sqlx::query("DELETE FROM oauth_codes WHERE client_id = $1")
            .bind(&code_record.2)
            .execute(state.db.pool())
            .await;

        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_grant",
                "error_description": "authorization code already used"
            })),
        )
            .into_response();
    }

    // Validate client_id matches
    if code_record.2 != req.client_id {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_grant",
                "error_description": "client_id mismatch"
            })),
        )
            .into_response();
    }

    // Validate redirect_uri matches
    if code_record.4 != req.redirect_uri {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_grant",
                "error_description": "redirect_uri mismatch"
            })),
        )
            .into_response();
    }

    // Validate PKCE: compute SHA256 of code_verifier and compare with stored code_challenge
    let mut hasher = Sha256::new();
    hasher.update(req.code_verifier.as_bytes());
    let computed_challenge = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        hasher.finalize(),
    );

    if computed_challenge != code_record.1 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_grant",
                "error_description": "code_verifier validation failed"
            })),
        )
            .into_response();
    }

    // Mark the code as used
    if let Err(e) = sqlx::query("UPDATE oauth_codes SET used = true WHERE id = $1")
        .bind(code_record.0)
        .execute(state.db.pool())
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response();
    }

    // Get user info
    let user = match state.db.get_user_by_id(code_record.3).await {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("user not found".into()).error_response()),
            )
                .into_response();
        }
    };

    // Generate JWT access token
    let access_token = match state.jwt_service.generate_token(
        &user.id.to_string(),
        &user.username,
        &user.role,
        None,
    ) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Internal(format!("token generation failed: {e}")).error_response()),
            )
                .into_response();
        }
    };

    // Generate refresh token
    let refresh_token = generate_random_string(64);
    let refresh_token_hash = hash_token(&refresh_token);
    let refresh_expires_at = Utc::now() + Duration::days(30);

    if let Err(e) = sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, scope, expires_at) VALUES ($1, $2, $3, $4)",
    )
    .bind(code_record.3)
    .bind(&refresh_token_hash)
    .bind("openid profile")
    .bind(refresh_expires_at)
    .execute(state.db.pool())
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response();
    }

    // Return tokens
    let response = OAuthTokenResponse {
        access_token,
        token_type: "Bearer".into(),
        expires_in: state.jwt_service.expiry_seconds(),
        refresh_token: Some(refresh_token),
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// OAuth2 Refresh Token endpoint (POST /api/v1/oauth/refresh)
pub async fn oauth_refresh(
    State(state): State<AppState>,
    Json(req): Json<OAuthRefreshRequest>,
) -> impl IntoResponse {
    // Validate grant_type
    if req.grant_type != "refresh_token" {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "unsupported_grant_type",
                "error_description": "grant_type must be 'refresh_token'"
            })),
        )
            .into_response();
    }

    // Hash the refresh token
    let token_hash = hash_token(&req.refresh_token);

    // Find the refresh token
    let token_record = match sqlx::query_as::<_, (uuid::Uuid, uuid::Uuid, bool)>(
        "SELECT id, user_id, revoked_at IS NOT NULL as revoked FROM refresh_tokens WHERE token_hash = $1 AND expires_at > NOW()",
    )
    .bind(&token_hash)
    .fetch_optional(state.db.pool())
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_grant",
                    "error_description": "invalid or expired refresh token"
                })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    // Check if revoked
    if token_record.2 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_grant",
                "error_description": "refresh token has been revoked"
            })),
        )
            .into_response();
    }

    // Get user
    let user = match state.db.get_user_by_id(token_record.1).await {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("user not found".into()).error_response()),
            )
                .into_response();
        }
    };

    // Generate new access token
    let access_token = match state.jwt_service.generate_token(
        &user.id.to_string(),
        &user.username,
        &user.role,
        None,
    ) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Internal(format!("token generation failed: {e}")).error_response()),
            )
                .into_response();
        }
    };

    // Generate new refresh token (rotation)
    let new_refresh_token = generate_random_string(64);
    let new_refresh_token_hash = hash_token(&new_refresh_token);
    let new_refresh_expires_at = Utc::now() + Duration::days(30);

    // Revoke old token and insert new one
    let _ = sqlx::query("UPDATE refresh_tokens SET revoked_at = NOW() WHERE id = $1")
        .bind(token_record.0)
        .execute(state.db.pool())
        .await;

    if let Err(e) = sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, scope, expires_at) VALUES ($1, $2, $3, $4)",
    )
    .bind(token_record.1)
    .bind(&new_refresh_token_hash)
    .bind("openid profile")
    .bind(new_refresh_expires_at)
    .execute(state.db.pool())
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response();
    }

    let response = OAuthTokenResponse {
        access_token,
        token_type: "Bearer".into(),
        expires_in: state.jwt_service.expiry_seconds(),
        refresh_token: Some(new_refresh_token),
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// Register a new OAuth2 client (POST /api/v1/oauth/clients)
pub async fn oauth_register_client(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<OAuthRegisterClientRequest>,
) -> impl IntoResponse {
    // Require admin
    if let Err(rejection) = require_admin(&_auth) {
        return rejection.into_response();
    }

    // Validate inputs
    if req.name.is_empty() || req.name.len() > 255 {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest("name must be 1-255 characters".into()).error_response()),
        )
            .into_response();
    }

    if req.redirect_uris.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest("at least one redirect_uri is required".into()).error_response()),
        )
            .into_response();
    }

    // Generate client_id and client_secret
    let client_id = format!("cf_{}", generate_random_string(32));
    let client_secret = generate_random_string(64);
    let client_secret_hash = hash_token(&client_secret);

    // Store the client
    if let Err(e) = sqlx::query(
        "INSERT INTO oauth_clients (client_id, client_secret_hash, name, redirect_uris) VALUES ($1, $2, $3, $4)",
    )
    .bind(&client_id)
    .bind(&client_secret_hash)
    .bind(&req.name)
    .bind(serde_json::to_value(&req.redirect_uris).unwrap_or_default())
    .execute(state.db.pool())
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response();
    }

    let response = OAuthRegisterClientResponse {
        client_id,
        client_secret,
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

#[allow(dead_code)]
fn validate_input(username: &str, email: &str, display_name: &str) -> Result<(), CoreError> {
    if username.is_empty() || username.len() > 64 {
        return Err(CoreError::Config("username must be 1-64 characters".into()));
    }
    if email.len() > 254 {
        return Err(CoreError::Config(
            "email must be at most 254 characters".into(),
        ));
    }
    if display_name.len() > 256 {
        return Err(CoreError::Config(
            "display_name must be at most 256 characters".into(),
        ));
    }

    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(CoreError::Config(
            "username must contain only alphanumeric characters, hyphens, and underscores".into(),
        ));
    }

    if !email.contains('@') || !email.contains('.') {
        return Err(CoreError::Config("invalid email format".into()));
    }

    for s in [username, email, display_name] {
        for ch in s.chars() {
            if ch.is_control() || ch == '\u{202E}' || ch == '\u{200B}' {
                return Err(CoreError::Config(
                    "input contains invalid characters".into(),
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::rbac::Role;

    #[test]
    fn test_validate_input_success() {
        assert!(validate_input("alice", "alice@example.com", "Alice").is_ok());
        assert!(validate_input("user-name", "a@b.co", "Bob").is_ok());
        assert!(validate_input("user_name", "test@domain.org", "C D").is_ok());
    }

    #[test]
    fn test_validate_input_empty_username() {
        assert!(validate_input("", "a@b.co", "Alice").is_err());
    }

    #[test]
    fn test_validate_input_long_username() {
        let long = "a".repeat(65);
        assert!(validate_input(&long, "a@b.co", "Alice").is_err());
    }

    #[test]
    fn test_validate_input_long_email() {
        let long = "a".repeat(255) + "@b.co";
        assert!(validate_input("alice", &long, "Alice").is_err());
    }

    #[test]
    fn test_validate_input_long_display_name() {
        let long = "A".repeat(257);
        assert!(validate_input("alice", "a@b.co", &long).is_err());
    }

    #[test]
    fn test_validate_input_invalid_username_chars() {
        assert!(validate_input("alice!", "a@b.co", "Alice").is_err());
        assert!(validate_input("alice.", "a@b.co", "Alice").is_err());
        assert!(validate_input("alice@", "a@b.co", "Alice").is_err());
    }

    #[test]
    fn test_validate_input_invalid_email() {
        assert!(validate_input("alice", "noat", "Alice").is_err());
        assert!(validate_input("alice", "nodot@", "Alice").is_err());
    }

    #[test]
    fn test_validate_input_control_chars() {
        assert!(validate_input("alice\n", "a@b.co", "Alice").is_err());
        assert!(validate_input("alice", "a@b.co\n", "Alice").is_err());
        assert!(validate_input("alice", "a@b.co", "Alice\0").is_err());
    }

    #[test]
    fn test_validate_input_rtl_override() {
        assert!(validate_input("\u{202E}", "a@b.co", "Alice").is_err());
    }

    #[test]
    fn test_validate_input_zero_width() {
        assert!(validate_input("alice", "a@b.co", "\u{200B}").is_err());
    }

    #[test]
    fn test_login_request_parse() {
        let json = r#"{"username":"alice","password":"secret123"}"#;
        let req: LoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.username, "alice");
        assert_eq!(req.password, "secret123");
    }

    #[test]
    fn test_register_request_parse() {
        let json = r#"{"username":"alice","email":"alice@example.com","display_name":"Alice","password":"secret123"}"#;
        let req: RegisterRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.username, "alice");
        assert_eq!(req.email, "alice@example.com");
        assert_eq!(req.display_name, "Alice");
        assert_eq!(req.password, "secret123");
    }

    #[test]
    fn test_login_request_missing_fields() {
        let json = r#"{"username":"alice"}"#;
        let result = serde_json::from_str::<LoginRequest>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_me_response_from_auth_user() {
        let auth = AuthUser {
            user_id: "u-1".into(),
            username: "alice".into(),
            role: Role::Admin,
            org_id: Some("org-1".into()),
        };
        let resp = MeResponse::from(auth);
        assert_eq!(resp.user_id, "u-1");
        assert_eq!(resp.username, "alice");
        assert_eq!(resp.role, "admin");
        assert_eq!(resp.org_id.as_deref(), Some("org-1"));
    }

    #[test]
    fn test_me_response_no_org() {
        let auth = AuthUser {
            user_id: "u-2".into(),
            username: "bob".into(),
            role: Role::Guest,
            org_id: None,
        };
        let resp = MeResponse::from(auth);
        assert!(resp.org_id.is_none());
        assert_eq!(resp.role, "guest");
    }

    #[test]
    fn test_me_response_serialization() {
        let auth = AuthUser {
            user_id: "u-1".into(),
            username: "alice".into(),
            role: Role::Member,
            org_id: None,
        };
        let resp = MeResponse::from(auth);
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"username\":\"alice\""));
        assert!(json.contains("\"role\":\"member\""));
        assert!(json.contains("\"org_id\":null"));
    }

    #[test]
    fn test_login_response_serialization() {
        let resp = LoginResponse {
            token: "jwt-token".into(),
            user: UserResponse {
                id: "123".into(),
                username: "alice".into(),
                email: "alice@example.com".into(),
                display_name: "Alice".into(),
                bio: None,
                role: "admin".into(),
                avatar_url: None,
                location: None,
                website: None,
                created_at: "2025-01-01T00:00:00+00:00".into(),
                updated_at: "2025-01-01T00:00:00+00:00".into(),
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"token\":\"jwt-token\""));
        assert!(json.contains("\"username\":\"alice\""));
    }

    #[test]
    fn test_generate_verification_code_is_six_digits() {
        let code = generate_verification_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
        let num: u32 = code.parse().unwrap();
        assert!(num < 1_000_000);
    }

    #[test]
    fn test_generate_verification_code_varies() {
        let codes: Vec<String> = (0..10).map(|_| generate_verification_code()).collect();
        let unique: std::collections::HashSet<&str> = codes.iter().map(|s| s.as_str()).collect();
        assert!(unique.len() > 1, "codes should vary across calls");
    }

    #[test]
    fn test_verify_email_request_parse() {
        let json = r#"{"email":"alice@example.com","code":"123456"}"#;
        let req: VerifyEmailRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.email, "alice@example.com");
        assert_eq!(req.code, "123456");
    }

    #[test]
    fn test_verify_email_response_serialization() {
        let resp = VerifyEmailResponse {
            status: "ok".into(),
            message: "Email verified successfully".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("Email verified successfully"));
    }
}
