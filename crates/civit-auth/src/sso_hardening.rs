#![forbid(unsafe_code)]

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, thiserror::Error)]
pub enum SsoError {
    #[error("SSO configuration error: {0}")]
    Config(String),

    #[error("OIDC validation error: {0}")]
    OidcValidation(String),

    #[error("SAML validation error: {0}")]
    SamlValidation(String),

    #[error("session error: {0}")]
    Session(String),

    #[error("MFA required for this operation")]
    MfaRequired,

    #[error("provisioning error: {0}")]
    Provisioning(String),

    #[error("token expired")]
    TokenExpired,

    #[error("issuer mismatch: expected {expected}, got {actual}")]
    IssuerMismatch { expected: String, actual: String },

    #[error("JWKS rotation required")]
    JwksRotationRequired,

    #[error("internal error: {0}")]
    Internal(String),
}

pub type SsoResult<T> = std::result::Result<T, SsoError>;

// -- OIDC Provider Configuration --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcProviderConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwks_uri: String,
    pub scopes: Vec<String>,
    pub claim_mappings: ClaimMappings,
    pub group_role_mapping: HashMap<String, String>,
    pub allowed_audiences: Vec<String>,
    pub clock_skew_secs: i64,
}

impl OidcProviderConfig {
    pub fn new(issuer_url: String, client_id: String, client_secret: String) -> Self {
        Self {
            issuer_url,
            client_id,
            client_secret,
            authorization_endpoint: String::new(),
            token_endpoint: String::new(),
            userinfo_endpoint: String::new(),
            jwks_uri: String::new(),
            scopes: vec!["openid".into(), "profile".into(), "email".into()],
            claim_mappings: ClaimMappings::default(),
            group_role_mapping: HashMap::new(),
            allowed_audiences: Vec::new(),
            clock_skew_secs: 30,
        }
    }

