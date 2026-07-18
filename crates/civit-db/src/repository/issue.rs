#![forbid(unsafe_code)]
#![allow(clippy::too_many_arguments)]

use crate::error::{DbError, Result};
use crate::models::Issue;
use regex;
use uuid::Uuid;

impl super::DbRepository {
    // --- Issues ---

    pub async fn create_issue(
        &self,
        repo_id: Uuid,
        title: &str,
        body: &str,
        author_id: Uuid,
    ) -> Result<Issue> {
        let row = sqlx::query_as::<_, Issue>(
            r#"INSERT INTO issues (repo_id, title, body, author_id)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(title)
        .bind(body)
        .bind(author_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_issue: {e}")))?;
        Ok(row)
    }

    pub async fn get_issue(&self, id: Uuid) -> Result<Issue> {
        sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_issue: {e}")))
    }

    pub async fn list_issues(&self, repo_id: Uuid, limit: i64, offset: i64) -> Result<Vec<Issue>> {
        let rows = sqlx::query_as::<_, Issue>(
            "SELECT * FROM issues WHERE repo_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(repo_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_issues: {e}")))?;
        Ok(rows)
    }

    pub async fn update_issue(
        &self,
        id: Uuid,
        title: Option<&str>,
        body: Option<&str>,
        status: Option<&str>,
        assignee_id: Option<Option<Uuid>>,
    ) -> Result<Issue> {
        let row = sqlx::query_as::<_, Issue>(
            r#"UPDATE issues
               SET title      = COALESCE($2, title),
                   body       = COALESCE($3, body),
                   status     = COALESCE($4, status),
                   assignee_id = COALESCE($5, assignee_id),
                   updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(title)
        .bind(body)
        .bind(status)
        .bind(assignee_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_issue: {e}")))?;
        Ok(row)
    }

    // --- Issue auto-close on PR merge ---

    pub async fn close_issues_for_pr(
        &self,
        repo_id: Uuid,
        pr_title: &str,
        pr_body: &str,
        actor_id: Uuid,
    ) -> Result<Vec<i32>> {
        let text = format!("{pr_title}\n{pr_body}");
        let re = regex::Regex::new(r"(?i)(?:fix(?:es|ed)?|closes?|resolves?)\s+#(\d+)").expect("valid regex pattern");
        let issue_numbers: Vec<i32> = re
            .captures_iter(&text)
            .filter_map(|c| c.get(1)?.as_str().parse::<i32>().ok())
            .collect();

        let mut closed = Vec::new();
        for num in &issue_numbers {
            let result: Option<(Uuid,)> = sqlx::query_as(
                r#"UPDATE issues
                   SET status = 'closed', closed_at = NOW(), updated_at = NOW()
                   WHERE repo_id = $1 AND number = $2 AND status != 'closed'
                   RETURNING id"#,
            )
            .bind(repo_id)
            .bind(num)
            .fetch_optional(&self.pool)
            .await
            .unwrap_or(None);
            if let Some((issue_id,)) = result {
                let _ = sqlx::query(
                    "INSERT INTO issue_timeline (issue_id, actor_id, event_type, event_detail, created_at) VALUES ($1, $2, 'closed_by_pr', $3, NOW())",
                )
                .bind(issue_id)
                .bind(actor_id)
                .bind("Closed by merge of PR")
                .execute(&self.pool)
                .await;
                closed.push(*num);
            }
        }
        Ok(closed)
    }

    // --- Issue Templates ---

    pub async fn list_issue_templates(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<crate::models::IssueTemplate>> {
        let rows = sqlx::query_as::<_, crate::models::IssueTemplate>(
            "SELECT id, repo_id, name, title, body, labels, created_at FROM issue_templates WHERE repo_id = $1 ORDER BY name",
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_issue_templates: {e}")))?;
        Ok(rows)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_issue_template(
        &self,
        repo_id: Uuid,
        name: &str,
        title: &str,
        body: &str,
        labels: &[String],
    ) -> Result<crate::models::IssueTemplate> {
        let row = sqlx::query_as::<_, crate::models::IssueTemplate>(
            "INSERT INTO issue_templates (repo_id, name, title, body, labels, created_at) VALUES ($1, $2, $3, $4, $5, NOW()) RETURNING id, repo_id, name, title, body, labels, created_at",
        )
        .bind(repo_id)
        .bind(name)
        .bind(title)
        .bind(body)
        .bind(labels)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_issue_template: {e}")))?;
        Ok(row)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_issue_template(
        &self,
        template_id: Uuid,
        repo_id: Uuid,
        name: Option<&str>,
        title: Option<&str>,
        body: Option<&str>,
        labels: Option<&[String]>,
    ) -> Result<crate::models::IssueTemplate> {
        let row = sqlx::query_as::<_, crate::models::IssueTemplate>(
            "UPDATE issue_templates SET name = COALESCE($3, name), title = COALESCE($4, title), body = COALESCE($5, body), labels = COALESCE($6, labels) WHERE id = $1 AND repo_id = $2 RETURNING id, repo_id, name, title, body, labels, created_at",
        )
        .bind(template_id)
        .bind(repo_id)
        .bind(name)
        .bind(title)
        .bind(body)
        .bind(labels)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_issue_template: {e}")))?;
        Ok(row)
    }

    pub async fn delete_issue_template(
        &self,
        template_id: Uuid,
        repo_id: Uuid,
    ) -> Result<()> {
        sqlx::query("DELETE FROM issue_templates WHERE id = $1 AND repo_id = $2")
            .bind(template_id)
            .bind(repo_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_issue_template: {e}")))?;
        Ok(())
    }


}
