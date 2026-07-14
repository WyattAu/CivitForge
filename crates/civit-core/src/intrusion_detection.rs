#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrusionDetection {
    pub id: Uuid,
    pub detection_type: String,
    pub severity: String,
    pub source_ip: String,
    pub target: String,
    pub message: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIntrusionDetection {
    pub detection_type: String,
    pub severity: Option<String>,
    pub source_ip: String,
    pub target: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIntrusionDetection {
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrusionDetectionRule {
    pub id: Uuid,
    pub name: String,
    pub detection_type: String,
    pub pattern: String,
    pub severity: String,
    pub action: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIntrusionRule {
    pub name: String,
    pub detection_type: String,
    pub pattern: String,
    pub severity: Option<String>,
    pub action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrusionIncident {
    pub id: Uuid,
    pub detection_id: Uuid,
    pub response_action: String,
    pub response_data: serde_json::Value,
    pub resolved: bool,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct DetectionRow {
    id: Uuid,
    detection_type: String,
    severity: String,
    source_ip: String,
    target: String,
    message: String,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<DetectionRow> for IntrusionDetection {
    fn from(row: DetectionRow) -> Self {
        IntrusionDetection {
            id: row.id,
            detection_type: row.detection_type,
            severity: row.severity,
            source_ip: row.source_ip,
            target: row.target,
            message: row.message,
            status: row.status,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct RuleRow {
    id: Uuid,
    name: String,
    detection_type: String,
    pattern: String,
    severity: String,
    action: String,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<RuleRow> for IntrusionDetectionRule {
    fn from(row: RuleRow) -> Self {
        IntrusionDetectionRule {
            id: row.id,
            name: row.name,
            detection_type: row.detection_type,
            pattern: row.pattern,
            severity: row.severity,
            action: row.action,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct IncidentRow {
    id: Uuid,
    detection_id: Uuid,
    response_action: String,
    response_data: serde_json::Value,
    resolved: bool,
    resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<IncidentRow> for IntrusionIncident {
    fn from(row: IncidentRow) -> Self {
        IntrusionIncident {
            id: row.id,
            detection_id: row.detection_id,
            response_action: row.response_action,
            response_data: row.response_data,
            resolved: row.resolved,
            resolved_at: row.resolved_at,
            created_at: row.created_at,
        }
    }
}

pub struct IntrusionDetectionService {
    pool: PgPool,
}

impl IntrusionDetectionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_detection(
        &self,
        input: CreateIntrusionDetection,
    ) -> Result<IntrusionDetection, sqlx::Error> {
        let row = sqlx::query_as::<_, DetectionRow>(
            r#"INSERT INTO intrusion_detections (detection_type, severity, source_ip, target, message)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, detection_type, severity, source_ip, target, message, status, created_at"#,
        )
        .bind(&input.detection_type)
        .bind(input.severity.as_deref().unwrap_or("medium"))
        .bind(&input.source_ip)
        .bind(&input.target)
        .bind(&input.message)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_detection(
        &self,
        id: Uuid,
    ) -> Result<Option<IntrusionDetection>, sqlx::Error> {
        let row = sqlx::query_as::<_, DetectionRow>(
            r#"SELECT id, detection_type, severity, source_ip, target, message, status, created_at
             FROM intrusion_detections WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_detections(&self) -> Result<Vec<IntrusionDetection>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DetectionRow>(
            r#"SELECT id, detection_type, severity, source_ip, target, message, status, created_at
             FROM intrusion_detections ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_detection(
        &self,
        id: Uuid,
        input: UpdateIntrusionDetection,
    ) -> Result<IntrusionDetection, sqlx::Error> {
        let row = sqlx::query_as::<_, DetectionRow>(
            r#"UPDATE intrusion_detections SET
             status = COALESCE($2, status)
             WHERE id = $1
             RETURNING id, detection_type, severity, source_ip, target, message, status, created_at"#,
        )
        .bind(id)
        .bind(&input.status)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn create_rule(
        &self,
        input: CreateIntrusionRule,
    ) -> Result<IntrusionDetectionRule, sqlx::Error> {
        let row = sqlx::query_as::<_, RuleRow>(
            r#"INSERT INTO intrusion_detection_rules (name, detection_type, pattern, severity, action)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, name, detection_type, pattern, severity, action, enabled, created_at"#,
        )
        .bind(&input.name)
        .bind(&input.detection_type)
        .bind(&input.pattern)
        .bind(input.severity.as_deref().unwrap_or("medium"))
        .bind(input.action.as_deref().unwrap_or("alert"))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn list_rules(&self) -> Result<Vec<IntrusionDetectionRule>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RuleRow>(
            r#"SELECT id, name, detection_type, pattern, severity, action, enabled, created_at
             FROM intrusion_detection_rules ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_incident(
        &self,
        detection_id: Uuid,
        response_action: &str,
    ) -> Result<IntrusionIncident, sqlx::Error> {
        let row = sqlx::query_as::<_, IncidentRow>(
            r#"INSERT INTO intrusion_incidents (detection_id, response_action)
             VALUES ($1, $2)
             RETURNING id, detection_id, response_action, response_data, resolved, resolved_at, created_at"#,
        )
        .bind(detection_id)
        .bind(response_action)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn resolve_incident(
        &self,
        id: Uuid,
    ) -> Result<IntrusionIncident, sqlx::Error> {
        let row = sqlx::query_as::<_, IncidentRow>(
            r#"UPDATE intrusion_incidents SET
             resolved = true, resolved_at = NOW()
             WHERE id = $1
             RETURNING id, detection_id, response_action, response_data, resolved, resolved_at, created_at"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_incidents_for_detection(
        &self,
        detection_id: Uuid,
    ) -> Result<Vec<IntrusionIncident>, sqlx::Error> {
        let rows = sqlx::query_as::<_, IncidentRow>(
            r#"SELECT id, detection_id, response_action, response_data, resolved, resolved_at, created_at
             FROM intrusion_incidents WHERE detection_id = $1 ORDER BY created_at DESC"#,
        )
        .bind(detection_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn detect_from_log(
        &self,
        log_entry: &str,
        source_ip: &str,
        target: &str,
    ) -> Result<Vec<IntrusionDetection>, sqlx::Error> {
        let rules = sqlx::query_as::<_, RuleRow>(
            r#"SELECT id, name, detection_type, pattern, severity, action, enabled, created_at
             FROM intrusion_detection_rules WHERE enabled = true"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut detections = Vec::new();
        for rule in rules {
            if log_entry.contains(&rule.pattern) {
                let detection = self
                    .create_detection(CreateIntrusionDetection {
                        detection_type: rule.detection_type,
                        severity: Some(rule.severity),
                        source_ip: source_ip.to_string(),
                        target: target.to_string(),
                        message: format!("Rule '{}' matched: {}", rule.name, rule.pattern),
                    })
                    .await?;
                detections.push(detection);
            }
        }

        Ok(detections)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detection_serialization() {
        let detection = IntrusionDetection {
            id: Uuid::new_v4(),
            detection_type: "brute_force".into(),
            severity: "high".into(),
            source_ip: "10.0.0.1".into(),
            target: "ssh".into(),
            message: "multiple failed logins".into(),
            status: "open".into(),
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&detection).unwrap();
        assert!(json.contains("brute_force"));
        assert!(json.contains("high"));
    }

    #[test]
    fn test_rule_serialization() {
        let rule = IntrusionDetectionRule {
            id: Uuid::new_v4(),
            name: "ssh-brute".into(),
            detection_type: "brute_force".into(),
            pattern: "Failed password".into(),
            severity: "high".into(),
            action: "block".into(),
            enabled: true,
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&rule).unwrap();
        assert!(json.contains("Failed password"));
    }
}
