#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantResourceQuota {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub resource_type: String,
    pub quota_limit: i64,
    pub quota_used: i64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantIsolationPolicy {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub policy_type: String,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantBilling {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub plan: String,
    pub billing_cycle: String,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub amount_cents: i32,
    pub currency: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantUsageReport {
    pub tenant_id: Uuid,
    pub quotas: Vec<TenantResourceQuota>,
    pub policies: Vec<TenantIsolationPolicy>,
    pub billing: Option<TenantBilling>,
}

#[derive(Debug, sqlx::FromRow)]
struct TenantResourceQuotaRow {
    id: Uuid,
    tenant_id: Uuid,
    resource_type: String,
    quota_limit: i64,
    quota_used: i64,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl From<TenantResourceQuotaRow> for TenantResourceQuota {
    fn from(row: TenantResourceQuotaRow) -> Self {
        TenantResourceQuota {
            id: row.id,
            tenant_id: row.tenant_id,
            resource_type: row.resource_type,
            quota_limit: row.quota_limit,
            quota_used: row.quota_used,
            period_start: row.period_start,
            period_end: row.period_end,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct TenantIsolationPolicyRow {
    id: Uuid,
    tenant_id: Uuid,
    policy_type: String,
    config: serde_json::Value,
    enabled: bool,
    created_at: DateTime<Utc>,
}

impl From<TenantIsolationPolicyRow> for TenantIsolationPolicy {
    fn from(row: TenantIsolationPolicyRow) -> Self {
        TenantIsolationPolicy {
            id: row.id,
            tenant_id: row.tenant_id,
            policy_type: row.policy_type,
            config: row.config,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct TenantBillingRow {
    id: Uuid,
    tenant_id: Uuid,
    plan: String,
    billing_cycle: String,
    current_period_start: DateTime<Utc>,
    current_period_end: DateTime<Utc>,
    amount_cents: i32,
    currency: String,
    status: String,
    created_at: DateTime<Utc>,
}

impl From<TenantBillingRow> for TenantBilling {
    fn from(row: TenantBillingRow) -> Self {
        TenantBilling {
            id: row.id,
            tenant_id: row.tenant_id,
            plan: row.plan,
            billing_cycle: row.billing_cycle,
            current_period_start: row.current_period_start,
            current_period_end: row.current_period_end,
            amount_cents: row.amount_cents,
            currency: row.currency,
            status: row.status,
            created_at: row.created_at,
        }
    }
}

pub struct TenantIsolationService {
    pool: PgPool,
}

impl TenantIsolationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn check_quota(
        &self,
        tenant_id: Uuid,
        resource_type: &str,
    ) -> Result<Option<TenantResourceQuota>, sqlx::Error> {
        let now = Utc::now();
        let row = sqlx::query_as::<_, TenantResourceQuotaRow>(
            r#"SELECT id, tenant_id, resource_type, quota_limit, quota_used, period_start, period_end, created_at
             FROM tenant_resource_quotas_v1
             WHERE tenant_id = $1 AND resource_type = $2 AND period_start <= $3 AND period_end > $3
             ORDER BY period_start DESC LIMIT 1"#,
        )
        .bind(tenant_id)
        .bind(resource_type)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn consume_quota(
        &self,
        tenant_id: Uuid,
        resource_type: &str,
        amount: i64,
    ) -> Result<bool, sqlx::Error> {
        let now = Utc::now();
        let result = sqlx::query(
            r#"UPDATE tenant_resource_quotas_v1
             SET quota_used = quota_used + $1
             WHERE tenant_id = $2 AND resource_type = $3 AND period_start <= $4 AND period_end > $4
               AND quota_used + $1 <= quota_limit
             RETURNING id"#,
        )
        .bind(amount)
        .bind(tenant_id)
        .bind(resource_type)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.is_some())
    }

    pub async fn reset_quotas(
        &self,
        tenant_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE tenant_resource_quotas_v1
             SET quota_used = 0, period_start = $1, period_end = $2
             WHERE tenant_id = $3"#,
        )
        .bind(period_start)
        .bind(period_end)
        .bind(tenant_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn set_policy(
        &self,
        tenant_id: Uuid,
        policy_type: &str,
        config: serde_json::Value,
    ) -> Result<TenantIsolationPolicy, sqlx::Error> {
        let row = sqlx::query_as::<_, TenantIsolationPolicyRow>(
            r#"INSERT INTO tenant_isolation_policies_v1 (tenant_id, policy_type, config)
             VALUES ($1, $2, $3)
             ON CONFLICT (tenant_id, policy_type) DO UPDATE SET config = $3
             RETURNING id, tenant_id, policy_type, config, enabled, created_at"#,
        )
        .bind(tenant_id)
        .bind(policy_type)
        .bind(&config)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_policies(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<TenantIsolationPolicy>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TenantIsolationPolicyRow>(
            r#"SELECT id, tenant_id, policy_type, config, enabled, created_at
             FROM tenant_isolation_policies_v1 WHERE tenant_id = $1
             ORDER BY created_at DESC"#,
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_billing(
        &self,
        tenant_id: Uuid,
        plan: &str,
        amount_cents: i32,
    ) -> Result<TenantBilling, sqlx::Error> {
        let now = Utc::now();
        let period_end = now + chrono::Duration::days(30);
        let row = sqlx::query_as::<_, TenantBillingRow>(
            r#"INSERT INTO tenant_billing_v1 (tenant_id, plan, current_period_start, current_period_end, amount_cents)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (tenant_id) DO UPDATE SET plan = $2, current_period_start = $3, current_period_end = $4, amount_cents = $5
             RETURNING id, tenant_id, plan, billing_cycle, current_period_start, current_period_end, amount_cents, currency, status, created_at"#,
        )
        .bind(tenant_id)
        .bind(plan)
        .bind(now)
        .bind(period_end)
        .bind(amount_cents)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_billing_status(
        &self,
        tenant_id: Uuid,
    ) -> Result<Option<TenantBilling>, sqlx::Error> {
        let row = sqlx::query_as::<_, TenantBillingRow>(
            r#"SELECT id, tenant_id, plan, billing_cycle, current_period_start, current_period_end, amount_cents, currency, status, created_at
             FROM tenant_billing_v1 WHERE tenant_id = $1
             ORDER BY created_at DESC LIMIT 1"#,
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn get_tenant_usage(
        &self,
        tenant_id: Uuid,
    ) -> Result<TenantUsageReport, sqlx::Error> {
        let now = Utc::now();
        let quota_rows = sqlx::query_as::<_, TenantResourceQuotaRow>(
            r#"SELECT id, tenant_id, resource_type, quota_limit, quota_used, period_start, period_end, created_at
             FROM tenant_resource_quotas_v1
             WHERE tenant_id = $1 AND period_start <= $2 AND period_end > $2"#,
        )
        .bind(tenant_id)
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        let policy_rows = sqlx::query_as::<_, TenantIsolationPolicyRow>(
            r#"SELECT id, tenant_id, policy_type, config, enabled, created_at
             FROM tenant_isolation_policies_v1 WHERE tenant_id = $1"#,
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let billing_row = sqlx::query_as::<_, TenantBillingRow>(
            r#"SELECT id, tenant_id, plan, billing_cycle, current_period_start, current_period_end, amount_cents, currency, status, created_at
             FROM tenant_billing_v1 WHERE tenant_id = $1
             ORDER BY created_at DESC LIMIT 1"#,
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(TenantUsageReport {
            tenant_id,
            quotas: quota_rows.into_iter().map(|r| r.into()).collect(),
            policies: policy_rows.into_iter().map(|r| r.into()).collect(),
            billing: billing_row.map(|r| r.into()),
        })
    }
}
