#![forbid(unsafe_code)]
#![allow(clippy::too_many_arguments)]

use crate::error::{DbError, Result};
use crate::models::{SshKey, User};
use sqlx::postgres::PgPool;
use uuid::Uuid;

impl super::DbRepository {
    // --- Users ---

    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        display_name: &str,
        role: &str,
        password_hash: &str,
    ) -> Result<User> {
        let row = sqlx::query_as::<_, User>(
            r#"INSERT INTO users (username, email, display_name, role, password_hash)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(username)
        .bind(email)
        .bind(display_name)
        .bind(role)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_user: {e}")))?;
        Ok(row)
    }

    pub async fn get_user_by_id(&self, id: Uuid) -> Result<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_user_by_id: {e}")))
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_user_by_username: {e}")))
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_user_by_email: {e}")))
    }

    pub async fn update_user(
        &self,
        id: Uuid,
        display_name: Option<&str>,
        bio: Option<&str>,
        role: Option<&str>,
    ) -> Result<User> {
        let row = sqlx::query_as::<_, User>(
            r#"UPDATE users
               SET display_name  = COALESCE($2, display_name),
                   bio           = COALESCE($3, bio),
                   role          = COALESCE($4, role),
                   updated_at    = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(display_name)
        .bind(bio)
        .bind(role)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_user: {e}")))?;
        Ok(row)
    }

    pub async fn update_user_profile(
        &self,
        id: Uuid,
        avatar_url: Option<&str>,
        location: Option<&str>,
        website: Option<&str>,
        display_name: Option<&str>,
        bio: Option<&str>,
    ) -> Result<User> {
        let row = sqlx::query_as::<_, User>(
            r#"UPDATE users
               SET avatar_url    = COALESCE($2, avatar_url),
                   location      = COALESCE($3, location),
                   website       = COALESCE($4, website),
                   display_name  = COALESCE($5, display_name),
                   bio           = COALESCE($6, bio),
                   updated_at    = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(avatar_url)
        .bind(location)
        .bind(website)
        .bind(display_name)
        .bind(bio)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_user_profile: {e}")))?;
        Ok(row)
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_user: {e}")))?;
        Ok(())
    }

    pub async fn list_users(&self, limit: i64, offset: i64) -> Result<Vec<User>> {
        let rows = sqlx::query_as::<_, User>(
            "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_users: {e}")))?;
        Ok(rows)
    }

    // --- SSH Keys ---

    pub async fn add_ssh_key(
        &self,
        user_id: Uuid,
        key_type: &str,
        public_key: &str,
        fingerprint: &str,
        label: &str,
    ) -> Result<SshKey> {
        let row = sqlx::query_as::<_, SshKey>(
            r#"INSERT INTO ssh_keys (user_id, key_type, public_key, fingerprint, label)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(key_type)
        .bind(public_key)
        .bind(fingerprint)
        .bind(label)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("add_ssh_key: {e}")))?;
        Ok(row)
    }

    pub async fn list_ssh_keys(&self, user_id: Uuid) -> Result<Vec<SshKey>> {
        let rows = sqlx::query_as::<_, SshKey>(
            "SELECT * FROM ssh_keys WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_ssh_keys: {e}")))?;
        Ok(rows)
    }

    pub async fn delete_ssh_key(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM ssh_keys WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_ssh_key: {e}")))?;
        Ok(())
    }

    pub async fn get_ssh_key_by_fingerprint(&self, fingerprint: &str) -> Result<SshKey> {
        sqlx::query_as::<_, SshKey>("SELECT * FROM ssh_keys WHERE fingerprint = $1")
            .bind(fingerprint)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_ssh_key_by_fingerprint: {e}")))
    }

    // --- Password ---

    pub async fn get_password_hash(&self, user_id: Uuid) -> Result<Option<String>> {
        let row: (Option<String>,) =
            sqlx::query_as("SELECT password_hash FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| DbError::Database(format!("get_password_hash: {e}")))?;
        Ok(row.0)
    }

    pub async fn change_password(&self, user_id: Uuid, password_hash: &str) -> Result<()> {
        sqlx::query(r#"UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2"#)
            .bind(password_hash)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("change_password: {e}")))?;
        Ok(())
    }

    // --- Login Attempts / Lockout ---

    pub async fn record_login_attempt(
        &self,
        username: &str,
        ip: &str,
        success: bool,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO login_attempts (username, ip_address, success) VALUES ($1, $2, $3)",
        )
        .bind(username)
        .bind(ip)
        .bind(success)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_login_attempt: {e}")))?;
        sqlx::query("DELETE FROM login_attempts WHERE created_at < NOW() - INTERVAL '24 hours'")
            .execute(&self.pool)
            .await
            .ok();
        Ok(())
    }

    pub async fn count_recent_failed_logins(
        &self,
        username: &str,
        window_secs: i64,
    ) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM login_attempts WHERE username = $1 AND success = false AND created_at > NOW() - ($2 || ' seconds')::INTERVAL",
        )
        .bind(username)
        .bind(window_secs.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("count_recent_failed_logins: {e}")))?;
        Ok(row.0)
    }

    pub async fn clear_login_attempts(&self, username: &str) -> Result<()> {
        sqlx::query("DELETE FROM login_attempts WHERE username = $1 AND success = false")
            .bind(username)
            .execute(&self.pool)
            .await
            .ok();
        Ok(())
    }

    // --- Stars ---

    pub async fn has_user_starred(&self, user_id: Uuid, repo_id: Uuid) -> Result<bool> {
        let row: Option<(bool,)> = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM repo_stars WHERE user_id = $1 AND repo_id = $2)",
        )
        .bind(user_id)
        .bind(repo_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("has_user_starred: {e}")))?;
        Ok(row.map(|(v,)| v).unwrap_or(false))
    }

    pub async fn toggle_star(&self, user_id: Uuid, repo_id: Uuid) -> Result<(i64, bool)> {
        let existing = self.has_user_starred(user_id, repo_id).await?;
        if existing {
            sqlx::query("DELETE FROM repo_stars WHERE user_id = $1 AND repo_id = $2")
                .bind(user_id)
                .bind(repo_id)
                .execute(&self.pool)
                .await
                .map_err(|e| DbError::Database(format!("toggle_star delete: {e}")))?;
            let row: (i64,) = sqlx::query_as(
                "UPDATE repositories SET stars_count = GREATEST(stars_count - 1, 0), updated_at = NOW() WHERE id = $1 RETURNING stars_count",
            )
            .bind(repo_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("toggle_star dec: {e}")))?;
            Ok((row.0, false))
        } else {
            sqlx::query("INSERT INTO repo_stars (user_id, repo_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(user_id)
                .bind(repo_id)
                .execute(&self.pool)
                .await
                .map_err(|e| DbError::Database(format!("toggle_star insert: {e}")))?;
            let row: (i64,) = sqlx::query_as(
                "UPDATE repositories SET stars_count = stars_count + 1, updated_at = NOW() WHERE id = $1 RETURNING stars_count",
            )
            .bind(repo_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("toggle_star inc: {e}")))?;
            Ok((row.0, true))
        }
    }

    pub async fn increment_stars(&self, repo_id: Uuid) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "UPDATE repositories SET stars_count = stars_count + 1, updated_at = NOW() WHERE id = $1 RETURNING stars_count",
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("increment_stars: {e}")))?;
        Ok(row.0)
    }

    pub async fn decrement_stars(&self, repo_id: Uuid) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "UPDATE repositories SET stars_count = GREATEST(stars_count - 1, 0), updated_at = NOW() WHERE id = $1 RETURNING stars_count",
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("decrement_stars: {e}")))?;
        Ok(row.0)
    }

    // --- Watchers ---

    pub async fn has_user_watched(&self, user_id: Uuid, repo_id: Uuid) -> Result<bool> {
        let row: Option<(bool,)> = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM repo_watchers WHERE user_id = $1 AND repo_id = $2)",
        )
        .bind(user_id)
        .bind(repo_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("has_user_watched: {e}")))?;
        Ok(row.map(|(v,)| v).unwrap_or(false))
    }

    pub async fn toggle_watch(&self, user_id: Uuid, repo_id: Uuid) -> Result<(i64, bool)> {
        let existing = self.has_user_watched(user_id, repo_id).await?;
        if existing {
            sqlx::query("DELETE FROM repo_watchers WHERE user_id = $1 AND repo_id = $2")
                .bind(user_id)
                .bind(repo_id)
                .execute(&self.pool)
                .await
                .map_err(|e| DbError::Database(format!("toggle_watch delete: {e}")))?;
            let row: (i64,) = sqlx::query_as(
                "UPDATE repositories SET watchers_count = GREATEST(watchers_count - 1, 0), updated_at = NOW() WHERE id = $1 RETURNING watchers_count",
            )
            .bind(repo_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("toggle_watch dec: {e}")))?;
            Ok((row.0, false))
        } else {
            sqlx::query("INSERT INTO repo_watchers (user_id, repo_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(user_id)
                .bind(repo_id)
                .execute(&self.pool)
                .await
                .map_err(|e| DbError::Database(format!("toggle_watch insert: {e}")))?;
            let row: (i64,) = sqlx::query_as(
                "UPDATE repositories SET watchers_count = watchers_count + 1, updated_at = NOW() WHERE id = $1 RETURNING watchers_count",
            )
            .bind(repo_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("toggle_watch inc: {e}")))?;
            Ok((row.0, true))
        }
    }

    pub async fn increment_watchers(&self, repo_id: Uuid) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "UPDATE repositories SET watchers_count = watchers_count + 1, updated_at = NOW() WHERE id = $1 RETURNING watchers_count",
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("increment_watchers: {e}")))?;
        Ok(row.0)
    }

    pub async fn decrement_watchers(&self, repo_id: Uuid) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "UPDATE repositories SET watchers_count = GREATEST(watchers_count - 1, 0), updated_at = NOW() WHERE id = $1 RETURNING watchers_count",
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("decrement_watchers: {e}")))?;
        Ok(row.0)
    }

}
