#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProjectBoard {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BoardColumn {
    pub id: Uuid,
    pub board_id: Uuid,
    pub name: String,
    pub position: i32,
    pub color: String,
    pub wip_limit: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BoardCard {
    pub id: Uuid,
    pub board_id: Uuid,
    pub column_id: Uuid,
    pub issue_id: Option<Uuid>,
    pub position: i32,
    pub assignee_id: Option<Uuid>,
    pub labels: serde_json::Value,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CardMovement {
    pub id: Uuid,
    pub card_id: Uuid,
    pub from_column_id: Option<Uuid>,
    pub to_column_id: Uuid,
    pub moved_by: Uuid,
    pub moved_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardWithColumns {
    pub board: ProjectBoard,
    pub columns: Vec<ColumnWithCards>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnWithCards {
    pub column: BoardColumn,
    pub cards: Vec<BoardCard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardAnalytics {
    pub total_cards: i64,
    pub cards_per_column: Vec<ColumnCardCount>,
    pub wip_violations: Vec<WipViolation>,
    pub average_cycle_time_days: Option<f64>,
    pub throughput_last_7_days: i64,
    pub throughput_last_30_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ColumnCardCount {
    pub column_id: Uuid,
    pub column_name: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WipViolation {
    pub column_id: Uuid,
    pub column_name: String,
    pub wip_limit: i32,
    pub current_count: i64,
}

pub struct ProjectBoardService {
    pool: PgPool,
}

impl ProjectBoardService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_column(
        &self,
        board_id: Uuid,
        name: &str,
        position: i32,
        color: &str,
        wip_limit: Option<i32>,
    ) -> Result<BoardColumn, sqlx::Error> {
        let column = sqlx::query_as::<_, BoardColumn>(
            r#"
            INSERT INTO project_board_columns_v1 (board_id, name, position, color, wip_limit)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, board_id, name, position, color, wip_limit, created_at
            "#,
        )
        .bind(board_id)
        .bind(name)
        .bind(position)
        .bind(color)
        .bind(wip_limit)
        .fetch_one(&self.pool)
        .await?;

        Ok(column)
    }

    pub async fn create_card(
        &self,
        board_id: Uuid,
        column_id: Uuid,
        issue_id: Option<Uuid>,
        position: i32,
    ) -> Result<BoardCard, sqlx::Error> {
        let card = sqlx::query_as::<_, BoardCard>(
            r#"
            INSERT INTO project_board_cards_v1 (board_id, column_id, issue_id, position)
            VALUES ($1, $2, $3, $4)
            RETURNING id, board_id, column_id, issue_id, position, assignee_id, labels, due_date, created_at, updated_at
            "#,
        )
        .bind(board_id)
        .bind(column_id)
        .bind(issue_id)
        .bind(position)
        .fetch_one(&self.pool)
        .await?;

        Ok(card)
    }

    pub async fn move_card(
        &self,
        card_id: Uuid,
        to_column_id: Uuid,
        moved_by: Uuid,
    ) -> Result<BoardCard, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let current: BoardCard = sqlx::query_as::<_, BoardCard>(
            r#"
            SELECT id, board_id, column_id, issue_id, position, assignee_id, labels, due_date, created_at, updated_at
            FROM project_board_cards_v1
            WHERE id = $1
            FOR UPDATE
            "#,
        )
        .bind(card_id)
        .fetch_one(&mut *tx)
        .await?;

        let from_column_id = if current.column_id == to_column_id {
            None
        } else {
            Some(current.column_id)
        };

        sqlx::query(
            r#"
            INSERT INTO project_board_card_movements_v1 (card_id, from_column_id, to_column_id, moved_by)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(card_id)
        .bind(from_column_id)
        .bind(to_column_id)
        .bind(moved_by)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            UPDATE project_board_cards_v1
            SET column_id = $2, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(card_id)
        .bind(to_column_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        let updated = sqlx::query_as::<_, BoardCard>(
            r#"
            SELECT id, board_id, column_id, issue_id, position, assignee_id, labels, due_date, created_at, updated_at
            FROM project_board_cards_v1
            WHERE id = $1
            "#,
        )
        .bind(card_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    pub async fn get_board_with_cards(
        &self,
        board_id: Uuid,
    ) -> Result<BoardWithColumns, sqlx::Error> {
        let board = sqlx::query_as::<_, ProjectBoard>(
            r#"
            SELECT id, name, description, owner_id, created_at, updated_at
            FROM project_boards
            WHERE id = $1
            "#,
        )
        .bind(board_id)
        .fetch_one(&self.pool)
        .await?;

        let columns = sqlx::query_as::<_, BoardColumn>(
            r#"
            SELECT id, board_id, name, position, color, wip_limit, created_at
            FROM project_board_columns_v1
            WHERE board_id = $1
            ORDER BY position ASC
            "#,
        )
        .bind(board_id)
        .fetch_all(&self.pool)
        .await?;

        let all_cards = sqlx::query_as::<_, BoardCard>(
            r#"
            SELECT id, board_id, column_id, issue_id, position, assignee_id, labels, due_date, created_at, updated_at
            FROM project_board_cards_v1
            WHERE board_id = $1
            ORDER BY position ASC
            "#,
        )
        .bind(board_id)
        .fetch_all(&self.pool)
        .await?;

        let mut columns_with_cards: Vec<ColumnWithCards> = Vec::with_capacity(columns.len());
        for col in columns {
            let cards: Vec<BoardCard> = all_cards
                .iter()
                .filter(|c| c.column_id == col.id)
                .cloned()
                .collect();
            columns_with_cards.push(ColumnWithCards { column: col, cards });
        }

        Ok(BoardWithColumns {
            board,
            columns: columns_with_cards,
        })
    }

    pub async fn get_board_analytics(
        &self,
        board_id: Uuid,
    ) -> Result<BoardAnalytics, sqlx::Error> {
        let total_cards: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)::bigint FROM project_board_cards_v1 WHERE board_id = $1",
        )
        .bind(board_id)
        .fetch_one(&self.pool)
        .await?;

        let cards_per_column = sqlx::query_as::<_, ColumnCardCount>(
            r#"
            SELECT c.id AS column_id, c.name AS column_name, COUNT(cr.id)::bigint AS count
            FROM project_board_columns_v1 c
            LEFT JOIN project_board_cards_v1 cr ON cr.column_id = c.id
            WHERE c.board_id = $1
            GROUP BY c.id, c.name
            ORDER BY c.position ASC
            "#,
        )
        .bind(board_id)
        .fetch_all(&self.pool)
        .await?;

        let wip_violations = sqlx::query_as::<_, WipViolation>(
            r#"
            SELECT c.id AS column_id, c.name AS column_name, c.wip_limit, COUNT(cr.id)::bigint AS current_count
            FROM project_board_columns_v1 c
            LEFT JOIN project_board_cards_v1 cr ON cr.column_id = c.id
            WHERE c.board_id = $1 AND c.wip_limit IS NOT NULL
            GROUP BY c.id, c.name, c.wip_limit
            HAVING COUNT(cr.id) > c.wip_limit
            "#,
        )
        .bind(board_id)
        .fetch_all(&self.pool)
        .await?;

        let avg_cycle_time: Option<f64> = sqlx::query_scalar(
            r#"
            SELECT AVG(EXTRACT(EPOCH FROM (m2.moved_at - m1.moved_at)) / 86400.0)::float8 AS avg_days
            FROM project_board_card_movements_v1 m1
            JOIN project_board_card_movements_v1 m2 ON m1.card_id = m2.card_id
            JOIN project_board_cards_v1 cr ON cr.id = m1.card_id
            JOIN project_board_columns_v1 c1 ON c1.id = m1.to_column_id
            JOIN project_board_columns_v1 c2 ON c2.id = m2.to_column_id
            WHERE cr.board_id = $1
              AND m2.moved_at > m1.moved_at
              AND c2.position > c1.position
            "#,
        )
        .bind(board_id)
        .fetch_one(&self.pool)
        .await?;

        let throughput_7d: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(DISTINCT m.card_id)::bigint
            FROM project_board_card_movements_v1 m
            JOIN project_board_cards_v1 cr ON cr.id = m.card_id
            WHERE cr.board_id = $1 AND m.moved_at >= NOW() - INTERVAL '7 days'
            "#,
        )
        .bind(board_id)
        .fetch_one(&self.pool)
        .await?;

        let throughput_30d: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(DISTINCT m.card_id)::bigint
            FROM project_board_card_movements_v1 m
            JOIN project_board_cards_v1 cr ON cr.id = m.card_id
            WHERE cr.board_id = $1 AND m.moved_at >= NOW() - INTERVAL '30 days'
            "#,
        )
        .bind(board_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(BoardAnalytics {
            total_cards,
            cards_per_column,
            wip_violations,
            average_cycle_time_days: avg_cycle_time,
            throughput_last_7_days: throughput_7d,
            throughput_last_30_days: throughput_30d,
        })
    }

    pub async fn reorder_cards(
        &self,
        column_id: Uuid,
        card_ids: &[Uuid],
    ) -> Result<Vec<BoardCard>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        for (i, card_id) in card_ids.iter().enumerate() {
            sqlx::query(
                r#"
                UPDATE project_board_cards_v1
                SET position = $2, updated_at = NOW()
                WHERE id = $1 AND column_id = $3
                "#,
            )
            .bind(card_id)
            .bind(i as i32)
            .bind(column_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        let cards = sqlx::query_as::<_, BoardCard>(
            r#"
            SELECT id, board_id, column_id, issue_id, position, assignee_id, labels, due_date, created_at, updated_at
            FROM project_board_cards_v1
            WHERE column_id = $1
            ORDER BY position ASC
            "#,
        )
        .bind(column_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(cards)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_serialization_roundtrip() {
        let board = ProjectBoard {
            id: Uuid::new_v4(),
            name: "Test Board".to_string(),
            description: Some("desc".to_string()),
            owner_id: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&board).unwrap();
        let de: ProjectBoard = serde_json::from_str(&json).unwrap();
        assert_eq!(de.name, "Test Board");
    }

    #[test]
    fn test_column_card_count_serialization() {
        let count = ColumnCardCount {
            column_id: Uuid::new_v4(),
            column_name: "To Do".to_string(),
            count: 5,
        };
        let json = serde_json::to_string(&count).unwrap();
        let de: ColumnCardCount = serde_json::from_str(&json).unwrap();
        assert_eq!(de.count, 5);
    }
}
