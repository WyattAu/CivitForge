use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::types::*;

#[derive(Debug, sqlx::FromRow)]
struct DashboardRow {
    id: Uuid,
    name: String,
    description: String,
    widgets: serde_json::Value,
    layout: serde_json::Value,
    is_public: bool,
    created_at: DateTime<Utc>,
}

impl From<DashboardRow> for Dashboard {
    fn from(row: DashboardRow) -> Self {
        Dashboard {
            id: row.id,
            name: row.name,
            description: row.description,
            widgets: row.widgets,
            layout: row.layout,
            is_public: row.is_public,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ReportRow {
    id: Uuid,
    name: String,
    report_type: String,
    config: serde_json::Value,
    schedule: Option<String>,
    last_generated_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<ReportRow> for Report {
    fn from(row: ReportRow) -> Self {
        Report {
            id: row.id,
            name: row.name,
            report_type: row.report_type,
            config: row.config,
            schedule: row.schedule,
            last_generated_at: row.last_generated_at,
            created_at: row.created_at,
        }
    }
}

pub struct DashboardReportingService {
    pool: PgPool,
}

impl DashboardReportingService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_dashboard(
        &self,
        input: CreateDashboard,
    ) -> Result<Dashboard, sqlx::Error> {
        let row = sqlx::query_as::<_, DashboardRow>(
            r#"INSERT INTO dashboards (name, description, widgets, layout, is_public)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, name, description, widgets, layout, is_public, created_at"#,
        )
        .bind(&input.name)
        .bind(input.description.as_deref().unwrap_or(""))
        .bind(input.widgets.unwrap_or(serde_json::json!([])))
        .bind(input.layout.unwrap_or(serde_json::json!({})))
        .bind(input.is_public.unwrap_or(false))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_dashboard(
        &self,
        id: Uuid,
    ) -> Result<Option<Dashboard>, sqlx::Error> {
        let row = sqlx::query_as::<_, DashboardRow>(
            r#"SELECT id, name, description, widgets, layout, is_public, created_at
             FROM dashboards WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_dashboards(&self) -> Result<Vec<Dashboard>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DashboardRow>(
            r#"SELECT id, name, description, widgets, layout, is_public, created_at
             FROM dashboards ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_dashboard(
        &self,
        id: Uuid,
        input: UpdateDashboard,
    ) -> Result<Dashboard, sqlx::Error> {
        let row = sqlx::query_as::<_, DashboardRow>(
            r#"UPDATE dashboards SET
             name = COALESCE($2, name),
             description = COALESCE($3, description),
             widgets = COALESCE($4, widgets),
             layout = COALESCE($5, layout),
             is_public = COALESCE($6, is_public)
             WHERE id = $1
             RETURNING id, name, description, widgets, layout, is_public, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.widgets)
        .bind(&input.layout)
        .bind(input.is_public)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_dashboard(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM dashboards WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn create_report(
        &self,
        input: CreateReport,
    ) -> Result<Report, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportRow>(
            r#"INSERT INTO reports (name, report_type, config, schedule)
             VALUES ($1, $2, $3, $4)
             RETURNING id, name, report_type, config, schedule, last_generated_at, created_at"#,
        )
        .bind(&input.name)
        .bind(&input.report_type)
        .bind(input.config.unwrap_or(serde_json::json!({})))
        .bind(&input.schedule)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_report(
        &self,
        id: Uuid,
    ) -> Result<Option<Report>, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportRow>(
            r#"SELECT id, name, report_type, config, schedule, last_generated_at, created_at
             FROM reports WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_reports(&self) -> Result<Vec<Report>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReportRow>(
            r#"SELECT id, name, report_type, config, schedule, last_generated_at, created_at
             FROM reports ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_report(
        &self,
        id: Uuid,
        input: UpdateReport,
    ) -> Result<Report, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportRow>(
            r#"UPDATE reports SET
             name = COALESCE($2, name),
             report_type = COALESCE($3, report_type),
             config = COALESCE($4, config),
             schedule = COALESCE($5, schedule)
             WHERE id = $1
             RETURNING id, name, report_type, config, schedule, last_generated_at, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.report_type)
        .bind(&input.config)
        .bind(&input.schedule)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_report(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM reports WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn generate_report(
        &self,
        id: Uuid,
    ) -> Result<ReportData, sqlx::Error> {
        let report = self.get_report(id).await?;
        let report = report.ok_or_else(|| sqlx::Error::RowNotFound)?;

        sqlx::query("UPDATE reports SET last_generated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        let data = serde_json::json!({
            "report_type": report.report_type,
            "config": report.config,
            "generated_at": Utc::now()
        });

        Ok(ReportData {
            report_id: id,
            name: report.name,
            report_type: report.report_type,
            data,
            generated_at: Utc::now(),
        })
    }

    pub async fn get_scheduled_reports(
        &self,
    ) -> Result<Vec<Report>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReportRow>(
            r#"SELECT id, name, report_type, config, schedule, last_generated_at, created_at
             FROM reports WHERE schedule IS NOT NULL
             ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_dashboard_stats(
        &self,
    ) -> Result<DashboardStats, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct StatsRow {
            total_dashboards: i64,
            public_dashboards: i64,
        }

        #[derive(Debug, sqlx::FromRow)]
        struct ReportStatsRow {
            total_reports: i64,
            scheduled_reports: i64,
        }

        let dash_stats = sqlx::query_as::<_, StatsRow>(
            r#"SELECT
             COUNT(*) as total_dashboards,
             COUNT(*) FILTER (WHERE is_public) as public_dashboards
             FROM dashboards"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let report_stats = sqlx::query_as::<_, ReportStatsRow>(
            r#"SELECT
             COUNT(*) as total_reports,
             COUNT(*) FILTER (WHERE schedule IS NOT NULL) as scheduled_reports
             FROM reports"#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DashboardStats {
            total_dashboards: dash_stats.total_dashboards,
            public_dashboards: dash_stats.public_dashboards,
            total_reports: report_stats.total_reports,
            scheduled_reports: report_stats.scheduled_reports,
        })
    }
}
