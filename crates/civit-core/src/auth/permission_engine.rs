#![forbid(unsafe_code)]

//! Full permission engine with deny-overrides, role hierarchy, and branch protection.
//!
//! Permission evaluation order:
//! 1. Explicit deny on repo policy → DENY
//! 2. Explicit grant on repo policy → ALLOW
//! 3. Branch protection rule → may DENY
//! 4. Org-level role grant → ALLOW
//! 5. Role default grants → ALLOW
//! 6. Default → DENY
//!
//! Uses `sqlx::query` / `sqlx::query_as` (runtime) instead of compile-time macros
//! to avoid requiring `DATABASE_URL` at build time.

use chrono::Utc;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use tracing::{debug, instrument};

use crate::error::CoreError;
use civit_shared::id::{RepoId, UserId};
use civit_shared::permissions::{Action, PermissionCheck, PipelineVariableResponse, Resource};
use civit_shared::user::UserRole;

use ring::aead::{AES_256_GCM, LessSafeKey, Nonce, UnboundKey};
use ring::rand::SecureRandom;

// ---------------------------------------------------------------------------
// AES-256-GCM encryption for pipeline variables
// ---------------------------------------------------------------------------

/// Derive the AES-256-GCM encryption key.
/// Reads `CIVIT_ENCRYPTION_KEY` env var (64-char hex = 32 bytes).
/// Falls back to a deterministic key for backward compatibility with existing plaintext data.
/// Production deployments **MUST** set `CIVIT_ENCRYPTION_KEY` — a warning is logged at init.
pub fn get_encryption_key() -> Vec<u8> {
    // Try environment variable first
    if let Ok(key_hex) = std::env::var("CIVIT_ENCRYPTION_KEY") {
        if key_hex.len() == 64 {
            // 32 bytes = 256 bits, hex-encoded
            if let Ok(key_bytes) = hex_decode(&key_hex) {
                return key_bytes;
            }
        }
    }

    // Fallback: deterministic key — only for development / migration.
    tracing::warn!(
        "CIVIT_ENCRYPTION_KEY not set — using built-in fallback key. \
         Production deployments MUST set a 64-char hex key in CIVIT_ENCRYPTION_KEY."
    );
    b"civitforge-pipeline-variable-encryption-key-01!".to_vec()
}

fn hex_decode(hex: &str) -> Result<Vec<u8>, &'static str> {
    if hex.len() % 2 != 0 {
        return Err("hex string must have even length");
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| "invalid hex"))
        .collect()
}

/// Encrypt plaintext using AES-256-GCM. Returns (ciphertext_with_tag, nonce).
/// The nonce is 12 bytes, prepended to the ciphertext.
pub(crate) fn encrypt_value(plaintext: &str) -> Result<(Vec<u8>, Vec<u8>), CoreError> {
    let key_bytes = get_encryption_key();
    let key = UnboundKey::new(&AES_256_GCM, &key_bytes)
        .map_err(|_| CoreError::Internal("failed to create AES key".into()))?;
    let less_safe_key = LessSafeKey::new(key);

    // Generate random nonce
    let rng = ring::rand::SystemRandom::new();
    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes)
        .map_err(|_| CoreError::Internal("nonce generation failed".into()))?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    // Encrypt in-place: prefix with 128 bytes for tag
    let plaintext_bytes = plaintext.as_bytes();
    let mut ciphertext = vec![0u8; plaintext_bytes.len() + 16]; // 16 byte tag
    ciphertext[..plaintext_bytes.len()].copy_from_slice(plaintext_bytes);

    less_safe_key
        .seal_in_place_append_tag(nonce, ring::aead::Aad::empty(), &mut ciphertext)
        .map_err(|_| CoreError::Internal("AES-256-GCM encryption failed".into()))?;

    Ok((ciphertext, nonce_bytes.to_vec()))
}

