#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub action: String,
    pub protocol: String,
    pub source_ip: Option<String>,
    pub source_port: Option<i32>,
    pub destination_ip: Option<String>,
    pub destination_port: Option<i32>,
    pub enabled: bool,
    pub priority: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFirewallRule {
    pub name: String,
    pub description: Option<String>,
    pub action: Option<String>,
    pub protocol: Option<String>,
    pub source_ip: Option<String>,
    pub source_port: Option<i32>,
    pub destination_ip: Option<String>,
    pub destination_port: Option<i32>,
    pub enabled: Option<bool>,
    pub priority: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFirewallRule {
    pub name: Option<String>,
    pub description: Option<String>,
    pub action: Option<String>,
    pub protocol: Option<String>,
    pub source_ip: Option<String>,
    pub source_port: Option<i32>,
    pub destination_ip: Option<String>,
    pub destination_port: Option<i32>,
    pub enabled: Option<bool>,
    pub priority: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallEvaluation {
    pub allowed: bool,
    pub matched_rule_id: Option<Uuid>,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRuleTest {
    pub source_ip: String,
    pub destination_ip: Option<String>,
    pub destination_port: Option<i32>,
    pub protocol: String,
}

#[derive(Debug, sqlx::FromRow)]
struct FirewallRuleRow {
    id: Uuid,
    name: String,
    description: String,
    action: String,
    protocol: String,
    source_ip: Option<String>,
    source_port: Option<i32>,
    destination_ip: Option<String>,
    destination_port: Option<i32>,
    enabled: bool,
    priority: i32,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<FirewallRuleRow> for FirewallRule {
    fn from(row: FirewallRuleRow) -> Self {
        FirewallRule {
            id: row.id,
            name: row.name,
            description: row.description,
            action: row.action,
            protocol: row.protocol,
            source_ip: row.source_ip,
            source_port: row.source_port,
            destination_ip: row.destination_ip,
            destination_port: row.destination_port,
            enabled: row.enabled,
            priority: row.priority,
            created_at: row.created_at,
        }
    }
}

pub struct FirewallService {
    pool: PgPool,
}

impl FirewallService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_rule(&self, input: CreateFirewallRule) -> Result<FirewallRule, sqlx::Error> {
        let row = sqlx::query_as::<_, FirewallRuleRow>(
            r#"INSERT INTO firewall_rules (name, description, action, protocol, source_ip, source_port, destination_ip, destination_port, enabled, priority)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             RETURNING id, name, description, action, protocol, source_ip, source_port, destination_ip, destination_port, enabled, priority, created_at"#,
        )
        .bind(&input.name)
        .bind(input.description.as_deref().unwrap_or(""))
        .bind(input.action.as_deref().unwrap_or("allow"))
        .bind(input.protocol.as_deref().unwrap_or("tcp"))
        .bind(&input.source_ip)
        .bind(input.source_port)
        .bind(&input.destination_ip)
        .bind(input.destination_port)
        .bind(input.enabled.unwrap_or(true))
        .bind(input.priority.unwrap_or(0))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_rule(&self, id: Uuid) -> Result<Option<FirewallRule>, sqlx::Error> {
        let row = sqlx::query_as::<_, FirewallRuleRow>(
            r#"SELECT id, name, description, action, protocol, source_ip, source_port, destination_ip, destination_port, enabled, priority, created_at
             FROM firewall_rules WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_rules(&self) -> Result<Vec<FirewallRule>, sqlx::Error> {
        let rows = sqlx::query_as::<_, FirewallRuleRow>(
            r#"SELECT id, name, description, action, protocol, source_ip, source_port, destination_ip, destination_port, enabled, priority, created_at
             FROM firewall_rules ORDER BY priority DESC, created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_rule(
        &self,
        id: Uuid,
        input: UpdateFirewallRule,
    ) -> Result<FirewallRule, sqlx::Error> {
        let row = sqlx::query_as::<_, FirewallRuleRow>(
            r#"UPDATE firewall_rules SET
             name = COALESCE($2, name),
             description = COALESCE($3, description),
             action = COALESCE($4, action),
             protocol = COALESCE($5, protocol),
             source_ip = COALESCE($6, source_ip),
             source_port = COALESCE($7, source_port),
             destination_ip = COALESCE($8, destination_ip),
             destination_port = COALESCE($9, destination_port),
             enabled = COALESCE($10, enabled),
             priority = COALESCE($11, priority)
             WHERE id = $1
             RETURNING id, name, description, action, protocol, source_ip, source_port, destination_ip, destination_port, enabled, priority, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.action)
        .bind(&input.protocol)
        .bind(&input.source_ip)
        .bind(input.source_port)
        .bind(&input.destination_ip)
        .bind(input.destination_port)
        .bind(input.enabled)
        .bind(input.priority)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_rule(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM firewall_rules WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn evaluate(
        &self,
        source_ip: &str,
        destination_ip: Option<&str>,
        destination_port: Option<i32>,
        protocol: &str,
    ) -> Result<FirewallEvaluation, sqlx::Error> {
        let rows = sqlx::query_as::<_, FirewallRuleRow>(
            r#"SELECT id, name, description, action, protocol, source_ip, source_port, destination_ip, destination_port, enabled, priority, created_at
             FROM firewall_rules WHERE enabled = true ORDER BY priority DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        for row in rows {
            if Self::matches_rule(&row, source_ip, destination_ip, destination_port, protocol) {
                return Ok(FirewallEvaluation {
                    allowed: row.action == "allow",
                    matched_rule_id: Some(row.id),
                    action: row.action,
                });
            }
        }

        Ok(FirewallEvaluation {
            allowed: true,
            matched_rule_id: None,
            action: "allow".into(),
        })
    }

    pub async fn test_rule(
        &self,
        rule_id: Uuid,
        test: &FirewallRuleTest,
    ) -> Result<FirewallEvaluation, sqlx::Error> {
        let row = sqlx::query_as::<_, FirewallRuleRow>(
            r#"SELECT id, name, description, action, protocol, source_ip, source_port, destination_ip, destination_port, enabled, priority, created_at
             FROM firewall_rules WHERE id = $1"#,
        )
        .bind(rule_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => {
                let matches = Self::matches_rule(
                    &r,
                    &test.source_ip,
                    test.destination_ip.as_deref(),
                    test.destination_port,
                    &test.protocol,
                );
                Ok(FirewallEvaluation {
                    allowed: !matches || r.action == "allow",
                    matched_rule_id: Some(r.id),
                    action: if matches { r.action } else { "no_match".into() },
                })
            }
            None => Ok(FirewallEvaluation {
                allowed: true,
                matched_rule_id: None,
                action: "no_rule".into(),
            }),
        }
    }

    fn matches_rule(
        row: &FirewallRuleRow,
        source_ip: &str,
        destination_ip: Option<&str>,
        destination_port: Option<i32>,
        protocol: &str,
    ) -> bool {
        if row.protocol != protocol && row.protocol != "*" {
            return false;
        }

        if let Some(ref src_ip) = row.source_ip
            && src_ip != source_ip && src_ip != "*" && !Self::cidr_match(src_ip, source_ip) {
                return false;
            }

        if let Some(src_port) = row.source_port
            && let Some(tp) = destination_port
                && src_port != tp {
                    return false;
                }

        if let Some(ref dst_ip) = row.destination_ip {
            if let Some(dip) = destination_ip {
                if dst_ip != dip && dst_ip != "*" && !Self::cidr_match(dst_ip, dip) {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(dst_port) = row.destination_port
            && let Some(tp) = destination_port
                && dst_port != tp {
                    return false;
                }

        true
    }

    fn cidr_match(cidr: &str, ip: &str) -> bool {
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() != 2 {
            return cidr == ip;
        }
        let network = parts[0];
        let ip_parts: Vec<&str> = ip.split('.').collect();
        let network_parts: Vec<&str> = network.split('.').collect();
        if ip_parts.len() != 4 || network_parts.len() != 4 {
            return false;
        }
        if let Ok(mask) = parts[1].parse::<u32>() {
            let ip_val = Self::ip_to_u32(&ip_parts);
            let net_val = Self::ip_to_u32(&network_parts);
            let mask_bits = 0xFFFFFFFF << (32 - mask);
            return (ip_val & mask_bits) == (net_val & mask_bits);
        }
        false
    }

    fn ip_to_u32(parts: &[&str]) -> u32 {
        let mut result = 0u32;
        for part in parts {
            if let Ok(octet) = part.parse::<u8>() {
                result = (result << 8) | octet as u32;
            }
        }
        result
    }

    pub async fn log_rule_action(
        &self,
        rule_id: Uuid,
        source_ip: &str,
        destination_ip: Option<&str>,
        destination_port: Option<i32>,
        action_taken: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO firewall_rule_logs (rule_id, source_ip, destination_ip, destination_port, action_taken)
             VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(rule_id)
        .bind(source_ip)
        .bind(destination_ip)
        .bind(destination_port)
        .bind(action_taken)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firewall_rule_serialization() {
        let rule = FirewallRule {
            id: Uuid::new_v4(),
            name: "test-rule".into(),
            description: "desc".into(),
            action: "allow".into(),
            protocol: "tcp".into(),
            source_ip: Some("10.0.0.0/8".into()),
            source_port: None,
            destination_ip: None,
            destination_port: Some(443),
            enabled: true,
            priority: 100,
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&rule).unwrap();
        assert!(json.contains("allow"));
        assert!(json.contains("tcp"));
    }

    #[test]
    fn test_cidr_match() {
        assert!(FirewallService::cidr_match("10.0.0.0/8", "10.1.2.3"));
        assert!(!FirewallService::cidr_match("10.0.0.0/8", "192.168.1.1"));
        assert!(FirewallService::cidr_match("192.168.1.0/24", "192.168.1.100"));
        assert!(!FirewallService::cidr_match("192.168.1.0/24", "192.168.2.1"));
    }

    #[test]
    fn test_ip_to_u32() {
        let ip = FirewallService::ip_to_u32(&["192", "168", "1", "1"]);
        assert_eq!(ip, 0xC0A80101);
    }

    #[test]
    fn test_matches_rule_all_fields() {
        let row = FirewallRuleRow {
            id: Uuid::new_v4(),
            name: "test".into(),
            description: String::new(),
            action: "allow".into(),
            protocol: "tcp".into(),
            source_ip: Some("10.0.0.0/8".into()),
            source_port: None,
            destination_ip: None,
            destination_port: Some(443),
            enabled: true,
            priority: 0,
            created_at: chrono::Utc::now(),
        };
        assert!(FirewallService::matches_rule(&row, "10.1.2.3", None, Some(443), "tcp"));
        assert!(!FirewallService::matches_rule(&row, "192.168.1.1", None, Some(443), "tcp"));
        assert!(!FirewallService::matches_rule(&row, "10.1.2.3", None, Some(80), "tcp"));
        assert!(!FirewallService::matches_rule(&row, "10.1.2.3", None, Some(443), "udp"));
    }

    #[test]
    fn test_matches_rule_wildcard() {
        let row = FirewallRuleRow {
            id: Uuid::new_v4(),
            name: "test".into(),
            description: String::new(),
            action: "allow".into(),
            protocol: "*".into(),
            source_ip: None,
            source_port: None,
            destination_ip: None,
            destination_port: None,
            enabled: true,
            priority: 0,
            created_at: chrono::Utc::now(),
        };
        assert!(FirewallService::matches_rule(&row, "1.2.3.4", None, None, "tcp"));
        assert!(FirewallService::matches_rule(&row, "1.2.3.4", None, None, "udp"));
    }
}
