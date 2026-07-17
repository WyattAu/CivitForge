#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CollaborationSession {
    pub id: Uuid,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub user_id: Uuid,
    pub cursor_position: Option<serde_json::Value>,
    pub last_active: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub line: u32,
    pub column: u32,
    pub selection_start: Option<u32>,
    pub selection_end: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborativeChange {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub change_type: ChangeType,
    pub position: CursorPosition,
    pub content: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeType {
    Insert,
    Delete,
    Replace,
    CursorMove,
    SelectionChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub session_id: Uuid,
    pub change_id: Uuid,
    pub resolved: bool,
    pub merged_content: Option<String>,
    pub resolution_type: ResolutionType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResolutionType {
    AutoMerge,
    LastWriteWins,
    OperationalTransform,
    ManualResolution,
}

struct SessionEntry {
    session: CollaborationSession,
    senders: Vec<tokio::sync::mpsc::UnboundedSender<CollaborativeChange>>,
}

pub struct LiveCollaborationService {
    sessions: Arc<RwLock<HashMap<Uuid, SessionEntry>>>,
    resource_sessions: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
}

impl Default for LiveCollaborationService {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveCollaborationService {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            resource_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn resource_key(resource_type: &str, resource_id: Uuid) -> String {
        format!("{}:{}", resource_type, resource_id)
    }

    pub async fn join_session(
        &self,
        pool: &sqlx::PgPool,
        user_id: Uuid,
        resource_type: &str,
        resource_id: Uuid,
    ) -> Result<(CollaborationSession, tokio::sync::mpsc::UnboundedReceiver<CollaborativeChange>), sqlx::Error> {
        let existing = sqlx::query_as::<_, CollaborationSession>(
            "SELECT id, resource_type, resource_id, user_id, cursor_position, last_active, created_at \
             FROM live_collaboration_sessions \
             WHERE resource_type = $1 AND resource_id = $2 AND user_id = $3",
        )
        .bind(resource_type)
        .bind(resource_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        let session = match existing {
            Some(s) => {
                sqlx::query_as::<_, CollaborationSession>(
                    "UPDATE live_collaboration_sessions \
                     SET last_active = NOW() \
                     WHERE id = $1 \
                     RETURNING id, resource_type, resource_id, user_id, cursor_position, last_active, created_at",
                )
                .bind(s.id)
                .fetch_one(pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, CollaborationSession>(
                    "INSERT INTO live_collaboration_sessions (id, resource_type, resource_id, user_id) \
                     VALUES ($1, $2, $3, $4) \
                     RETURNING id, resource_type, resource_id, user_id, cursor_position, last_active, created_at",
                )
                .bind(Uuid::new_v4())
                .bind(resource_type)
                .bind(resource_id)
                .bind(user_id)
                .fetch_one(pool)
                .await?
            }
        };

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let entry = SessionEntry {
            session: session.clone(),
            senders: vec![tx],
        };

        self.sessions.write().await.insert(session.id, entry);

        let resource_key = Self::resource_key(resource_type, resource_id);
        self.resource_sessions
            .write()
            .await
            .entry(resource_key)
            .or_default()
            .push(session.id);

        Ok((session, rx))
    }

    pub async fn leave_session(
        &self,
        pool: &sqlx::PgPool,
        session_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let session = sqlx::query_as::<_, CollaborationSession>(
            "SELECT id, resource_type, resource_id, user_id, cursor_position, last_active, created_at \
             FROM live_collaboration_sessions \
             WHERE id = $1",
        )
        .bind(session_id)
        .fetch_optional(pool)
        .await?;

        if let Some(s) = session {
            self.sessions.write().await.remove(&session_id);

            let resource_key = Self::resource_key(&s.resource_type, s.resource_id);
            if let Some(ids) = self.resource_sessions.write().await.get_mut(&resource_key) {
                ids.retain(|id| *id != session_id);
            }
        }

        Ok(())
    }

    pub async fn update_cursor(
        &self,
        pool: &sqlx::PgPool,
        session_id: Uuid,
        position: CursorPosition,
    ) -> Result<(), sqlx::Error> {
        let position_json = serde_json::to_value(&position).unwrap_or_default();

        sqlx::query(
            "UPDATE live_collaboration_sessions \
             SET cursor_position = $1, last_active = NOW() \
             WHERE id = $2",
        )
        .bind(&position_json)
        .bind(session_id)
        .execute(pool)
        .await?;

        if let Some(entry) = self.sessions.write().await.get_mut(&session_id) {
            entry.session.cursor_position = Some(position_json);
            entry.session.last_active = Utc::now();
        }

        Ok(())
    }

    pub async fn broadcast_change(
        &self,
        change: CollaborativeChange,
    ) -> usize {
        let resource_key = Self::resource_key(&change.resource_type, change.resource_id);
        let resource_sessions = self.resource_sessions.read().await;

        match resource_sessions.get(&resource_key) {
            Some(session_ids) => {
                let sessions = self.sessions.read().await;
                let mut sent = 0;

                for session_id in session_ids {
                    if let Some(entry) = sessions.get(session_id)
                        && entry.session.user_id != change.user_id {
                            for sender in &entry.senders {
                                if sender.send(change.clone()).is_ok() {
                                    sent += 1;
                                }
                            }
                        }
                }

                sent
            }
            None => 0,
        }
    }

    pub async fn get_session(
        &self,
        pool: &sqlx::PgPool,
        session_id: Uuid,
    ) -> Result<Option<CollaborationSession>, sqlx::Error> {
        sqlx::query_as::<_, CollaborationSession>(
            "SELECT id, resource_type, resource_id, user_id, cursor_position, last_active, created_at \
             FROM live_collaboration_sessions \
             WHERE id = $1",
        )
        .bind(session_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn get_active_sessions(
        &self,
        pool: &sqlx::PgPool,
        resource_type: &str,
        resource_id: Uuid,
    ) -> Result<Vec<CollaborationSession>, sqlx::Error> {
        sqlx::query_as::<_, CollaborationSession>(
            "SELECT id, resource_type, resource_id, user_id, cursor_position, last_active, created_at \
             FROM live_collaboration_sessions \
             WHERE resource_type = $1 AND resource_id = $2 \
             AND last_active > NOW() - INTERVAL '5 minutes' \
             ORDER BY last_active DESC",
        )
        .bind(resource_type)
        .bind(resource_id)
        .fetch_all(pool)
        .await
    }

    pub async fn cleanup_stale_sessions(
        &self,
        pool: &sqlx::PgPool,
        timeout_minutes: i64,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM live_collaboration_sessions \
             WHERE last_active < NOW() - make_interval(mins => $1)",
        )
        .bind(timeout_minutes)
        .execute(pool)
        .await?;

        let deleted_count = result.rows_affected();

        let stale_ids: Vec<Uuid> = self
            .sessions
            .read()
            .await
            .iter()
            .filter(|(_, entry)| {
                let now = Utc::now();
                let duration = now.signed_duration_since(entry.session.last_active);
                duration.num_minutes() > timeout_minutes
            })
            .map(|(id, _)| *id)
            .collect();

        for id in stale_ids {
            self.sessions.write().await.remove(&id);
        }

        Ok(deleted_count)
    }

    pub async fn resolve_conflict(
        &self,
        session_id: Uuid,
        change: &CollaborativeChange,
        resolution: ResolutionType,
    ) -> ConflictResolution {
        let resolved = matches!(
            resolution,
            ResolutionType::AutoMerge | ResolutionType::LastWriteWins | ResolutionType::OperationalTransform
        );

        ConflictResolution {
            session_id,
            change_id: Uuid::new_v4(),
            resolved,
            merged_content: change.content.clone(),
            resolution_type: resolution,
        }
    }

    pub async fn get_active_resource_count(&self) -> usize {
        self.resource_sessions.read().await.len()
    }

    pub async fn get_total_session_count(&self) -> usize {
        self.sessions.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collaboration_session_creation() {
        let session = CollaborationSession {
            id: Uuid::new_v4(),
            resource_type: "document".to_string(),
            resource_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            cursor_position: None,
            last_active: Utc::now(),
            created_at: Utc::now(),
        };

        assert_eq!(session.resource_type, "document");
        assert!(session.cursor_position.is_none());
    }

    #[test]
    fn test_cursor_position_creation() {
        let cursor = CursorPosition {
            line: 10,
            column: 5,
            selection_start: Some(0),
            selection_end: Some(10),
        };

        assert_eq!(cursor.line, 10);
        assert_eq!(cursor.column, 5);
        assert!(cursor.selection_start.is_some());
    }

    #[test]
    fn test_collaborative_change_creation() {
        let change = CollaborativeChange {
            session_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            resource_type: "document".to_string(),
            resource_id: Uuid::new_v4(),
            change_type: ChangeType::Insert,
            position: CursorPosition {
                line: 0,
                column: 0,
                selection_start: None,
                selection_end: None,
            },
            content: Some("Hello".to_string()),
            timestamp: Utc::now(),
        };

        assert_eq!(change.change_type, ChangeType::Insert);
        assert_eq!(change.content, Some("Hello".to_string()));
    }

    #[test]
    fn test_conflict_resolution_creation() {
        let resolution = ConflictResolution {
            session_id: Uuid::new_v4(),
            change_id: Uuid::new_v4(),
            resolved: true,
            merged_content: Some("merged content".to_string()),
            resolution_type: ResolutionType::AutoMerge,
        };

        assert!(resolution.resolved);
        assert_eq!(resolution.resolution_type, ResolutionType::AutoMerge);
    }

    #[test]
    fn test_change_type_variants() {
        assert_eq!(ChangeType::Insert, ChangeType::Insert);
        assert_eq!(ChangeType::Delete, ChangeType::Delete);
        assert_eq!(ChangeType::Replace, ChangeType::Replace);
        assert_eq!(ChangeType::CursorMove, ChangeType::CursorMove);
        assert_eq!(ChangeType::SelectionChange, ChangeType::SelectionChange);
    }

    #[test]
    fn test_resolution_type_variants() {
        assert_eq!(ResolutionType::AutoMerge, ResolutionType::AutoMerge);
        assert_eq!(ResolutionType::LastWriteWins, ResolutionType::LastWriteWins);
        assert_eq!(ResolutionType::OperationalTransform, ResolutionType::OperationalTransform);
        assert_eq!(ResolutionType::ManualResolution, ResolutionType::ManualResolution);
    }

    #[test]
    fn test_service_new() {
        let service = LiveCollaborationService::new();
        assert_eq!(service.sessions.blocking_read().len(), 0);
        assert_eq!(service.resource_sessions.blocking_read().len(), 0);
    }

    #[test]
    fn test_resource_key() {
        let key = LiveCollaborationService::resource_key("document", Uuid::nil());
        assert!(key.starts_with("document:"));
    }

    #[tokio::test]
    async fn test_get_active_resource_count_empty() {
        let service = LiveCollaborationService::new();
        let count = service.get_active_resource_count().await;
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_get_total_session_count_empty() {
        let service = LiveCollaborationService::new();
        let count = service.get_total_session_count().await;
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_broadcast_change_no_sessions() {
        let service = LiveCollaborationService::new();
        let change = CollaborativeChange {
            session_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            resource_type: "document".to_string(),
            resource_id: Uuid::new_v4(),
            change_type: ChangeType::Insert,
            position: CursorPosition {
                line: 0,
                column: 0,
                selection_start: None,
                selection_end: None,
            },
            content: None,
            timestamp: Utc::now(),
        };

        let sent = service.broadcast_change(change).await;
        assert_eq!(sent, 0);
    }

    #[tokio::test]
    async fn test_resolve_conflict_auto_merge() {
        let service = LiveCollaborationService::new();
        let change = CollaborativeChange {
            session_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            resource_type: "document".to_string(),
            resource_id: Uuid::new_v4(),
            change_type: ChangeType::Insert,
            position: CursorPosition {
                line: 0,
                column: 0,
                selection_start: None,
                selection_end: None,
            },
            content: Some("test content".to_string()),
            timestamp: Utc::now(),
        };

        let resolution = service
            .resolve_conflict(Uuid::new_v4(), &change, ResolutionType::AutoMerge)
            .await;

        assert!(resolution.resolved);
        assert_eq!(resolution.resolution_type, ResolutionType::AutoMerge);
        assert_eq!(resolution.merged_content, Some("test content".to_string()));
    }

    #[tokio::test]
    async fn test_resolve_conflict_manual() {
        let service = LiveCollaborationService::new();
        let change = CollaborativeChange {
            session_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            resource_type: "document".to_string(),
            resource_id: Uuid::new_v4(),
            change_type: ChangeType::Insert,
            position: CursorPosition {
                line: 0,
                column: 0,
                selection_start: None,
                selection_end: None,
            },
            content: None,
            timestamp: Utc::now(),
        };

        let resolution = service
            .resolve_conflict(Uuid::new_v4(), &change, ResolutionType::ManualResolution)
            .await;

        assert!(!resolution.resolved);
        assert_eq!(resolution.resolution_type, ResolutionType::ManualResolution);
    }
}