/// Decrypt AES-256-GCM ciphertext. Expects nonce as separate parameter.
pub(crate) fn decrypt_value(ciphertext: &[u8], nonce: &[u8]) -> Result<String, CoreError> {
    // If nonce is all zeros, this is legacy plaintext data
    if nonce.iter().all(|&b| b == 0) {
        return Ok(String::from_utf8_lossy(ciphertext).into_owned());
    }

    let key_bytes = get_encryption_key();
    let key = UnboundKey::new(&AES_256_GCM, &key_bytes)
        .map_err(|_| CoreError::Internal("failed to create AES key".into()))?;
    let less_safe_key = LessSafeKey::new(key);

    let nonce = Nonce::try_assume_unique_for_key(nonce)
        .map_err(|_| CoreError::Internal("invalid nonce length".into()))?;

    let mut data = ciphertext.to_vec();
    let plaintext = less_safe_key
        .open_in_place(nonce, ring::aead::Aad::empty(), &mut data)
        .map_err(|_| CoreError::Internal("AES-256-GCM decryption failed (wrong key?)".into()))?;

    String::from_utf8(plaintext.to_vec())
        .map_err(|_| CoreError::Internal("decrypted value is not valid UTF-8".into()))
}

// ---------------------------------------------------------------------------
// Database models for permission tables
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MemberRoleRow {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub org_id: Option<uuid::Uuid>,
    pub repo_id: Option<uuid::Uuid>,
    pub role: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RepoPolicyRow {
    pub id: uuid::Uuid,
    pub repo_id: uuid::Uuid,
    pub role: String,
    pub resource: String,
    pub action: String,
    pub effect: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BranchProtectionRow {
    pub id: uuid::Uuid,
    pub repo_id: uuid::Uuid,
    pub pattern: String,
    pub push_restricted: bool,
    pub allowed_roles: serde_json::Value,
    pub required_reviews: Option<i32>,
    pub require_ci: bool,
    pub force_push_allowed: bool,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PipelineVariableRow {
    pub id: uuid::Uuid,
    pub repo_id: uuid::Uuid,
    pub name: String,
    pub value_enc: Vec<u8>,
    pub nonce: Vec<u8>,
    pub masked: bool,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Permission Engine
// ---------------------------------------------------------------------------

/// The permission engine evaluates (user, resource, action) → allowed/denied.
pub struct PermissionEngine;

impl PermissionEngine {
    /// Check if a user has permission on a resource.
    ///
    /// This is the main entry point for permission checks.
    #[instrument(skip_all)]
    pub async fn check(
        pool: &PgPool,
        user_id: UserId,
        resource: Resource,
        action: Action,
        repo_id: Option<RepoId>,
        org_id: Option<uuid::Uuid>,
        branch_name: Option<&str>,
    ) -> Result<PermissionCheck, CoreError> {
        let user_uuid = user_id.get();

        // System-level admins bypass all checks
        if Self::is_system_admin(pool, user_uuid).await? {
            return Ok(PermissionCheck::allowed(resource, action));
        }

        // 1. Check explicit repo-level denies
        if let Some(rid) = repo_id {
            if let Some(deny) =
                Self::check_repo_deny(pool, rid.get(), &resource, &action, user_uuid).await?
            {
                return Ok(PermissionCheck::denied(resource, action, deny));
            }
        }

        // 2. Check explicit repo-level grants
        if let Some(rid) = repo_id {
            if let Some(_grant) =
                Self::check_repo_grant(pool, rid.get(), &resource, &action, user_uuid).await?
            {
                return Ok(PermissionCheck::allowed(resource, action));
            }
        }

        // 3. Branch protection
        if resource == Resource::Branch && action == Action::Push {
            if let Some((rid, branch)) = repo_id.zip(branch_name) {
                if let Some(deny) =
                    Self::check_branch_protection(pool, rid.get(), branch, user_uuid).await?
                {
                    return Ok(PermissionCheck::denied(
                        Resource::Branch,
                        Action::Push,
                        deny,
                    ));
                }
            }
        }

        // 4. Check org-level role
        if let Some(oid) = org_id {
            if let Some(_grant) =
                Self::check_org_role(pool, oid, &resource, &action, user_uuid).await?
            {
                return Ok(PermissionCheck::allowed(resource, action));
            }
        }

        // 5. Fallback: check repo role (from member_roles if user is a direct member)
        if let Some(rid) = repo_id {
            if let Some(_grant) =
                Self::check_repo_role(pool, rid.get(), &resource, &action, user_uuid).await?
            {
                return Ok(PermissionCheck::allowed(resource, action));
            }
        }

        // 6. Default deny
        Ok(PermissionCheck::denied(
            resource,
            action,
            "No matching permission grant found",
        ))
    }

    /// Check if user is a system-level admin (org owner or has admin role anywhere).
    async fn is_system_admin(pool: &PgPool, user_id: uuid::Uuid) -> Result<bool, CoreError> {
        // Check if user owns any org (system-level)
        let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM organizations WHERE owner_id = $1)")
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;
        if row.get::<bool, _>(0) {
            return Ok(true);
        }

        // Check if user has admin role anywhere
        let row = sqlx::query(
            "SELECT EXISTS(SELECT 1 FROM member_roles WHERE user_id = $1 AND role = 'admin')",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;
        Ok(row.get::<bool, _>(0))
    }

    /// Check explicit deny on repo policy.
    async fn check_repo_deny(
        pool: &PgPool,
        repo_id: uuid::Uuid,
        resource: &Resource,
        action: &Action,
        user_id: uuid::Uuid,
    ) -> Result<Option<String>, CoreError> {
        let role = Self::get_most_privileged_role(pool, repo_id, user_id, true).await?;
        let role_str = role.to_string();

        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(
                SELECT 1 FROM repo_policies
                WHERE repo_id = $1 AND role = $2 AND resource = $3 AND action = $4 AND effect = 'deny'
            )"
        )
        .bind(repo_id)
        .bind(&role_str)
        .bind(resource.as_str())
        .bind(action.as_str())
        .fetch_one(pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        if exists {
            Ok(Some(format!(
                "Denied by repo policy: {resource:?} {action:?} for role {role}"
            )))
        } else {
            Ok(None)
        }
    }

    /// Check explicit grant on repo policy.
    async fn check_repo_grant(
        pool: &PgPool,
        repo_id: uuid::Uuid,
        resource: &Resource,
        action: &Action,
        user_id: uuid::Uuid,
    ) -> Result<Option<String>, CoreError> {
        let role = Self::get_most_privileged_role(pool, repo_id, user_id, true).await?;
        let role_str = role.to_string();

        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(
                SELECT 1 FROM repo_policies
                WHERE repo_id = $1 AND role = $2 AND resource = $3 AND action = $4 AND effect = 'grant'
            )"
        )
        .bind(repo_id)
        .bind(&role_str)
        .bind(resource.as_str())
        .bind(action.as_str())
        .fetch_one(pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        if exists {
            Ok(Some(format!(
                "Granted by repo policy: {resource:?} {action:?} for role {role}"
            )))
        } else {
            Ok(None)
        }
    }

    /// Check branch protection rules for a push.
    async fn check_branch_protection(
        pool: &PgPool,
        repo_id: uuid::Uuid,
        branch: &str,
        user_id: uuid::Uuid,
    ) -> Result<Option<String>, CoreError> {
        let rules = sqlx::query_as::<_, BranchProtectionRow>(
            "SELECT id, repo_id, pattern, push_restricted, allowed_roles,
                    required_reviews, require_ci, force_push_allowed, created_at
             FROM branch_protections WHERE repo_id = $1",
        )
        .bind(repo_id)
        .fetch_all(pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        let matching = rules.iter().filter(|r| pattern_matches(&r.pattern, branch));

        let mut violations = Vec::new();
        for rule in matching {
            if rule.push_restricted {
                let role = Self::get_most_privileged_role(pool, repo_id, user_id, true).await?;
                let allowed: Vec<String> =
                    serde_json::from_value(rule.allowed_roles.clone()).unwrap_or_default();

                if !allowed.iter().any(|a| a == &role.to_string()) {
                    violations.push(format!(
                        "Branch '{branch}' requires one of roles {allowed:?}, user has '{role}'",
                    ));
                }
            }
            if let Some(required) = rule.required_reviews {
                violations.push(format!(
                    "Branch '{branch}' requires {required} approvals before merge",
                ));
            }
            if rule.require_ci {
                violations.push(format!(
                    "Branch '{branch}' requires CI to pass before merge",
                ));
            }
            if !rule.force_push_allowed {
                violations.push(format!("Branch '{branch}' does not allow force-push"));
            }
        }

        if violations.is_empty() {
            Ok(None) // No violations — push allowed
        } else {
            Ok(Some(violations.join("; "))) // Violations found — push blocked
        }
    }

    /// Check org-level role permissions.
    async fn check_org_role(
        pool: &PgPool,
        org_id: uuid::Uuid,
        resource: &Resource,
        action: &Action,
        user_id: uuid::Uuid,
    ) -> Result<Option<String>, CoreError> {
        let role = Self::get_org_role(pool, org_id, user_id).await?;

        let role_rank = role.as_ref().map(|r| r.rank()).unwrap_or(0);
        let required_rank = Self::min_role_rank_for(resource, action);

        if role_rank >= required_rank {
            Ok(Some(format!(
                "Granted by org role '{}': {resource:?} {action:?}",
                role.as_ref()
                    .map(|r| r.to_string())
                    .unwrap_or_else(|| "none".into())
            )))
        } else {
            Ok(None)
        }
    }

    /// Check repo-level role permissions.
    async fn check_repo_role(
        pool: &PgPool,
        repo_id: uuid::Uuid,
        resource: &Resource,
        action: &Action,
        user_id: uuid::Uuid,
    ) -> Result<Option<String>, CoreError> {
        let role = Self::get_most_privileged_role(pool, repo_id, user_id, true).await?;

        let role_rank = role.rank();
        let required_rank = Self::min_role_rank_for(resource, action);

        if role_rank >= required_rank {
            Ok(Some(format!(
                "Granted by repo role '{role}': {resource:?} {action:?}",
            )))
        } else {
            Ok(None)
        }
    }

    /// Get the most privileged role a user has on a repo.
    /// If `indirect` is true, also falls back to org-level roles.
    async fn get_most_privileged_role(
        pool: &PgPool,
        repo_id: uuid::Uuid,
        user_id: uuid::Uuid,
        indirect: bool,
    ) -> Result<UserRole, CoreError> {
        // Direct repo role
        if let Some(role) = Self::get_repo_role(pool, repo_id, user_id).await? {
            return Ok(role);
        }

        // Check org-level role if indirect
        if indirect {
            if let Some(org_id) = Self::get_repo_org_id(pool, repo_id).await? {
                if let Some(org_role) = Self::get_org_role(pool, org_id, user_id).await? {
                    return Ok(org_role);
                }
            }
        }

        // Default: Guest
        Ok(UserRole::Guest)
    }

    /// Get a user's role on a specific repo.
    async fn get_repo_role(
        pool: &PgPool,
        repo_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<Option<UserRole>, CoreError> {
        let row: Option<PgRow> =
            sqlx::query("SELECT role FROM member_roles WHERE repo_id = $1 AND user_id = $2")
                .bind(repo_id)
                .bind(user_id)
                .fetch_optional(pool)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?;

        match row {
            Some(r) => {
                let role_str: Option<String> = r.get(0);
                match role_str {
                    Some(r) => {
                        let role: UserRole = r.parse().unwrap_or(UserRole::Guest);
                        debug!(user_id = %user_id, repo_id = %repo_id, ?role, "direct repo role");
                        Ok(Some(role))
                    }
                    None => Ok(None),
                }
            }
            None => {
                debug!(user_id = %user_id, repo_id = %repo_id, "no direct repo role");
                Ok(None)
            }
        }
    }

    /// Get the org that owns a repo.
    async fn get_repo_org_id(
        pool: &PgPool,
        repo_id: uuid::Uuid,
    ) -> Result<Option<uuid::Uuid>, CoreError> {
        let row: Option<PgRow> = sqlx::query("SELECT org_id FROM repositories WHERE id = $1")
            .bind(repo_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(r.get::<Option<uuid::Uuid>, _>(0)),
            None => Ok(None),
        }
    }

    /// Get a user's role within an org.
    async fn get_org_role(
        pool: &PgPool,
        org_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<Option<UserRole>, CoreError> {
        let row: Option<PgRow> =
            sqlx::query("SELECT role FROM member_roles WHERE org_id = $1 AND user_id = $2")
                .bind(org_id)
                .bind(user_id)
                .fetch_optional(pool)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?;

        match row {
            Some(r) => {
                let role_str: Option<String> = r.get(0);
                match role_str {
                    Some(r) => {
                        let role: UserRole = r.parse().unwrap_or(UserRole::Guest);
                        debug!(user_id = %user_id, org_id = %org_id, ?role, "org role");
                        Ok(Some(role))
                    }
                    None => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    /// Get the minimum role rank required for a resource+action pair.
    fn min_role_rank_for(resource: &Resource, action: &Action) -> u8 {
        let base = match action {
            // Read operations: Reporter (20)
            Action::Read => UserRole::Reporter.rank(),

            // Write operations: Developer (30)
            Action::Create => UserRole::Developer.rank(),
            Action::Update => UserRole::Developer.rank(),
            Action::Push => UserRole::Developer.rank(),
            Action::Merge => UserRole::Developer.rank(),
            Action::Rebase => UserRole::Developer.rank(),

            // Admin operations: Admin (50)
            Action::Administer => UserRole::Admin.rank(),
            Action::ManageWebhooks => UserRole::Admin.rank(),
            Action::ManageMembers => UserRole::Admin.rank(),

            // Dangerous operations: Admin (50)
            Action::Delete => UserRole::Admin.rank(),
            Action::Transfer => UserRole::Admin.rank(),

            // Special cases
            Action::ForcePush => UserRole::Admin.rank(),
            Action::PublishPackage => UserRole::Maintainer.rank(),
            Action::DownloadArtifact => UserRole::Reporter.rank(),
            Action::TriggerPipeline => UserRole::Developer.rank(),
            Action::CancelPipeline => UserRole::Developer.rank(),
            Action::ManageVariables => UserRole::Maintainer.rank(),
            Action::ManageRunners => UserRole::Admin.rank(),
            Action::Fork => UserRole::Developer.rank(),
        };

        // Some resources require higher privileges regardless of action
        let resource_boost = match resource {
            Resource::Runner => 10, // Admin only
            Resource::User => 10,   // Admin only
            Resource::Package => 5, // Maintainer for publish
            _ => 0,
        };

        (base + resource_boost).min(UserRole::Admin.rank())
    }
}

/// Check if a glob pattern matches a branch name.
/// Supports: exact match, "release/*" (single-level glob), and "v*" prefix.
fn pattern_matches(pattern: &str, branch: &str) -> bool {
    if pattern == branch {
        return true;
    }

    // Wildcard suffix: "release/*" matches "release/1.0" but not "release/1.0/extra"
    if let Some(rest) = pattern.strip_suffix("/*") {
        if let Some(after) = branch.strip_prefix(rest) {
            return after.starts_with('/') && !after[1..].contains('/');
        }
        return false;
    }

    // Prefix match: "v*" matches "v1.0", "v2.1.0"
    if let Some(prefix) = pattern.strip_suffix('*') {
        return branch.starts_with(prefix);
    }

    false
}

// ---------------------------------------------------------------------------
// Pipeline Variable Management
// ---------------------------------------------------------------------------

impl PermissionEngine {
    /// List pipeline variables for a repo (masked values for non-admins).
    #[instrument(skip_all)]
    pub async fn list_pipeline_variables(
        pool: &PgPool,
        repo_id: uuid::Uuid,
        _user_id: UserId,
        _is_admin: bool,
    ) -> Result<Vec<PipelineVariableResponse>, CoreError> {
        let rows = sqlx::query_as::<_, PipelineVariableRow>(
            "SELECT id, repo_id, name, value_enc, nonce, masked, created_at, updated_at
             FROM pipeline_variables WHERE repo_id = $1 ORDER BY name",
        )
        .bind(repo_id)
        .fetch_all(pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| PipelineVariableResponse {
                id: row.id,
                repo_id: RepoId::new(row.repo_id),
                name: row.name,
                masked: row.masked,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect())
    }

    /// Create or update a pipeline variable.
    #[instrument(skip_all)]
    pub async fn upsert_pipeline_variable(
        pool: &PgPool,
        repo_id: uuid::Uuid,
        user_id: UserId,
        name: &str,
        value: &str,
        masked: bool,
    ) -> Result<PipelineVariableResponse, CoreError> {
        // Permission check: only Maintainer+ can manage variables
        let check = Self::check(
            pool,
            user_id,
            Resource::PipelineVariable,
            Action::ManageVariables,
            Some(RepoId::new(repo_id)),
            None,
            None,
        )
        .await?;

        if !check.allowed {
            return Err(CoreError::Forbidden(
                "Insufficient permissions to manage pipeline variables".into(),
            ));
        }

        // AES-256-GCM encrypt the variable value
        let (value_enc, nonce) = encrypt_value(value)?;

        let id = sqlx::query_scalar::<_, Option<uuid::Uuid>>(
            "INSERT INTO pipeline_variables (repo_id, name, value_enc, nonce, masked)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (repo_id, name)
             DO UPDATE SET value_enc = $3, nonce = $4, masked = $5, updated_at = now()
             RETURNING id",
        )
        .bind(repo_id)
        .bind(name)
        .bind(&value_enc)
        .bind(&nonce)
        .bind(masked)
        .fetch_one(pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?
        .unwrap_or(uuid::Uuid::nil());

        Ok(PipelineVariableResponse {
            id,
            repo_id: RepoId::new(repo_id),
            name: name.to_string(),
            masked,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Delete a pipeline variable.
    #[instrument(skip_all)]
    pub async fn delete_pipeline_variable(
        pool: &PgPool,
        repo_id: uuid::Uuid,
        user_id: UserId,
        variable_id: uuid::Uuid,
    ) -> Result<(), CoreError> {
        let check = Self::check(
            pool,
            user_id,
            Resource::PipelineVariable,
            Action::Delete,
            Some(RepoId::new(repo_id)),
            None,
            None,
        )
        .await?;

        if !check.allowed {
            return Err(CoreError::Forbidden(
                "Insufficient permissions to delete pipeline variables".into(),
            ));
        }

        let result = sqlx::query("DELETE FROM pipeline_variables WHERE id = $1 AND repo_id = $2")
            .bind(variable_id)
            .bind(repo_id)
            .execute(pool)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound("Pipeline variable not found".into()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_exact_match() {
        assert!(pattern_matches("main", "main"));
    }

    #[test]
    fn pattern_wildcard_single_level() {
        assert!(pattern_matches("release/*", "release/1.0"));
        assert!(pattern_matches("release/*", "release/2.0"));
        assert!(!pattern_matches("release/*", "release/1.0/extra"));
        assert!(!pattern_matches("release/*", "hotfix/1.0"));
        assert!(!pattern_matches("release/*", "main"));
    }

    #[test]
    fn pattern_prefix() {
        assert!(pattern_matches("v*", "v1.0"));
        assert!(pattern_matches("v*", "v2.1.0"));
        assert!(!pattern_matches("v*", "main"));
    }

    #[test]
    fn min_role_rank_table() {
        // Reporters can read
        assert!(
            UserRole::Reporter.rank()
                >= PermissionEngine::min_role_rank_for(&Resource::Repository, &Action::Read)
        );
        assert!(
            UserRole::Guest.rank()
                < PermissionEngine::min_role_rank_for(&Resource::Repository, &Action::Read)
        );

        // Developers can push
        assert!(
            UserRole::Developer.rank()
                >= PermissionEngine::min_role_rank_for(&Resource::Repository, &Action::Push)
        );
        assert!(
            UserRole::Reporter.rank()
                < PermissionEngine::min_role_rank_for(&Resource::Repository, &Action::Push)
        );

        // Admins can delete
        assert!(
            UserRole::Admin.rank()
                >= PermissionEngine::min_role_rank_for(&Resource::Repository, &Action::Delete)
        );
        assert!(
            UserRole::Maintainer.rank()
                < PermissionEngine::min_role_rank_for(&Resource::Repository, &Action::Delete)
        );

        // ForcePush requires admin
        assert_eq!(
            PermissionEngine::min_role_rank_for(&Resource::Branch, &Action::ForcePush),
            UserRole::Admin.rank()
        );

        // Fork is developer-level
        assert!(
            PermissionEngine::min_role_rank_for(&Resource::Repository, &Action::Fork)
                <= UserRole::Developer.rank()
        );

        // Runner management is admin-only
        assert_eq!(
            PermissionEngine::min_role_rank_for(&Resource::Runner, &Action::ManageRunners),
            UserRole::Admin.rank()
        );
    }

    #[test]
    fn user_role_from_str() {
        assert_eq!("owner".parse::<UserRole>().unwrap(), UserRole::Owner);
        assert_eq!("admin".parse::<UserRole>().unwrap(), UserRole::Admin);
        assert_eq!(
            "maintainer".parse::<UserRole>().unwrap(),
            UserRole::Maintainer
        );
        assert_eq!(
            "developer".parse::<UserRole>().unwrap(),
            UserRole::Developer
        );
        assert_eq!("reporter".parse::<UserRole>().unwrap(), UserRole::Reporter);
        assert_eq!("guest".parse::<UserRole>().unwrap(), UserRole::Guest);
        assert!("unknown".parse::<UserRole>().is_err());
    }

    #[test]
    fn user_role_roundtrip() {
        for role in [
            UserRole::Owner,
            UserRole::Admin,
            UserRole::Maintainer,
            UserRole::Developer,
            UserRole::Reporter,
            UserRole::Guest,
        ] {
            let s = role.to_string();
            let back: UserRole = s.parse().unwrap();
            assert_eq!(role, back);
        }
    }
}
