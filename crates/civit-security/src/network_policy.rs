#![forbid(unsafe_code)]

//! Network policy management for CivitForge.
//!
//! Provides ingress/egress rule management, network segmentation,
//! and traffic filtering capabilities.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRule {
    pub protocol: String,
    pub ports: Vec<u16>,
    pub sources: Vec<String>,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub ingress_rules: Vec<NetworkRule>,
    pub egress_rules: Vec<NetworkRule>,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNetworkPolicy {
    pub name: String,
    pub description: String,
    pub ingress_rules: Vec<NetworkRule>,
    pub egress_rules: Vec<NetworkRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNetworkPolicy {
    pub name: Option<String>,
    pub description: Option<String>,
    pub ingress_rules: Option<Vec<NetworkRule>>,
    pub egress_rules: Option<Vec<NetworkRule>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, sqlx::FromRow)]
struct NetworkPolicyRow {
    id: Uuid,
    name: String,
    description: String,
    ingress_rules: serde_json::Value,
    egress_rules: serde_json::Value,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<NetworkPolicyRow> for NetworkPolicy {
    fn from(row: NetworkPolicyRow) -> Self {
        let ingress: Vec<NetworkRule> = serde_json::from_value(row.ingress_rules).unwrap_or_default();
        let egress: Vec<NetworkRule> = serde_json::from_value(row.egress_rules).unwrap_or_default();
        NetworkPolicy {
            id: row.id,
            name: row.name,
            description: row.description,
            ingress_rules: ingress,
            egress_rules: egress,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

pub struct NetworkPolicyService {
    pool: PgPool,
}

impl NetworkPolicyService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_policy(&self, input: CreateNetworkPolicy) -> Result<NetworkPolicy, sqlx::Error> {
        let ingress = serde_json::to_value(&input.ingress_rules).unwrap_or_default();
        let egress = serde_json::to_value(&input.egress_rules).unwrap_or_default();

        let row = sqlx::query_as::<_, NetworkPolicyRow>(
            r#"INSERT INTO network_policies (name, description, ingress_rules, egress_rules)
             VALUES ($1, $2, $3, $4)
             RETURNING id, name, description, ingress_rules, egress_rules, enabled, created_at"#,
        )
        .bind(&input.name)
        .bind(&input.description)
        .bind(ingress)
        .bind(egress)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_policy(&self, id: Uuid) -> Result<Option<NetworkPolicy>, sqlx::Error> {
        let row = sqlx::query_as::<_, NetworkPolicyRow>(
            r#"SELECT id, name, description, ingress_rules, egress_rules, enabled, created_at
             FROM network_policies WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_policies(&self) -> Result<Vec<NetworkPolicy>, sqlx::Error> {
        let rows = sqlx::query_as::<_, NetworkPolicyRow>(
            r#"SELECT id, name, description, ingress_rules, egress_rules, enabled, created_at
             FROM network_policies ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_policy(
        &self,
        id: Uuid,
        input: UpdateNetworkPolicy,
    ) -> Result<NetworkPolicy, sqlx::Error> {
        let ingress = input.ingress_rules.as_ref().and_then(|r| serde_json::to_value(r).ok());
        let egress = input.egress_rules.as_ref().and_then(|r| serde_json::to_value(r).ok());

        let row = sqlx::query_as::<_, NetworkPolicyRow>(
            r#"UPDATE network_policies SET
             name = COALESCE($2, name),
             description = COALESCE($3, description),
             ingress_rules = COALESCE($4, ingress_rules),
             egress_rules = COALESCE($5, egress_rules),
             enabled = COALESCE($6, enabled)
             WHERE id = $1
             RETURNING id, name, description, ingress_rules, egress_rules, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(ingress)
        .bind(egress)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_policy(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM network_policies WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn check_allowed(
        &self,
        source_ip: &str,
        destination_ip: &str,
        port: u16,
        protocol: &str,
    ) -> Result<bool, sqlx::Error> {
        let policies = sqlx::query_as::<_, NetworkPolicyRow>(
            r#"SELECT id, name, description, ingress_rules, egress_rules, enabled, created_at
             FROM network_policies WHERE enabled = true"#,
        )
        .fetch_all(&self.pool)
        .await?;

        for policy in &policies {
            if self.matches_egress(&policy.egress_rules, destination_ip, port, protocol)
                && self.matches_ingress(&policy.ingress_rules, source_ip, port, protocol)
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn matches_ingress(
        &self,
        rules: &serde_json::Value,
        source_ip: &str,
        port: u16,
        protocol: &str,
    ) -> bool {
        self.matches_rule(rules, source_ip, port, protocol)
    }

    fn matches_egress(
        &self,
        rules: &serde_json::Value,
        dest_ip: &str,
        port: u16,
        protocol: &str,
    ) -> bool {
        self.matches_rule(rules, dest_ip, port, protocol)
    }

    fn matches_rule(
        &self,
        rules: &serde_json::Value,
        ip: &str,
        port: u16,
        protocol: &str,
    ) -> bool {
        if let Some(arr) = rules.as_array() {
            for rule_val in arr {
                if let Ok(rule) = serde_json::from_value::<NetworkRule>(rule_val.clone())
                    && rule.protocol == protocol
                        && rule.ports.contains(&port)
                        && (rule.sources.is_empty() || rule.sources.iter().any(|s| s == ip || s == "*"))
                    {
                        return true;
                    }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_rule_serialization() {
        let rule = NetworkRule {
            protocol: "TCP".into(),
            ports: vec![80, 443],
            sources: vec!["10.0.0.0/8".into()],
            action: "allow".into(),
        };
        let json = serde_json::to_string(&rule).unwrap();
        assert!(json.contains("TCP"));
        assert!(json.contains("80"));
    }
}