    pub fn with_endpoints(
        mut self,
        authorization: &str,
        token: &str,
        userinfo: &str,
        jwks: &str,
    ) -> Self {
        self.authorization_endpoint = authorization.into();
        self.token_endpoint = token.into();
        self.userinfo_endpoint = userinfo.into();
        self.jwks_uri = jwks.into();
        self
    }

    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }

    pub fn with_claim_mappings(mut self, mappings: ClaimMappings) -> Self {
        self.claim_mappings = mappings;
        self
    }

    pub fn with_group_role_mapping(mut self, mapping: HashMap<String, String>) -> Self {
        self.group_role_mapping = mapping;
        self
    }

    pub fn with_allowed_audiences(mut self, audiences: Vec<String>) -> Self {
        self.allowed_audiences = audiences;
        self
    }

    pub fn with_clock_skew(mut self, secs: i64) -> Self {
        self.clock_skew_secs = secs;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimMappings {
    pub sub: String,
    pub email: String,
    pub email_verified: String,
    pub username: String,
    pub display_name: String,
    pub groups: String,
    pub roles: String,
}

impl Default for ClaimMappings {
    fn default() -> Self {
        Self {
            sub: "sub".into(),
            email: "email".into(),
            email_verified: "email_verified".into(),
            username: "preferred_username".into(),
            display_name: "name".into(),
            groups: "groups".into(),
            roles: "roles".into(),
        }
    }
}

// -- OIDC Token Validation --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcClaims {
    pub issuer: String,
    pub subject: String,
    pub audience: Vec<String>,
    pub expiration: i64,
    pub issued_at: i64,
    pub nonce: Option<String>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub groups: Vec<String>,
    pub roles: Vec<String>,
    pub raw_claims: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct OidcTokenValidator {
    provider: OidcProviderConfig,
    jwks: Arc<RwLock<JwksCache>>,
}

#[derive(Debug, Clone, Default)]
pub struct JwksCache {
    keys: Vec<JwksKey>,
    fetched_at: Option<DateTime<Utc>>,
    _etag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwksKey {
    pub kty: String,
    pub kid: String,
    pub alg: String,
    pub n: Option<String>,
    pub e: Option<String>,
    pub x5c: Option<Vec<String>>,
}

impl OidcTokenValidator {
    pub fn new(provider: OidcProviderConfig) -> Self {
        Self {
            provider,
            jwks: Arc::new(RwLock::new(JwksCache::default())),
        }
    }

    pub fn validate_issuer(&self, issuer: &str) -> SsoResult<()> {
        let expected = &self.provider.issuer_url;
        let normalized_expected = expected.trim_end_matches('/');
        let normalized_actual = issuer.trim_end_matches('/');
        if normalized_expected != normalized_actual {
            return Err(SsoError::IssuerMismatch {
                expected: expected.clone(),
                actual: issuer.into(),
            });
        }
        Ok(())
    }

    pub fn validate_audience(&self, audiences: &[String]) -> SsoResult<()> {
        if self.provider.allowed_audiences.is_empty() {
            return Ok(());
        }
        for aud in audiences {
            if self.provider.allowed_audiences.contains(aud) {
                return Ok(());
            }
        }
        Err(SsoError::OidcValidation(
            "no matching audience found".into(),
        ))
    }

    pub fn validate_timestamps(&self, expiration: i64, issued_at: i64) -> SsoResult<()> {
        let now = Utc::now().timestamp();
        let skew = self.provider.clock_skew_secs;
        if now > expiration + skew {
            return Err(SsoError::TokenExpired);
        }
        if now < issued_at - skew {
            return Err(SsoError::OidcValidation("token issued in the future".into()));
        }
        Ok(())
    }

    pub fn validate_full(&self, claims: &OidcClaims) -> SsoResult<()> {
        self.validate_issuer(&claims.issuer)?;
        self.validate_audience(&claims.audience)?;
        self.validate_timestamps(claims.expiration, claims.issued_at)?;
        Ok(())
    }

    pub fn update_jwks(&self, keys: Vec<JwksKey>) {
        let mut cache = self.jwks.write();
        cache.keys = keys;
        cache.fetched_at = Some(Utc::now());
    }

    pub fn needs_jwks_refresh(&self, max_age: Duration) -> bool {
        let cache = self.jwks.read();
        match cache.fetched_at {
            Some(at) => Utc::now() - at > max_age,
            None => true,
        }
    }

    pub fn jwks_keys(&self) -> Vec<JwksKey> {
        self.jwks.read().keys.clone()
    }

    pub fn extract_claims(&self, raw: &HashMap<String, serde_json::Value>) -> OidcClaims {
        let mappings = &self.provider.claim_mappings;
        let sub = raw
            .get(&mappings.sub)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let email = raw
            .get(&mappings.email)
            .and_then(|v| v.as_str())
            .map(String::from);
        let email_verified = raw
            .get(&mappings.email_verified)
            .and_then(|v| v.as_bool());
        let groups = raw
            .get(&mappings.groups)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let roles = raw
            .get(&mappings.roles)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        OidcClaims {
            issuer: String::new(),
            subject: sub,
            audience: Vec::new(),
            expiration: 0,
            issued_at: 0,
            nonce: None,
            email,
            email_verified,
            groups,
            roles,
            raw_claims: raw.clone(),
        }
    }

    pub fn resolve_role(&self, groups: &[String], oidc_roles: &[String]) -> Option<String> {
        for role in oidc_roles {
            if let Some(mapped) = self.provider.group_role_mapping.get(role) {
                return Some(mapped.clone());
            }
        }
        for group in groups {
            if let Some(mapped) = self.provider.group_role_mapping.get(group) {
                return Some(mapped.clone());
            }
        }
        None
    }
}

// -- SAML Assertion Validation --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAssertionConfig {
    pub entity_id: String,
    pub sso_url: String,
    pub certificate: String,
    pub signature_method: String,
    pub digest_method: String,
    pub audience_restriction: String,
    pub assertion_consumer_service_url: String,
    pub max_clock_skew_secs: i64,
    pub require_signed_assertion: bool,
    pub require_signed_response: bool,
}

impl SamlAssertionConfig {
    pub fn new(entity_id: String, sso_url: String, certificate: String) -> Self {
        Self {
            entity_id,
            sso_url,
            certificate,
            signature_method: "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256".into(),
            digest_method: "http://www.w3.org/2001/04/xmlenc#sha256".into(),
            audience_restriction: String::new(),
            assertion_consumer_service_url: String::new(),
            max_clock_skew_secs: 300,
            require_signed_assertion: true,
            require_signed_response: true,
        }
    }

    pub fn with_signature_method(mut self, method: &str) -> Self {
        self.signature_method = method.into();
        self
    }

    pub fn with_digest_method(mut self, method: &str) -> Self {
        self.digest_method = method.into();
        self
    }

    pub fn with_audience_restriction(mut self, audience: &str) -> Self {
        self.audience_restriction = audience.into();
        self
    }

    pub fn with_max_clock_skew(mut self, secs: i64) -> Self {
        self.max_clock_skew_secs = secs;
        self
    }

    pub fn with_require_signed_assertion(mut self, required: bool) -> Self {
        self.require_signed_assertion = required;
        self
    }

    pub fn with_require_signed_response(mut self, required: bool) -> Self {
        self.require_signed_response = required;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAssertionData {
    pub name_id: String,
    pub session_index: String,
    pub issuer: String,
    pub audience: String,
    pub not_before: DateTime<Utc>,
    pub not_on_or_after: DateTime<Utc>,
    pub authn_instant: DateTime<Utc>,
    pub attributes: HashMap<String, Vec<String>>,
    pub signature_valid: bool,
    pub signed_assertion: bool,
    pub signed_response: bool,
}

#[derive(Debug, Clone)]
pub struct SamlAssertionValidator {
    config: SamlAssertionConfig,
}

impl SamlAssertionValidator {
    pub fn new(config: SamlAssertionConfig) -> Self {
        Self { config }
    }

    pub fn validate_signature(&self, assertion: &SamlAssertionData) -> SsoResult<()> {
        if self.config.require_signed_assertion && !assertion.signed_assertion {
            return Err(SsoError::SamlValidation(
                "assertion must be signed".into(),
            ));
        }
        if self.config.require_signed_response && !assertion.signed_response {
            return Err(SsoError::SamlValidation(
                "response must be signed".into(),
            ));
        }
        if !assertion.signature_valid {
            return Err(SsoError::SamlValidation(
                "signature verification failed".into(),
            ));
        }
        Ok(())
    }

    pub fn validate_timestamp(&self, assertion: &SamlAssertionData) -> SsoResult<()> {
        let now = Utc::now();
        let skew = Duration::seconds(self.config.max_clock_skew_secs);
        if now + skew < assertion.not_before {
            return Err(SsoError::SamlValidation(
                "assertion not yet valid (not_before)".into(),
            ));
        }
        if now - skew >= assertion.not_on_or_after {
            return Err(SsoError::SamlValidation(
                "assertion expired (not_on_or_after)".into(),
            ));
        }
        Ok(())
    }

    pub fn validate_audience(&self, assertion: &SamlAssertionData) -> SsoResult<()> {
        let expected = &self.config.audience_restriction;
        if expected.is_empty() {
            return Ok(());
        }
        if assertion.audience != *expected {
            return Err(SsoError::SamlValidation(format!(
                "audience mismatch: expected '{}', got '{}'",
                expected, assertion.audience
            )));
        }
        Ok(())
    }

    pub fn validate_issuer(&self, assertion: &SamlAssertionData) -> SsoResult<()> {
        if assertion.issuer != self.config.entity_id {
            return Err(SsoError::SamlValidation(format!(
                "issuer mismatch: expected '{}', got '{}'",
                self.config.entity_id, assertion.issuer
            )));
        }
        Ok(())
    }

    pub fn validate_full(&self, assertion: &SamlAssertionData) -> SsoResult<()> {
        self.validate_signature(assertion)?;
        self.validate_timestamp(assertion)?;
        self.validate_audience(assertion)?;
        self.validate_issuer(assertion)?;
        Ok(())
    }

    pub fn extract_groups(&self, assertion: &SamlAssertionData) -> Vec<String> {
        assertion
            .attributes
            .get("groups")
            .or_else(|| assertion.attributes.get("memberOf"))
            .cloned()
            .unwrap_or_default()
    }

    pub fn extract_roles(&self, assertion: &SamlAssertionData) -> Vec<String> {
        assertion
            .attributes
            .get("roles")
            .or_else(|| assertion.attributes.get("groups"))
            .cloned()
            .unwrap_or_default()
    }
}

// -- Session Fixation Prevention --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub session_timeout_secs: i64,
    pub absolute_timeout_secs: i64,
    pub idle_timeout_secs: i64,
    pub regenerate_on_auth: bool,
    pub max_concurrent_sessions: u32,
    pub bind_to_ip: bool,
    pub secure_cookie: bool,
    pub http_only_cookie: bool,
    pub same_site: SameSitePolicy,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_timeout_secs: 3600,
            absolute_timeout_secs: 86400,
            idle_timeout_secs: 1800,
            regenerate_on_auth: true,
            max_concurrent_sessions: 5,
            bind_to_ip: false,
            secure_cookie: true,
            http_only_cookie: true,
            same_site: SameSitePolicy::Strict,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SameSitePolicy {
    Strict,
    Lax,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionToken {
    pub id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub mfa_verified: bool,
    pub fingerprint: String,
}

impl SessionToken {
    pub fn new(user_id: &str, fingerprint: &str, config: &SessionConfig) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.into(),
            created_at: now,
            last_activity: now,
            expires_at: now + Duration::seconds(config.session_timeout_secs),
            ip_address: None,
            user_agent: None,
            mfa_verified: false,
            fingerprint: fingerprint.into(),
        }
    }

    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        now > self.expires_at
    }

    pub fn is_idle_expired(&self, now: DateTime<Utc>, idle_timeout_secs: i64) -> bool {
        now - self.last_activity > Duration::seconds(idle_timeout_secs)
    }

    pub fn is_absolute_expired(&self, now: DateTime<Utc>, absolute_timeout_secs: i64) -> bool {
        now - self.created_at > Duration::seconds(absolute_timeout_secs)
    }
}

#[derive(Debug, Clone)]
pub struct SessionManager {
    config: SessionConfig,
    sessions: Arc<RwLock<HashMap<String, Vec<SessionToken>>>>,
}

impl SessionManager {
    pub fn new(config: SessionConfig) -> Self {
        Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_session(
        &self,
        user_id: &str,
        fingerprint: &str,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> SessionToken {
        let mut session = SessionToken::new(user_id, fingerprint, &self.config);
        session.ip_address = ip_address.map(String::from);
        session.user_agent = user_agent.map(String::from);

        let mut sessions = self.sessions.write();
        let user_sessions = sessions.entry(user_id.into()).or_default();

        if self.config.regenerate_on_auth {
            let now = Utc::now();
            user_sessions.retain(|s| !s.is_expired(now));
            if user_sessions.len() >= self.config.max_concurrent_sessions as usize {
                user_sessions.remove(0);
            }
        }

        user_sessions.push(session.clone());
        session
    }

    pub fn validate_session(
        &self,
        user_id: &str,
        session_id: &str,
        fingerprint: &str,
    ) -> SsoResult<SessionToken> {
        let sessions = self.sessions.read();
        let user_sessions = sessions
            .get(user_id)
            .ok_or_else(|| SsoError::Session("no sessions found for user".into()))?;

        let session = user_sessions
            .iter()
            .find(|s| s.id == session_id)
            .ok_or_else(|| SsoError::Session("session not found".into()))?;

        let now = Utc::now();

        if session.is_expired(now) {
            return Err(SsoError::Session("session expired".into()));
        }
        if session.is_idle_expired(now, self.config.idle_timeout_secs) {
            return Err(SsoError::Session("session idle timeout".into()));
        }
        if session.is_absolute_expired(now, self.config.absolute_timeout_secs) {
            return Err(SsoError::Session("session absolute timeout".into()));
        }
        if session.fingerprint != fingerprint {
            return Err(SsoError::Session("session fingerprint mismatch".into()));
        }

        Ok(session.clone())
    }

    pub fn invalidate_session(&self, user_id: &str, session_id: &str) -> SsoResult<()> {
        let mut sessions = self.sessions.write();
        if let Some(user_sessions) = sessions.get_mut(user_id) {
            user_sessions.retain(|s| s.id != session_id);
        }
        Ok(())
    }

    pub fn invalidate_all_sessions(&self, user_id: &str) {
        let mut sessions = self.sessions.write();
        sessions.remove(user_id);
    }

    pub fn active_session_count(&self, user_id: &str) -> u32 {
        let sessions = self.sessions.read();
        sessions
            .get(user_id)
            .map(|s| s.len() as u32)
            .unwrap_or(0)
    }
}

// -- MFA Enforcement --

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MfaLevel {
    None,
    Optional,
    Required,
    RequiredForAdmin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaEnforcementPolicy {
    pub level: MfaLevel,
    pub admin_operations_require_mfa: bool,
    pub sensitive_operations: Vec<String>,
    pub grace_period_hours: u32,
    pub allowed_methods: Vec<MfaMethod>,
}

impl Default for MfaEnforcementPolicy {
    fn default() -> Self {
        Self {
            level: MfaLevel::RequiredForAdmin,
            admin_operations_require_mfa: true,
            sensitive_operations: vec![
                "delete_user".into(),
                "update_settings".into(),
                "manage_api_keys".into(),
                "manage_sso".into(),
            ],
            grace_period_hours: 72,
            allowed_methods: vec![MfaMethod::Totp, MfaMethod::WebAuthn],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MfaMethod {
    Totp,
    WebAuthn,
    Sms,
    Email,
}

#[derive(Debug, Clone)]
pub struct MfaEnforcer {
    policy: MfaEnforcementPolicy,
}

impl MfaEnforcer {
    pub fn new(policy: MfaEnforcementPolicy) -> Self {
        Self { policy }
    }

    pub fn requires_mfa(&self, operation: &str, is_admin: bool) -> bool {
        match self.policy.level {
            MfaLevel::None => false,
            MfaLevel::Optional => false,
            MfaLevel::Required => true,
            MfaLevel::RequiredForAdmin => {
                is_admin
                    || (self.policy.admin_operations_require_mfa
                        && self.policy.sensitive_operations.contains(&operation.to_string()))
            }
        }
    }

    pub fn is_method_allowed(&self, method: MfaMethod) -> bool {
        self.policy.allowed_methods.contains(&method)
    }

    pub fn admin_operation_requires_mfa(&self, operation: &str) -> bool {
        self.policy.admin_operations_require_mfa
            && self
                .policy
                .sensitive_operations
                .contains(&operation.to_string())
    }
}

// -- JIT User Provisioning --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitProvisioningConfig {
    pub enabled: bool,
    pub default_role: String,
    pub auto_link_existing: bool,
    pub create_org: bool,
    pub default_org_role: String,
    pub required_claims: Vec<String>,
    pub attribute_transforms: HashMap<String, String>,
}

impl Default for JitProvisioningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_role: "member".into(),
            auto_link_existing: true,
            create_org: false,
            default_org_role: "member".into(),
            required_claims: vec!["email".into()],
            attribute_transforms: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitUserInfo {
    pub external_id: String,
    pub email: String,
    pub username: String,
    pub display_name: Option<String>,
    pub groups: Vec<String>,
    pub roles: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub provider: String,
}

#[derive(Debug, Clone)]
pub struct JitProvisioner {
    config: JitProvisioningConfig,
    group_role_mapping: HashMap<String, String>,
}

impl JitProvisioner {
    pub fn new(
        config: JitProvisioningConfig,
        group_role_mapping: HashMap<String, String>,
    ) -> Self {
        Self {
            config,
            group_role_mapping,
        }
    }

    pub fn validate_required_claims(&self, user_info: &JitUserInfo) -> SsoResult<()> {
        for claim in &self.config.required_claims {
            match claim.as_str() {
                "email" if user_info.email.is_empty() => {
                    return Err(SsoError::Provisioning(
                        "required claim 'email' is missing".into(),
                    ));
                }
                "username" if user_info.username.is_empty() => {
                    return Err(SsoError::Provisioning(
                        "required claim 'username' is missing".into(),
                    ));
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn resolve_role(&self, user_info: &JitUserInfo) -> String {
        for role in &user_info.roles {
            if let Some(mapped) = self.group_role_mapping.get(role) {
                return mapped.clone();
            }
        }
        for group in &user_info.groups {
            if let Some(mapped) = self.group_role_mapping.get(group) {
                return mapped.clone();
            }
        }
        self.config.default_role.clone()
    }

    pub fn should_create_user(&self, existing_user: bool) -> bool {
        !existing_user || self.config.auto_link_existing
    }

    pub fn transform_attribute(&self, key: &str, value: &str) -> String {
        self.config
            .attribute_transforms
            .get(key)
            .map(|t| t.replace("{}", value))
            .unwrap_or_else(|| value.to_string())
    }
}

// -- SSO Session Timeout Configuration --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoSessionTimeoutConfig {
    pub idle_timeout_secs: i64,
    pub absolute_timeout_secs: i64,
    pub refresh_before_expiry_secs: i64,
    pub re_auth_for_admin_secs: i64,
    pub max_reauth_age_secs: i64,
}

impl Default for SsoSessionTimeoutConfig {
    fn default() -> Self {
        Self {
            idle_timeout_secs: 1800,
            absolute_timeout_secs: 86400,
            refresh_before_expiry_secs: 300,
            re_auth_for_admin_secs: 300,
            max_reauth_age_secs: 3600,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRoleMapping {
    pub group_name: String,
    pub mapped_role: String,
    pub organization_id: Option<String>,
    pub permissions: Vec<String>,
}

// -- SSO Hardening State --

#[derive(Debug, Clone)]
pub struct SsoHardeningState {
    pub oidc_validators: Arc<RwLock<HashMap<String, OidcTokenValidator>>>,
    pub saml_validators: Arc<RwLock<HashMap<String, SamlAssertionValidator>>>,
    pub session_manager: SessionManager,
    pub mfa_enforcer: MfaEnforcer,
    pub jit_provisioner: JitProvisioner,
    pub session_timeout_config: SsoSessionTimeoutConfig,
    pub group_role_mappings: Arc<RwLock<HashMap<String, Vec<GroupRoleMapping>>>>,
}

impl SsoHardeningState {
    pub fn new(
        session_config: SessionConfig,
        mfa_policy: MfaEnforcementPolicy,
        jit_config: JitProvisioningConfig,
        session_timeout: SsoSessionTimeoutConfig,
        group_role_mapping: HashMap<String, String>,
    ) -> Self {
        Self {
            oidc_validators: Arc::new(RwLock::new(HashMap::new())),
            saml_validators: Arc::new(RwLock::new(HashMap::new())),
            session_manager: SessionManager::new(session_config),
            mfa_enforcer: MfaEnforcer::new(mfa_policy),
            jit_provisioner: JitProvisioner::new(jit_config, group_role_mapping),
            session_timeout_config: session_timeout,
            group_role_mappings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register_oidc_provider(&self, name: &str, config: OidcProviderConfig) {
        let validator = OidcTokenValidator::new(config);
        let mut validators = self.oidc_validators.write();
        validators.insert(name.into(), validator);
    }

    pub fn register_saml_provider(&self, name: &str, config: SamlAssertionConfig) {
        let validator = SamlAssertionValidator::new(config);
        let mut validators = self.saml_validators.write();
        validators.insert(name.into(), validator);
    }

    pub fn add_group_role_mapping(&self, provider: &str, mapping: GroupRoleMapping) {
        let mut mappings = self.group_role_mappings.write();
        mappings
            .entry(provider.into())
            .or_default()
            .push(mapping);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oidc_provider_config_defaults() {
        let config =
            OidcProviderConfig::new("https://idp.example.com".into(), "client-1".into(), "secret".into());
        assert_eq!(config.issuer_url, "https://idp.example.com");
        assert_eq!(config.client_id, "client-1");
        assert!(config.scopes.contains(&"openid".to_string()));
    }

    #[test]
    fn test_oidc_validate_issuer_pass() {
        let config =
            OidcProviderConfig::new("https://idp.example.com".into(), "c".into(), "s".into());
        let validator = OidcTokenValidator::new(config);
        assert!(validator.validate_issuer("https://idp.example.com").is_ok());
    }

    #[test]
    fn test_oidc_validate_issuer_mismatch() {
        let config =
            OidcProviderConfig::new("https://idp.example.com".into(), "c".into(), "s".into());
        let validator = OidcTokenValidator::new(config);
        assert!(validator.validate_issuer("https://other.com").is_err());
    }

    #[test]
    fn test_oidc_validate_issuer_trailing_slash() {
        let config =
            OidcProviderConfig::new("https://idp.example.com/".into(), "c".into(), "s".into());
        let validator = OidcTokenValidator::new(config);
        assert!(validator.validate_issuer("https://idp.example.com").is_ok());
    }

    #[test]
    fn test_oidc_validate_audience_empty_allows_all() {
        let config =
            OidcProviderConfig::new("https://idp.example.com".into(), "c".into(), "s".into());
        let validator = OidcTokenValidator::new(config);
        assert!(validator.validate_audience(&["anything".into()]).is_ok());
    }

    #[test]
    fn test_oidc_validate_audience_match() {
        let config = OidcProviderConfig::new(
            "https://idp.example.com".into(),
            "c".into(),
            "s".into(),
        )
        .with_allowed_audiences(vec!["my-app".into()]);
        let validator = OidcTokenValidator::new(config);
        assert!(validator.validate_audience(&["my-app".into()]).is_ok());
    }

    #[test]
    fn test_oidc_validate_audience_no_match() {
        let config = OidcProviderConfig::new(
            "https://idp.example.com".into(),
            "c".into(),
            "s".into(),
        )
        .with_allowed_audiences(vec!["my-app".into()]);
        let validator = OidcTokenValidator::new(config);
        assert!(validator.validate_audience(&["other-app".into()]).is_err());
    }

    #[test]
    fn test_oidc_validate_timestamps_expired() {
        let config =
            OidcProviderConfig::new("https://idp.example.com".into(), "c".into(), "s".into());
        let validator = OidcTokenValidator::new(config);
        let past = (Utc::now() - Duration::hours(1)).timestamp();
        let result = validator.validate_timestamps(past, past - 3600);
        assert!(result.is_err());
    }

    #[test]
    fn test_oidc_validate_timestamps_valid() {
        let config =
            OidcProviderConfig::new("https://idp.example.com".into(), "c".into(), "s".into());
        let validator = OidcTokenValidator::new(config);
        let now = Utc::now().timestamp();
        let result = validator.validate_timestamps(now + 3600, now - 60);
        assert!(result.is_ok());
    }

    #[test]
    fn test_oidc_validate_full_pass() {
        let config = OidcProviderConfig::new(
            "https://idp.example.com".into(),
            "c".into(),
            "s".into(),
        )
        .with_allowed_audiences(vec!["my-app".into()]);
        let validator = OidcTokenValidator::new(config);
        let now = Utc::now().timestamp();
        let claims = OidcClaims {
            issuer: "https://idp.example.com".into(),
            subject: "user-1".into(),
            audience: vec!["my-app".into()],
            expiration: now + 3600,
            issued_at: now - 60,
            nonce: None,
            email: Some("user@example.com".into()),
            email_verified: Some(true),
            groups: Vec::new(),
            roles: Vec::new(),
            raw_claims: HashMap::new(),
        };
        assert!(validator.validate_full(&claims).is_ok());
    }

    #[test]
    fn test_oidc_jwks_refresh_needed() {
        let config =
            OidcProviderConfig::new("https://idp.example.com".into(), "c".into(), "s".into());
        let validator = OidcTokenValidator::new(config);
        assert!(validator.needs_jwks_refresh(Duration::hours(1)));
        validator.update_jwks(vec![]);
        assert!(!validator.needs_jwks_refresh(Duration::hours(1)));
    }

    #[test]
    fn test_oidc_resolve_role_from_group() {
        let mut mapping = HashMap::new();
        mapping.insert("admins".into(), "admin".into());
        let config = OidcProviderConfig::new(
            "https://idp.example.com".into(),
            "c".into(),
            "s".into(),
        )
        .with_group_role_mapping(mapping);
        let validator = OidcTokenValidator::new(config);
        let role = validator.resolve_role(&["admins".into()], &[]);
        assert_eq!(role.as_deref(), Some("admin"));
    }

    #[test]
    fn test_oidc_resolve_role_from_oidc_role() {
        let mut mapping = HashMap::new();
        mapping.insert("superadmin".into(), "super-admin".into());
        let config = OidcProviderConfig::new(
            "https://idp.example.com".into(),
            "c".into(),
            "s".into(),
        )
        .with_group_role_mapping(mapping);
        let validator = OidcTokenValidator::new(config);
        let role = validator.resolve_role(&[], &["superadmin".into()]);
        assert_eq!(role.as_deref(), Some("super-admin"));
    }

    #[test]
    fn test_saml_config_defaults() {
        let config = SamlAssertionConfig::new(
            "https://idp.example.com/metadata".into(),
            "https://idp.example.com/sso".into(),
            "cert-data".into(),
        );
        assert!(config.require_signed_assertion);
        assert!(config.require_signed_response);
        assert_eq!(config.max_clock_skew_secs, 300);
    }

    #[test]
    fn test_saml_validate_signature_pass() {
        let config = SamlAssertionConfig::new(
            "https://idp.example.com/metadata".into(),
            "https://idp.example.com/sso".into(),
            "cert".into(),
        );
        let validator = SamlAssertionValidator::new(config);
        let assertion = SamlAssertionData {
            name_id: "user@example.com".into(),
            session_index: "session-1".into(),
            issuer: "https://idp.example.com/metadata".into(),
            audience: String::new(),
            not_before: Utc::now() - Duration::minutes(5),
            not_on_or_after: Utc::now() + Duration::minutes(5),
            authn_instant: Utc::now(),
            attributes: HashMap::new(),
            signature_valid: true,
            signed_assertion: true,
            signed_response: true,
        };
        assert!(validator.validate_signature(&assertion).is_ok());
    }

    #[test]
    fn test_saml_validate_signature_unsigned_assertion() {
        let config = SamlAssertionConfig::new(
            "https://idp.example.com/metadata".into(),
            "https://idp.example.com/sso".into(),
            "cert".into(),
        );
        let validator = SamlAssertionValidator::new(config);
        let assertion = SamlAssertionData {
            name_id: "user@example.com".into(),
            session_index: "session-1".into(),
            issuer: "https://idp.example.com/metadata".into(),
            audience: String::new(),
            not_before: Utc::now() - Duration::minutes(5),
            not_on_or_after: Utc::now() + Duration::minutes(5),
            authn_instant: Utc::now(),
            attributes: HashMap::new(),
            signature_valid: true,
            signed_assertion: false,
            signed_response: true,
        };
        assert!(validator.validate_signature(&assertion).is_err());
    }

    #[test]
    fn test_saml_validate_timestamp_expired() {
        let config = SamlAssertionConfig::new(
            "https://idp.example.com/metadata".into(),
            "https://idp.example.com/sso".into(),
            "cert".into(),
        );
        let validator = SamlAssertionValidator::new(config);
        let assertion = SamlAssertionData {
            name_id: "user@example.com".into(),
            session_index: "session-1".into(),
            issuer: "https://idp.example.com/metadata".into(),
            audience: String::new(),
            not_before: Utc::now() - Duration::hours(2),
            not_on_or_after: Utc::now() - Duration::hours(1),
            authn_instant: Utc::now() - Duration::hours(2),
            attributes: HashMap::new(),
            signature_valid: true,
            signed_assertion: true,
            signed_response: true,
        };
        assert!(validator.validate_timestamp(&assertion).is_err());
    }

    #[test]
    fn test_saml_validate_audience_pass() {
        let config = SamlAssertionConfig::new(
            "https://idp.example.com/metadata".into(),
            "https://idp.example.com/sso".into(),
            "cert".into(),
        )
        .with_audience_restriction("https://myapp.example.com");
        let validator = SamlAssertionValidator::new(config);
        let assertion = SamlAssertionData {
            name_id: "user@example.com".into(),
            session_index: "session-1".into(),
            issuer: "https://idp.example.com/metadata".into(),
            audience: "https://myapp.example.com".into(),
            not_before: Utc::now() - Duration::minutes(5),
            not_on_or_after: Utc::now() + Duration::minutes(5),
            authn_instant: Utc::now(),
            attributes: HashMap::new(),
            signature_valid: true,
            signed_assertion: true,
            signed_response: true,
        };
        assert!(validator.validate_audience(&assertion).is_ok());
    }

    #[test]
    fn test_saml_validate_audience_mismatch() {
        let config = SamlAssertionConfig::new(
            "https://idp.example.com/metadata".into(),
            "https://idp.example.com/sso".into(),
            "cert".into(),
        )
        .with_audience_restriction("https://myapp.example.com");
        let validator = SamlAssertionValidator::new(config);
        let assertion = SamlAssertionData {
            name_id: "user@example.com".into(),
            session_index: "session-1".into(),
            issuer: "https://idp.example.com/metadata".into(),
            audience: "https://other.example.com".into(),
            not_before: Utc::now() - Duration::minutes(5),
            not_on_or_after: Utc::now() + Duration::minutes(5),
            authn_instant: Utc::now(),
            attributes: HashMap::new(),
            signature_valid: true,
            signed_assertion: true,
            signed_response: true,
        };
        assert!(validator.validate_audience(&assertion).is_err());
    }

    #[test]
    fn test_saml_extract_groups() {
        let config = SamlAssertionConfig::new(
            "https://idp.example.com/metadata".into(),
            "https://idp.example.com/sso".into(),
            "cert".into(),
        );
        let validator = SamlAssertionValidator::new(config);
        let mut attrs = HashMap::new();
        attrs.insert(
            "groups".into(),
            vec!["admins".into(), "developers".into()],
        );
        let assertion = SamlAssertionData {
            name_id: "user@example.com".into(),
            session_index: "session-1".into(),
            issuer: "https://idp.example.com/metadata".into(),
            audience: String::new(),
            not_before: Utc::now() - Duration::minutes(5),
            not_on_or_after: Utc::now() + Duration::minutes(5),
            authn_instant: Utc::now(),
            attributes: attrs,
            signature_valid: true,
            signed_assertion: true,
            signed_response: true,
        };
        let groups = validator.extract_groups(&assertion);
        assert_eq!(groups.len(), 2);
        assert!(groups.contains(&"admins".to_string()));
    }

    #[test]
    fn test_session_config_defaults() {
        let config = SessionConfig::default();
        assert_eq!(config.session_timeout_secs, 3600);
        assert_eq!(config.idle_timeout_secs, 1800);
        assert!(config.regenerate_on_auth);
        assert_eq!(config.max_concurrent_sessions, 5);
    }

    #[test]
    fn test_session_token_expiry() {
        let config = SessionConfig::default();
        let session = SessionToken::new("user-1", "fp-1", &config);
        assert!(!session.is_expired(Utc::now()));
        assert!(session.is_expired(Utc::now() + Duration::seconds(config.session_timeout_secs + 1)));
    }

    #[test]
    fn test_session_manager_create_and_validate() {
        let config = SessionConfig::default();
        let manager = SessionManager::new(config);
        let session = manager.create_session("user-1", "fp-1", None, None);
        assert_eq!(session.user_id, "user-1");

        let result = manager.validate_session("user-1", &session.id, "fp-1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_session_manager_fingerprint_mismatch() {
        let config = SessionConfig::default();
        let manager = SessionManager::new(config);
        let session = manager.create_session("user-1", "fp-1", None, None);
        let result = manager.validate_session("user-1", &session.id, "fp-2");
        assert!(result.is_err());
    }

    #[test]
    fn test_session_manager_invalidate_all() {
        let config = SessionConfig::default();
        let manager = SessionManager::new(config);
        manager.create_session("user-1", "fp-1", None, None);
        manager.create_session("user-1", "fp-2", None, None);
        assert_eq!(manager.active_session_count("user-1"), 2);
        manager.invalidate_all_sessions("user-1");
        assert_eq!(manager.active_session_count("user-1"), 0);
    }

    #[test]
    fn test_mfa_enforcer_none() {
        let policy = MfaEnforcementPolicy {
            level: MfaLevel::None,
            ..Default::default()
        };
        let enforcer = MfaEnforcer::new(policy);
        assert!(!enforcer.requires_mfa("delete_user", true));
    }

    #[test]
    fn test_mfa_enforcer_required_for_admin() {
        let policy = MfaEnforcementPolicy {
            level: MfaLevel::RequiredForAdmin,
            admin_operations_require_mfa: true,
            sensitive_operations: vec!["delete_user".into()],
            ..Default::default()
        };
        let enforcer = MfaEnforcer::new(policy);
        assert!(enforcer.requires_mfa("delete_user", true));
        assert!(!enforcer.requires_mfa("view_repo", false));
    }

    #[test]
    fn test_mfa_enforcer_admin_operation_check() {
        let policy = MfaEnforcementPolicy {
            level: MfaLevel::RequiredForAdmin,
            admin_operations_require_mfa: true,
            sensitive_operations: vec!["update_settings".into()],
            ..Default::default()
        };
        let enforcer = MfaEnforcer::new(policy);
        assert!(enforcer.admin_operation_requires_mfa("update_settings"));
        assert!(!enforcer.admin_operation_requires_mfa("view_repo"));
    }

    #[test]
    fn test_jit_provisioner_role_resolution() {
        let mut mapping = HashMap::new();
        mapping.insert("admins".into(), "admin".into());
        let config = JitProvisioningConfig::default();
        let provisioner = JitProvisioner::new(config, mapping);
        let user_info = JitUserInfo {
            external_id: "ext-1".into(),
            email: "user@example.com".into(),
            username: "user1".into(),
            display_name: None,
            groups: vec!["admins".into()],
            roles: Vec::new(),
            attributes: HashMap::new(),
            provider: "oidc".into(),
        };
        assert_eq!(provisioner.resolve_role(&user_info), "admin");
    }

    #[test]
    fn test_jit_provisioner_default_role() {
        let config = JitProvisioningConfig::default();
        let provisioner = JitProvisioner::new(config, HashMap::new());
        let user_info = JitUserInfo {
            external_id: "ext-1".into(),
            email: "user@example.com".into(),
            username: "user1".into(),
            display_name: None,
            groups: Vec::new(),
            roles: Vec::new(),
            attributes: HashMap::new(),
            provider: "oidc".into(),
        };
        assert_eq!(provisioner.resolve_role(&user_info), "member");
    }

    #[test]
    fn test_jit_validate_required_claims_pass() {
        let config = JitProvisioningConfig::default();
        let provisioner = JitProvisioner::new(config, HashMap::new());
        let user_info = JitUserInfo {
            external_id: "ext-1".into(),
            email: "user@example.com".into(),
            username: "user1".into(),
            display_name: None,
            groups: Vec::new(),
            roles: Vec::new(),
            attributes: HashMap::new(),
            provider: "oidc".into(),
        };
        assert!(provisioner.validate_required_claims(&user_info).is_ok());
    }

    #[test]
    fn test_jit_validate_required_claims_missing_email() {
        let config = JitProvisioningConfig::default();
        let provisioner = JitProvisioner::new(config, HashMap::new());
        let user_info = JitUserInfo {
            external_id: "ext-1".into(),
            email: String::new(),
            username: "user1".into(),
            display_name: None,
            groups: Vec::new(),
            roles: Vec::new(),
            attributes: HashMap::new(),
            provider: "oidc".into(),
        };
        assert!(provisioner.validate_required_claims(&user_info).is_err());
    }

    #[test]
    fn test_jit_transform_attribute() {
        let mut transforms = HashMap::new();
        transforms.insert("email".into(), "{}@company.com".into());
        let config = JitProvisioningConfig {
            attribute_transforms: transforms,
            ..Default::default()
        };
        let provisioner = JitProvisioner::new(config, HashMap::new());
        let result = provisioner.transform_attribute("email", "john.doe");
        assert_eq!(result, "john.doe@company.com");
    }

    #[test]
    fn test_sso_session_timeout_config_defaults() {
        let config = SsoSessionTimeoutConfig::default();
        assert_eq!(config.idle_timeout_secs, 1800);
        assert_eq!(config.absolute_timeout_secs, 86400);
        assert_eq!(config.refresh_before_expiry_secs, 300);
    }

    #[test]
    fn test_sso_hardening_state_register_providers() {
        let state = SsoHardeningState::new(
            SessionConfig::default(),
            MfaEnforcementPolicy::default(),
            JitProvisioningConfig::default(),
            SsoSessionTimeoutConfig::default(),
            HashMap::new(),
        );
        let oidc_config = OidcProviderConfig::new(
            "https://idp.example.com".into(),
            "c".into(),
            "s".into(),
        );
        state.register_oidc_provider("google", oidc_config);
        assert!(state.oidc_validators.read().contains_key("google"));

        let saml_config = SamlAssertionConfig::new(
            "https://idp.example.com/metadata".into(),
            "https://idp.example.com/sso".into(),
            "cert".into(),
        );
        state.register_saml_provider("okta", saml_config);
        assert!(state.saml_validators.read().contains_key("okta"));
    }
}
