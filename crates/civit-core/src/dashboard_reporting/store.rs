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

#[derive(Debug, sqlx::FromRow)]
struct WidgetV2Row {
    id: Uuid,
    dashboard_id: Uuid,
    widget_type: String,
    config: serde_json::Value,
    position: serde_json::Value,
    size: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl From<WidgetV2Row> for DashboardWidgetV2 {
    fn from(row: WidgetV2Row) -> Self {
        DashboardWidgetV2 {
            id: row.id,
            dashboard_id: row.dashboard_id,
            widget_type: row.widget_type,
            config: row.config,
            position: row.position,
            size: row.size,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ReportScheduleV2Row {
    id: Uuid,
    report_id: Uuid,
    cron_expression: String,
    enabled: bool,
    last_run_at: Option<DateTime<Utc>>,
    next_run_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl From<ReportScheduleV2Row> for ReportScheduleV2 {
    fn from(row: ReportScheduleV2Row) -> Self {
        ReportScheduleV2 {
            id: row.id,
            report_id: row.report_id,
            cron_expression: row.cron_expression,
            enabled: row.enabled,
            last_run_at: row.last_run_at,
            next_run_at: row.next_run_at,
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

    // V2: Widget configuration

    pub async fn create_widget_v2(
        &self,
        input: CreateDashboardWidgetV2,
    ) -> Result<DashboardWidgetV2, sqlx::Error> {
        let row = sqlx::query_as::<_, WidgetV2Row>(
            r#"INSERT INTO dashboard_widgets_v2 (dashboard_id, widget_type, config, position, size)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, dashboard_id, widget_type, config, position, size, created_at"#,
        )
        .bind(input.dashboard_id)
        .bind(&input.widget_type)
        .bind(input.config.unwrap_or(serde_json::json!({})))
        .bind(input.position.unwrap_or(serde_json::json!({"x": 0, "y": 0})))
        .bind(input.size.unwrap_or(serde_json::json!({"width": 6, "height": 4})))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_widget_v2(
        &self,
        id: Uuid,
    ) -> Result<Option<DashboardWidgetV2>, sqlx::Error> {
        let row = sqlx::query_as::<_, WidgetV2Row>(
            r#"SELECT id, dashboard_id, widget_type, config, position, size, created_at
             FROM dashboard_widgets_v2 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_widgets_for_dashboard(
        &self,
        dashboard_id: Uuid,
    ) -> Result<Vec<DashboardWidgetV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WidgetV2Row>(
            r#"SELECT id, dashboard_id, widget_type, config, position, size, created_at
             FROM dashboard_widgets_v2 WHERE dashboard_id = $1
             ORDER BY created_at ASC"#,
        )
        .bind(dashboard_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_widget_v2(
        &self,
        id: Uuid,
        input: UpdateDashboardWidgetV2,
    ) -> Result<DashboardWidgetV2, sqlx::Error> {
        let row = sqlx::query_as::<_, WidgetV2Row>(
            r#"UPDATE dashboard_widgets_v2 SET
             widget_type = COALESCE($2, widget_type),
             config = COALESCE($3, config),
             position = COALESCE($4, position),
             size = COALESCE($5, size)
             WHERE id = $1
             RETURNING id, dashboard_id, widget_type, config, position, size, created_at"#,
        )
        .bind(id)
        .bind(&input.widget_type)
        .bind(&input.config)
        .bind(&input.position)
        .bind(&input.size)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_widget_v2(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM dashboard_widgets_v2 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // V2: Report scheduling

    pub async fn create_report_schedule(
        &self,
        input: CreateReportScheduleV2,
    ) -> Result<ReportScheduleV2, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV2Row>(
            r#"INSERT INTO report_schedules (report_id, cron_expression, enabled, next_run_at)
             VALUES ($1, $2, $3, $4)
             RETURNING id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at"#,
        )
        .bind(input.report_id)
        .bind(&input.cron_expression)
        .bind(input.enabled.unwrap_or(true))
        .bind(input.next_run_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_report_schedule(
        &self,
        id: Uuid,
    ) -> Result<Option<ReportScheduleV2>, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV2Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_report_schedules(
        &self,
    ) -> Result<Vec<ReportScheduleV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReportScheduleV2Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_due_schedules(
        &self,
    ) -> Result<Vec<ReportScheduleV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReportScheduleV2Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules WHERE enabled = true AND next_run_at <= NOW()
             ORDER BY next_run_at ASC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_report_schedule(
        &self,
        id: Uuid,
        input: UpdateReportScheduleV2,
    ) -> Result<ReportScheduleV2, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV2Row>(
            r#"UPDATE report_schedules SET
             cron_expression = COALESCE($2, cron_expression),
             enabled = COALESCE($3, enabled),
             next_run_at = COALESCE($4, next_run_at)
             WHERE id = $1
             RETURNING id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at"#,
        )
        .bind(id)
        .bind(&input.cron_expression)
        .bind(input.enabled)
        .bind(input.next_run_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn mark_schedule_executed(
        &self,
        id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE report_schedules SET last_run_at = NOW() WHERE id = $1"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_report_schedule(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM report_schedules WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // V2: Report export

    pub async fn export_report(
        &self,
        request: ReportExportRequest,
    ) -> Result<ReportExportResult, sqlx::Error> {
        let report = self.get_report(request.report_id).await?;
        let report = report.ok_or_else(|| sqlx::Error::RowNotFound)?;

        let data = serde_json::json!({
            "report": {
                "id": report.id,
                "name": report.name,
                "report_type": report.report_type,
                "config": report.config,
            },
            "format": request.format,
            "filters": {
                "since": request.since,
                "until": request.until,
            },
            "generated_at": Utc::now()
        });

        Ok(ReportExportResult {
            report_id: request.report_id,
            format: request.format,
            data,
            exported_at: Utc::now(),
        })
    }

    pub async fn list_public_dashboards(
        &self,
    ) -> Result<Vec<Dashboard>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DashboardRow>(
            r#"SELECT id, name, description, widgets, layout, is_public, created_at
             FROM dashboards WHERE is_public = true
             ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}
