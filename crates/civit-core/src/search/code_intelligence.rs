#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Symbol {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub line_number: i32,
    pub column_number: i32,
    pub signature: Option<String>,
    pub documentation: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Reference {
    pub id: Uuid,
    pub symbol_id: Uuid,
    pub file_path: String,
    pub line_number: i32,
    pub column_number: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionLocation {
    pub file_path: String,
    pub line_number: i32,
    pub column_number: i32,
    pub symbol: Symbol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceLocation {
    pub file_path: String,
    pub line_number: i32,
    pub column_number: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverInfo {
    pub symbol: Symbol,
    pub documentation: Option<String>,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub name: String,
    pub kind: String,
    pub signature: Option<String>,
    pub documentation: Option<String>,
    pub file_path: String,
    pub line_number: i32,
    pub column_number: i32,
}

pub struct CodeIntelligenceService {
    db: sqlx::PgPool,
}

impl CodeIntelligenceService {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn index_symbol(
        &self,
        repo_id: Uuid,
        name: &str,
        kind: &str,
        file_path: &str,
        line_number: i32,
        column_number: i32,
        signature: Option<&str>,
        documentation: Option<&str>,
    ) -> Result<Symbol, sqlx::Error> {
        let id = Uuid::new_v4();
        let symbol = sqlx::query_as::<_, Symbol>(
            r#"
            INSERT INTO code_intelligence_symbols (id, repo_id, name, kind, file_path, line_number, column_number, signature, documentation)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(repo_id)
        .bind(name)
        .bind(kind)
        .bind(file_path)
        .bind(line_number)
        .bind(column_number)
        .bind(signature)
        .bind(documentation)
        .fetch_one(&self.db)
        .await?;

        Ok(symbol)
    }

    pub async fn get_definition(
        &self,
        repo_id: Uuid,
        name: &str,
    ) -> Result<Option<DefinitionLocation>, sqlx::Error> {
        let symbol = sqlx::query_as::<_, Symbol>(
            r#"
            SELECT * FROM code_intelligence_symbols
            WHERE repo_id = $1 AND name = $2
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(repo_id)
        .bind(name)
        .fetch_optional(&self.db)
        .await?;

        match symbol {
            Some(s) => Ok(Some(DefinitionLocation {
                file_path: s.file_path.clone(),
                line_number: s.line_number,
                column_number: s.column_number,
                symbol: s,
            })),
            None => Ok(None),
        }
    }

    pub async fn find_references(
        &self,
        symbol_id: Uuid,
    ) -> Result<Vec<ReferenceLocation>, sqlx::Error> {
        let references = sqlx::query_as::<_, Reference>(
            r#"
            SELECT * FROM code_intelligence_references
            WHERE symbol_id = $1
            ORDER BY file_path, line_number
            "#,
        )
        .bind(symbol_id)
        .fetch_all(&self.db)
        .await?;

        Ok(references
            .into_iter()
            .map(|r| ReferenceLocation {
                file_path: r.file_path,
                line_number: r.line_number,
                column_number: r.column_number,
            })
            .collect())
    }

    pub async fn get_hover_info(
        &self,
        repo_id: Uuid,
        name: &str,
    ) -> Result<Option<HoverInfo>, sqlx::Error> {
        let symbol = sqlx::query_as::<_, Symbol>(
            r#"
            SELECT * FROM code_intelligence_symbols
            WHERE repo_id = $1 AND name = $2
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(repo_id)
        .bind(name)
        .fetch_optional(&self.db)
        .await?;

        match symbol {
            Some(s) => Ok(Some(HoverInfo {
                documentation: s.documentation.clone(),
                signature: s.signature.clone(),
                symbol: s,
            })),
            None => Ok(None),
        }
    }

    pub async fn get_completions(
        &self,
        repo_id: Uuid,
        prefix: &str,
        limit: usize,
    ) -> Result<Vec<CompletionItem>, sqlx::Error> {
        let symbols = sqlx::query_as::<_, Symbol>(
            r#"
            SELECT * FROM code_intelligence_symbols
            WHERE repo_id = $1 AND name ILIKE $2
            ORDER BY name
            LIMIT $3
            "#,
        )
        .bind(repo_id)
        .bind(format!("{}%", prefix))
        .bind(limit as i64)
        .fetch_all(&self.db)
        .await?;

        Ok(symbols
            .into_iter()
            .map(|s| CompletionItem {
                name: s.name,
                kind: s.kind,
                signature: s.signature,
                documentation: s.documentation,
                file_path: s.file_path,
                line_number: s.line_number,
                column_number: s.column_number,
            })
            .collect())
    }
}