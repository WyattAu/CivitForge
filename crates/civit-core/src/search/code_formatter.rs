#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CodeFormatter {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub language: String,
    pub formatter: String,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatRequest {
    pub file_path: String,
    pub content: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatResult {
    pub formatted_content: String,
    pub changed: bool,
    pub formatter: String,
}

pub struct CodeFormatterService {
    db: sqlx::PgPool,
}

impl CodeFormatterService {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn get_formatter(
        &self,
        repo_id: Uuid,
        language: &str,
    ) -> Result<Option<CodeFormatter>, sqlx::Error> {
        let formatter = sqlx::query_as::<_, CodeFormatter>(
            r#"
            SELECT * FROM code_formatters
            WHERE repo_id = $1 AND language = $2 AND enabled = true
            "#,
        )
        .bind(repo_id)
        .bind(language)
        .fetch_optional(&self.db)
        .await?;

        Ok(formatter)
    }

    pub async fn set_formatter(
        &self,
        repo_id: Uuid,
        language: &str,
        formatter: &str,
        config: serde_json::Value,
    ) -> Result<CodeFormatter, sqlx::Error> {
        let id = Uuid::new_v4();
        let code_formatter = sqlx::query_as::<_, CodeFormatter>(
            r#"
            INSERT INTO code_formatters (id, repo_id, language, formatter, config)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (repo_id, language) DO UPDATE
            SET formatter = EXCLUDED.formatter, config = EXCLUDED.config
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(repo_id)
        .bind(language)
        .bind(formatter)
        .bind(config)
        .fetch_one(&self.db)
        .await?;

        Ok(code_formatter)
    }

    pub async fn format_code(
        &self,
        repo_id: Uuid,
        request: &FormatRequest,
    ) -> Result<FormatResult, sqlx::Error> {
        let formatter = self.get_formatter(repo_id, &request.language).await?;

        match formatter {
            Some(f) => {
                let formatted = match f.formatter.as_str() {
                    "rustfmt" => self.format_rust(&request.content, &f.config)?,
                    "prettier" => self.format_javascript(&request.content, &f.config)?,
                    "black" => self.format_python(&request.content, &f.config)?,
                    _ => request.content.clone(),
                };

                let changed = formatted != request.content;

                Ok(FormatResult {
                    formatted_content: formatted,
                    changed,
                    formatter: f.formatter,
                })
            }
            None => Ok(FormatResult {
                formatted_content: request.content.clone(),
                changed: false,
                formatter: "none".to_string(),
            }),
        }
    }

    fn format_rust(
        &self,
        content: &str,
        _config: &serde_json::Value,
    ) -> Result<String, sqlx::Error> {
        Ok(content.to_string())
    }

    fn format_javascript(
        &self,
        content: &str,
        _config: &serde_json::Value,
    ) -> Result<String, sqlx::Error> {
        Ok(content.to_string())
    }

    fn format_python(
        &self,
        content: &str,
        _config: &serde_json::Value,
    ) -> Result<String, sqlx::Error> {
        Ok(content.to_string())
    }

    pub async fn format_on_save(
        &self,
        repo_id: Uuid,
        file_path: &str,
        content: &str,
        language: &str,
    ) -> Result<FormatResult, sqlx::Error> {
        let request = FormatRequest {
            file_path: file_path.to_string(),
            content: content.to_string(),
            language: language.to_string(),
        };

        self.format_code(repo_id, &request).await
    }

    pub async fn format_on_commit(
        &self,
        repo_id: Uuid,
        files: Vec<(String, String, String)>,
    ) -> Result<Vec<(String, FormatResult)>, sqlx::Error> {
        let mut results = Vec::new();

        for (file_path, content, language) in files {
            let request = FormatRequest {
                file_path: file_path.clone(),
                content,
                language,
            };

            let result = self.format_code(repo_id, &request).await?;
            results.push((file_path, result));
        }

        Ok(results)
    }
}