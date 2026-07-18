#![forbid(unsafe_code)]
#![allow(clippy::too_many_arguments)]

use crate::error::{DbError, Result};
use crate::models::{Org, Team, TeamMember, User};
use chrono::{DateTime, Utc};
use uuid::Uuid;

impl super::DbRepository {
    // --- Organizations ---

    pub async fn create_org(
        &self,
        name: &str,
        display_name: &str,
        description: &str,
        visibility: &str,
        owner_id: Uuid,
    ) -> Result<Org> {
        let row = sqlx::query_as::<_, Org>(
            r#"INSERT INTO organizations (name, display_name, description, visibility, owner_id)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(name)
        .bind(display_name)
        .bind(description)
        .bind(visibility)
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_org: {e}")))?;
        Ok(row)
    }

    pub async fn get_org(&self, id: Uuid) -> Result<Org> {
        sqlx::query_as::<_, Org>("SELECT * FROM organizations WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_org: {e}")))
    }

    pub async fn list_orgs_by_owner(&self, owner_id: Uuid) -> Result<Vec<Org>> {
        let rows = sqlx::query_as::<_, Org>("SELECT * FROM organizations WHERE owner_id = $1")
            .bind(owner_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_orgs_by_owner: {e}")))?;
        Ok(rows)
    }

    pub async fn list_all_orgs(&self) -> Result<Vec<Org>> {
        let rows = sqlx::query_as::<_, Org>("SELECT * FROM organizations ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_all_orgs: {e}")))?;
        Ok(rows)
    }

    pub async fn update_org(
        &self,
        id: Uuid,
        display_name: Option<&str>,
        description: Option<&str>,
        visibility: Option<&str>,
    ) -> Result<Org> {
        let row = sqlx::query_as::<_, Org>(
            r#"UPDATE organizations
               SET display_name = COALESCE($2, display_name),
                   description  = COALESCE($3, description),
                   visibility   = COALESCE($4, visibility),
                   updated_at   = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(display_name)
        .bind(description)
        .bind(visibility)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_org: {e}")))?;
        Ok(row)
    }

    // --- Teams ---

    pub async fn create_team(
        &self,
        org_id: Uuid,
        name: &str,
        description: &str,
        privacy: &str,
    ) -> Result<Team> {
        // TECH DEBT: Inline DDL — should be moved to a proper migration.
        // Uses IF NOT EXISTS to avoid conflicts; safe to remove once migration exists.
        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS teams (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                privacy TEXT NOT NULL DEFAULT 'visible',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(org_id, name)
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_team table: {e}")))?;

        let row = sqlx::query_as::<_, Team>(
            r#"INSERT INTO teams (org_id, name, description, privacy)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(org_id)
        .bind(name)
        .bind(description)
        .bind(privacy)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_team: {e}")))?;
        Ok(row)
    }

    pub async fn get_team(&self, id: Uuid) -> Result<Team> {
        sqlx::query_as::<_, Team>("SELECT * FROM teams WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_team: {e}")))
    }

    pub async fn list_teams(&self, org_id: Uuid) -> Result<Vec<Team>> {
        let rows = sqlx::query_as::<_, Team>("SELECT * FROM teams WHERE org_id = $1 ORDER BY name")
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_teams: {e}")))?;
        Ok(rows)
    }

    pub async fn update_team(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        privacy: Option<&str>,
    ) -> Result<Team> {
        let row = sqlx::query_as::<_, Team>(
            r#"UPDATE teams
               SET name        = COALESCE($2, name),
                   description = COALESCE($3, description),
                   privacy     = COALESCE($4, privacy),
                   updated_at  = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(privacy)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_team: {e}")))?;
        Ok(row)
    }

    pub async fn delete_team(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM teams WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_team: {e}")))?;
        Ok(())
    }

    pub async fn add_team_member(
        &self,
        team_id: Uuid,
        user_id: Uuid,
        role: &str,
    ) -> Result<TeamMember> {
        // TECH DEBT: Inline DDL — should be moved to a proper migration.
        // Uses IF NOT EXISTS to avoid conflicts; safe to remove once migration exists.
        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS team_members (
                team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                role TEXT NOT NULL DEFAULT 'member',
                joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (team_id, user_id)
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("add_team_member table: {e}")))?;

        let row = sqlx::query_as::<_, TeamMember>(
            r#"INSERT INTO team_members (team_id, user_id, role)
               VALUES ($1, $2, $3)
               ON CONFLICT (team_id, user_id) DO UPDATE SET role = $3
               RETURNING *"#,
        )
        .bind(team_id)
        .bind(user_id)
        .bind(role)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("add_team_member: {e}")))?;
        Ok(row)
    }

    pub async fn remove_team_member(&self, team_id: Uuid, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM team_members WHERE team_id = $1 AND user_id = $2")
            .bind(team_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("remove_team_member: {e}")))?;
        Ok(())
    }

    pub async fn list_team_members(&self, team_id: Uuid) -> Result<Vec<TeamMember>> {
        let rows = sqlx::query_as::<_, TeamMember>(
            "SELECT * FROM team_members WHERE team_id = $1 ORDER BY joined_at",
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_team_members: {e}")))?;
        Ok(rows)
    }

    // --- Audit Log Admin ---

    #[allow(clippy::too_many_arguments)]
    pub async fn query_audit_events_admin(
        &self,
        actor_id: Option<Uuid>,
        action: Option<&str>,
        resource_type: Option<&str>,
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<
        Vec<(
            i64,
            Uuid,
            String,
            String,
            Option<Uuid>,
            Option<String>,
            Option<String>,
            String,
            Option<Uuid>,
            DateTime<Utc>,
        )>,
    > {
        let rows = sqlx::query_as(
            r#"SELECT id, actor_id, action, resource_type, resource_id, ip_address, user_agent, outcome, request_id, created_at
               FROM audit_events
               WHERE ($1::uuid IS NULL OR actor_id = $1)
                 AND ($2::varchar IS NULL OR action = $2)
                 AND ($3::varchar IS NULL OR resource_type = $3)
                 AND ($4::timestamptz IS NULL OR created_at >= $4)
                 AND ($5::timestamptz IS NULL OR created_at <= $5)
               ORDER BY created_at DESC
               LIMIT $6 OFFSET $7"#,
        )
        .bind(actor_id)
        .bind(action)
        .bind(resource_type)
        .bind(since)
        .bind(until)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("query_audit_events_admin: {e}")))?;
        Ok(rows)
    }

    pub async fn audit_event_stats(
        &self,
    ) -> Result<(
        i64,
        Vec<(String, i64)>,
        Vec<(Uuid, i64)>,
        Vec<(String, i64)>,
    )> {
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM audit_events")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("audit_event_stats total: {e}")))?;

        let per_day: Vec<(String, i64)> = sqlx::query_as(
            r#"SELECT DATE(created_at) as date, COUNT(*) as count
               FROM audit_events
               WHERE created_at > NOW() - INTERVAL '30 days'
               GROUP BY DATE(created_at)
               ORDER BY date DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("audit_event_stats per_day: {e}")))?;

        let top_actors: Vec<(Uuid, i64)> = sqlx::query_as(
            r#"SELECT actor_id, COUNT(*) as count
               FROM audit_events
               WHERE created_at > NOW() - INTERVAL '30 days'
               GROUP BY actor_id
               ORDER BY count DESC
               LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("audit_event_stats top_actors: {e}")))?;

        let top_actions: Vec<(String, i64)> = sqlx::query_as(
            r#"SELECT action, COUNT(*) as count
               FROM audit_events
               WHERE created_at > NOW() - INTERVAL '30 days'
               GROUP BY action
               ORDER BY count DESC
               LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("audit_event_stats top_actions: {e}")))?;

        Ok((total.0, per_day, top_actors, top_actions))
    }

    // --- Org Members ---

    pub async fn list_org_members(&self, org_id: Uuid) -> Result<Vec<User>> {
        let rows = sqlx::query_as::<_, User>(
            r#"SELECT u.* FROM users u
               INNER JOIN org_members om ON om.user_id = u.id
               WHERE om.org_id = $1
               ORDER BY u.username"#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_org_members: {e}")))?;
        Ok(rows)
    }

    // --- Admin: ban/unban user ---

    pub async fn ban_user(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE users SET banned = true, updated_at = NOW() WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("ban_user: {e}")))?;
        Ok(())
    }

    pub async fn unban_user(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE users SET banned = false, updated_at = NOW() WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("unban_user: {e}")))?;
        Ok(())
    }

}
