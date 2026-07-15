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

    // V3: Template management

    pub async fn create_dashboard_template(
        &self,
        input: CreateDashboardTemplate,
    ) -> Result<DashboardTemplate, sqlx::Error> {
        let row = sqlx::query_as::<_, DashboardTemplateRow>(
            r#"INSERT INTO dashboard_templates (name, description, template_type, config, is_public, author_id)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, description, template_type, config, is_public, author_id, usage_count, created_at"#,
        )
        .bind(&input.name)
        .bind(input.description.as_deref().unwrap_or(""))
        .bind(&input.template_type)
        .bind(input.config.unwrap_or(serde_json::json!({})))
        .bind(input.is_public.unwrap_or(false))
        .bind(input.author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_dashboard_template(
        &self,
        id: Uuid,
    ) -> Result<Option<DashboardTemplate>, sqlx::Error> {
        let row = sqlx::query_as::<_, DashboardTemplateRow>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, created_at
             FROM dashboard_templates WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_dashboard_templates(
        &self,
    ) -> Result<Vec<DashboardTemplate>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DashboardTemplateRow>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, created_at
             FROM dashboard_templates ORDER BY usage_count DESC, created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_dashboard_template(
        &self,
        id: Uuid,
        input: UpdateDashboardTemplate,
    ) -> Result<DashboardTemplate, sqlx::Error> {
        let row = sqlx::query_as::<_, DashboardTemplateRow>(
            r#"UPDATE dashboard_templates SET
             name = COALESCE($2, name),
             description = COALESCE($3, description),
             template_type = COALESCE($4, template_type),
             config = COALESCE($5, config),
             is_public = COALESCE($6, is_public)
             WHERE id = $1
             RETURNING id, name, description, template_type, config, is_public, author_id, usage_count, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.template_type)
        .bind(&input.config)
        .bind(input.is_public)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_dashboard_template(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM dashboard_templates WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn use_dashboard_template(
        &self,
        id: Uuid,
    ) -> Result<DashboardTemplate, sqlx::Error> {
        sqlx::query("UPDATE dashboard_templates SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        self.get_dashboard_template(id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn create_report_template(
        &self,
        input: CreateReportTemplate,
    ) -> Result<ReportTemplate, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportTemplateRow>(
            r#"INSERT INTO report_templates (name, description, report_type, config, is_public, author_id)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, description, report_type, config, is_public, author_id, usage_count, created_at"#,
        )
        .bind(&input.name)
        .bind(input.description.as_deref().unwrap_or(""))
        .bind(&input.report_type)
        .bind(input.config.unwrap_or(serde_json::json!({})))
        .bind(input.is_public.unwrap_or(false))
        .bind(input.author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_report_template(
        &self,
        id: Uuid,
    ) -> Result<Option<ReportTemplate>, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportTemplateRow>(
            r#"SELECT id, name, description, report_type, config, is_public, author_id, usage_count, created_at
             FROM report_templates WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_report_templates(
        &self,
    ) -> Result<Vec<ReportTemplate>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReportTemplateRow>(
            r#"SELECT id, name, description, report_type, config, is_public, author_id, usage_count, created_at
             FROM report_templates ORDER BY usage_count DESC, created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_report_template(
        &self,
        id: Uuid,
        input: UpdateReportTemplate,
    ) -> Result<ReportTemplate, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportTemplateRow>(
            r#"UPDATE report_templates SET
             name = COALESCE($2, name),
             description = COALESCE($3, description),
             report_type = COALESCE($4, report_type),
             config = COALESCE($5, config),
             is_public = COALESCE($6, is_public)
             WHERE id = $1
             RETURNING id, name, description, report_type, config, is_public, author_id, usage_count, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.report_type)
        .bind(&input.config)
        .bind(input.is_public)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_report_template(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM report_templates WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn use_report_template(
        &self,
        id: Uuid,
    ) -> Result<ReportTemplate, sqlx::Error> {
        sqlx::query("UPDATE report_templates SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        self.get_report_template(id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    // V3: Template marketplace

    pub async fn list_marketplace_templates(
        &self,
        template_type: Option<&str>,
    ) -> Result<Vec<TemplateMarketplaceItem>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct MarketplaceRow {
            id: Uuid,
            name: String,
            description: String,
            template_type: String,
            author_id: Option<Uuid>,
            usage_count: i64,
        }

        let rows = if let Some(t) = template_type {
            sqlx::query_as::<_, MarketplaceRow>(
                r#"SELECT id, name, description, template_type, author_id, usage_count
                 FROM dashboard_templates WHERE is_public = true AND template_type = $1
                 ORDER BY usage_count DESC"#,
            )
            .bind(t)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, MarketplaceRow>(
                r#"SELECT id, name, description, template_type, author_id, usage_count
                 FROM dashboard_templates WHERE is_public = true
                 ORDER BY usage_count DESC"#,
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows
            .into_iter()
            .map(|r| TemplateMarketplaceItem {
                id: r.id,
                name: r.name,
                description: r.description,
                template_type: r.template_type,
                author_id: r.author_id,
                usage_count: r.usage_count,
                rating: None,
                tags: vec![],
            })
            .collect())
    }

    pub async fn share_dashboard(
        &self,
        dashboard_id: Uuid,
        user_id: Uuid,
        permission: DashboardPermission,
    ) -> Result<DashboardShare, sqlx::Error> {
        let id = Uuid::new_v4();
        let permission_str = match permission {
            DashboardPermission::View => "view",
            DashboardPermission::Edit => "edit",
            DashboardPermission::Admin => "admin",
        };

        sqlx::query(
            r#"INSERT INTO dashboard_shares (id, dashboard_id, user_id, permission)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (dashboard_id, user_id) DO UPDATE SET permission = $4"#,
        )
        .bind(id)
        .bind(dashboard_id)
        .bind(user_id)
        .bind(permission_str)
        .execute(&self.pool)
        .await?;

        Ok(DashboardShare {
            id,
            dashboard_id,
            user_id,
            permission,
            shared_at: Utc::now(),
        })
    }

    pub async fn get_dashboard_stats_v3(
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

        #[derive(Debug, sqlx::FromRow)]
        struct TemplateStatsRow {
            total_templates: i64,
            public_templates: i64,
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

    // V4: Dashboard sharing

    pub async fn share_dashboard_v4(
        &self,
        dashboard_id: Uuid,
        user_id: Uuid,
        permission: DashboardPermission,
    ) -> Result<DashboardShare, sqlx::Error> {
        let id = Uuid::new_v4();
        let permission_str = match permission {
            DashboardPermission::View => "view",
            DashboardPermission::Edit => "edit",
            DashboardPermission::Admin => "admin",
        };

        sqlx::query(
            r#"INSERT INTO dashboard_shares (id, dashboard_id, user_id, permission)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (dashboard_id, user_id) DO UPDATE SET permission = $4"#,
        )
        .bind(id)
        .bind(dashboard_id)
        .bind(user_id)
        .bind(permission_str)
        .execute(&self.pool)
        .await?;

        Ok(DashboardShare {
            id,
            dashboard_id,
            user_id,
            permission,
            shared_at: Utc::now(),
        })
    }

    pub async fn get_dashboard_shares(
        &self,
        dashboard_id: Uuid,
    ) -> Result<Vec<DashboardShare>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct ShareRow {
            id: Uuid,
            dashboard_id: Uuid,
            user_id: Uuid,
            permission: String,
            created_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, ShareRow>(
            r#"SELECT id, dashboard_id, user_id, permission, created_at
             FROM dashboard_shares WHERE dashboard_id = $1
             ORDER BY created_at DESC"#,
        )
        .bind(dashboard_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| DashboardShare {
                id: r.id,
                dashboard_id: r.dashboard_id,
                user_id: r.user_id,
                permission: match r.permission.as_str() {
                    "edit" => DashboardPermission::Edit,
                    "admin" => DashboardPermission::Admin,
                    _ => DashboardPermission::View,
                },
                shared_at: r.created_at,
            })
            .collect())
    }

    pub async fn remove_dashboard_share(
        &self,
        dashboard_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM dashboard_shares WHERE dashboard_id = $1 AND user_id = $2",
        )
        .bind(dashboard_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // V4: Report scheduling v2

    pub async fn create_report_schedule_v2(
        &self,
        input: CreateReportScheduleV2,
    ) -> Result<ReportScheduleV2, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV2Row>(
            r#"INSERT INTO report_schedules_v2 (report_id, cron_expression, enabled, next_run_at)
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

    pub async fn get_report_schedule_v2(
        &self,
        id: Uuid,
    ) -> Result<Option<ReportScheduleV2>, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV2Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v2 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_report_schedules_v2(
        &self,
    ) -> Result<Vec<ReportScheduleV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReportScheduleV2Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v2 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_due_schedules_v2(
        &self,
    ) -> Result<Vec<ReportScheduleV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReportScheduleV2Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v2 WHERE enabled = true AND next_run_at <= NOW()
             ORDER BY next_run_at ASC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn mark_schedule_executed_v2(
        &self,
        id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE report_schedules_v2 SET last_run_at = NOW() WHERE id = $1"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_report_schedule_v2(
        &self,
        id: Uuid,
        input: UpdateReportScheduleV2,
    ) -> Result<ReportScheduleV2, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV2Row>(
            r#"UPDATE report_schedules_v2 SET
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

    pub async fn delete_report_schedule_v2(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM report_schedules_v2 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // V5: Dashboard sharing v2

    pub async fn share_dashboard_v5(
        &self,
        dashboard_id: Uuid,
        user_id: Uuid,
        permission: &str,
    ) -> Result<DashboardShareV2, sqlx::Error> {
        let row = sqlx::query_as::<_, DashboardShareV2Row>(
            r#"INSERT INTO dashboard_shares_v2 (dashboard_id, user_id, permission)
             VALUES ($1, $2, $3)
             ON CONFLICT (dashboard_id, user_id) DO UPDATE SET permission = $3
             RETURNING id, dashboard_id, user_id, permission, created_at"#,
        )
        .bind(dashboard_id)
        .bind(user_id)
        .bind(permission)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_dashboard_shares_v2(
        &self,
        dashboard_id: Uuid,
    ) -> Result<Vec<DashboardShareV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DashboardShareV2Row>(
            r#"SELECT id, dashboard_id, user_id, permission, created_at
             FROM dashboard_shares_v2 WHERE dashboard_id = $1
             ORDER BY created_at DESC"#,
        )
        .bind(dashboard_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn remove_dashboard_share_v2(
        &self,
        dashboard_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM dashboard_shares_v2 WHERE dashboard_id = $1 AND user_id = $2",
        )
        .bind(dashboard_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_dashboards_shared_with_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Dashboard>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DashboardRow>(
            r#"SELECT d.id, d.name, d.description, d.widgets, d.layout, d.is_public, d.created_at
             FROM dashboards d
             INNER JOIN dashboard_shares_v2 ds ON d.id = ds.dashboard_id
             WHERE ds.user_id = $1
             ORDER BY ds.created_at DESC"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    // V5: Report scheduling v3

    pub async fn create_report_schedule_v3(
        &self,
        input: CreateReportScheduleV3,
    ) -> Result<ReportScheduleV3, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV3Row>(
            r#"INSERT INTO report_schedules_v3 (report_id, cron_expression, enabled, next_run_at)
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

    pub async fn get_report_schedule_v3(
        &self,
        id: Uuid,
    ) -> Result<Option<ReportScheduleV3>, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV3Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v3 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn list_report_schedules_v3(
        &self,
    ) -> Result<Vec<ReportScheduleV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReportScheduleV3Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v3 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_due_schedules_v3(
        &self,
    ) -> Result<Vec<ReportScheduleV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReportScheduleV3Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v3 WHERE enabled = true AND next_run_at <= NOW()
             ORDER BY next_run_at ASC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn mark_schedule_executed_v3(
        &self,
        id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE report_schedules_v3 SET last_run_at = NOW() WHERE id = $1"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_report_schedule_v3(
        &self,
        id: Uuid,
        input: UpdateReportScheduleV3,
    ) -> Result<ReportScheduleV3, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV3Row>(
            r#"UPDATE report_schedules_v3 SET
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

    pub async fn delete_report_schedule_v3(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM report_schedules_v3 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // V5: Dashboard stats v5

    pub async fn get_dashboard_stats_v5(
        &self,
    ) -> Result<DashboardStatsV5, sqlx::Error> {
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
        #[derive(Debug, sqlx::FromRow)]
        struct ShareStatsRow {
            total_shares: i64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct ScheduleStatsRow {
            total_schedules: i64,
        }

        let dash_stats = sqlx::query_as::<_, StatsRow>(
            r#"SELECT COUNT(*) as total_dashboards,
             COUNT(*) FILTER (WHERE is_public) as public_dashboards
             FROM dashboards"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let report_stats = sqlx::query_as::<_, ReportStatsRow>(
            r#"SELECT COUNT(*) as total_reports,
             COUNT(*) FILTER (WHERE schedule IS NOT NULL) as scheduled_reports
             FROM reports"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let share_stats = sqlx::query_as::<_, ShareStatsRow>(
            r#"SELECT COUNT(*) as total_shares FROM dashboard_shares_v2"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let schedule_stats = sqlx::query_as::<_, ScheduleStatsRow>(
            r#"SELECT COUNT(*) as total_schedules FROM report_schedules_v3"#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DashboardStatsV5 {
            total_dashboards: dash_stats.total_dashboards,
            public_dashboards: dash_stats.public_dashboards,
            total_reports: report_stats.total_reports,
            scheduled_reports: report_stats.scheduled_reports,
            total_shares: share_stats.total_shares,
            total_schedules: schedule_stats.total_schedules,
        })
    }

    // V6: Dashboard sharing v3

    pub async fn share_dashboard_v6(
        &self,
        dashboard_id: Uuid,
        user_id: Uuid,
        permission: &str,
    ) -> Result<DashboardShareV3, sqlx::Error> {
        let row = sqlx::query_as::<_, DashboardShareV3Row>(
            r#"INSERT INTO dashboard_shares_v3 (dashboard_id, user_id, permission)
             VALUES ($1, $2, $3)
             ON CONFLICT (dashboard_id, user_id) DO UPDATE SET permission = $3
             RETURNING id, dashboard_id, user_id, permission, created_at"#,
        )
        .bind(dashboard_id)
        .bind(user_id)
        .bind(permission)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_dashboard_shares_v3(
        &self,
        dashboard_id: Uuid,
    ) -> Result<Vec<DashboardShareV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DashboardShareV3Row>(
            r#"SELECT id, dashboard_id, user_id, permission, created_at
             FROM dashboard_shares_v3 WHERE dashboard_id = $1
             ORDER BY created_at DESC"#,
        )
        .bind(dashboard_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn remove_dashboard_share_v3(
        &self,
        dashboard_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM dashboard_shares_v3 WHERE dashboard_id = $1 AND user_id = $2",
        )
        .bind(dashboard_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_dashboards_shared_with_user_v3(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Dashboard>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DashboardRow>(
            r#"SELECT d.id, d.name, d.description, d.widgets, d.layout, d.is_public, d.created_at
             FROM dashboards d
             INNER JOIN dashboard_shares_v3 ds ON d.id = ds.dashboard_id
             WHERE ds.user_id = $1
             ORDER BY ds.created_at DESC"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    // V6: Report scheduling v4

    pub async fn create_report_schedule_v4(
        &self,
        input: CreateReportScheduleV4,
    ) -> Result<ReportScheduleV4, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV4Row>(
            r#"INSERT INTO report_schedules_v4 (report_id, cron_expression, enabled, next_run_at)
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

    pub async fn get_report_schedule_v4(
        &self,
        id: Uuid,
    ) -> Result<Option<ReportScheduleV4>, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV4Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v4 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn list_report_schedules_v4(
        &self,
    ) -> Result<Vec<ReportScheduleV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReportScheduleV4Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v4 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_due_schedules_v4(
        &self,
    ) -> Result<Vec<ReportScheduleV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReportScheduleV4Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v4 WHERE enabled = true AND next_run_at <= NOW()
             ORDER BY next_run_at ASC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn mark_schedule_executed_v4(
        &self,
        id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE report_schedules_v4 SET last_run_at = NOW() WHERE id = $1"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_report_schedule_v4(
        &self,
        id: Uuid,
        input: UpdateReportScheduleV4,
    ) -> Result<ReportScheduleV4, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV4Row>(
            r#"UPDATE report_schedules_v4 SET
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

    pub async fn delete_report_schedule_v4(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM report_schedules_v4 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // V6: Dashboard stats v6

    pub async fn get_dashboard_stats_v6(
        &self,
    ) -> Result<DashboardStatsV6, sqlx::Error> {
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
        #[derive(Debug, sqlx::FromRow)]
        struct ShareStatsRow {
            total_shares: i64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct ScheduleStatsRow {
            total_schedules: i64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct AvgShareRow {
            avg_shares: f64,
        }

        let dash_stats = sqlx::query_as::<_, StatsRow>(
            r#"SELECT COUNT(*) as total_dashboards,
             COUNT(*) FILTER (WHERE is_public) as public_dashboards
             FROM dashboards"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let report_stats = sqlx::query_as::<_, ReportStatsRow>(
            r#"SELECT COUNT(*) as total_reports,
             COUNT(*) FILTER (WHERE schedule IS NOT NULL) as scheduled_reports
             FROM reports"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let share_stats = sqlx::query_as::<_, ShareStatsRow>(
            r#"SELECT COUNT(*) as total_shares FROM dashboard_shares_v3"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let schedule_stats = sqlx::query_as::<_, ScheduleStatsRow>(
            r#"SELECT COUNT(*) as total_schedules FROM report_schedules_v4"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let avg_shares = sqlx::query_as::<_, AvgShareRow>(
            r#"SELECT COALESCE(COUNT(ds.id)::float / NULLIF(COUNT(DISTINCT d.id), 0), 0.0) as avg_shares
             FROM dashboards d LEFT JOIN dashboard_shares_v3 ds ON d.id = ds.dashboard_id"#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DashboardStatsV6 {
            total_dashboards: dash_stats.total_dashboards,
            public_dashboards: dash_stats.public_dashboards,
            total_reports: report_stats.total_reports,
            scheduled_reports: report_stats.scheduled_reports,
            total_shares: share_stats.total_shares,
            total_schedules: schedule_stats.total_schedules,
            avg_shares_per_dashboard: avg_shares.avg_shares,
        })
    }

    // V7: Dashboard stats v7

    pub async fn get_dashboard_stats_v7(
        &self,
    ) -> Result<DashboardStatsV7, sqlx::Error> {
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
        #[derive(Debug, sqlx::FromRow)]
        struct ShareStatsRow {
            total_shares: i64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct ScheduleStatsRow {
            total_schedules: i64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct AvgShareRow {
            avg_shares: f64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct ViewStatsRow {
            total_views: i64,
        }

        let dash_stats = sqlx::query_as::<_, StatsRow>(
            r#"SELECT COUNT(*) as total_dashboards,
             COUNT(*) FILTER (WHERE is_public) as public_dashboards
             FROM dashboards"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let report_stats = sqlx::query_as::<_, ReportStatsRow>(
            r#"SELECT COUNT(*) as total_reports,
             COUNT(*) FILTER (WHERE schedule IS NOT NULL) as scheduled_reports
             FROM reports"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let share_stats = sqlx::query_as::<_, ShareStatsRow>(
            r#"SELECT COUNT(*) as total_shares FROM dashboard_shares_v4"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let schedule_stats = sqlx::query_as::<_, ScheduleStatsRow>(
            r#"SELECT COUNT(*) as total_schedules FROM report_schedules_v5"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let avg_shares = sqlx::query_as::<_, AvgShareRow>(
            r#"SELECT COALESCE(COUNT(ds.id)::float / NULLIF(COUNT(DISTINCT d.id), 0), 0.0) as avg_shares
             FROM dashboards d LEFT JOIN dashboard_shares_v4 ds ON d.id = ds.dashboard_id"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let view_stats = sqlx::query_as::<_, ViewStatsRow>(
            r#"SELECT COALESCE(SUM(view_count), 0) as total_views FROM dashboard_analytics"#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DashboardStatsV7 {
            total_dashboards: dash_stats.total_dashboards,
            public_dashboards: dash_stats.public_dashboards,
            total_reports: report_stats.total_reports,
            scheduled_reports: report_stats.scheduled_reports,
            total_shares: share_stats.total_shares,
            total_schedules: schedule_stats.total_schedules,
            avg_shares_per_dashboard: avg_shares.avg_shares,
            total_views: view_stats.total_views,
        })
    }

    // V8: Dashboard sharing v5

    pub async fn create_dashboard_share_v5(
        &self,
        input: CreateDashboardShareV5,
    ) -> Result<DashboardShareV5, sqlx::Error> {
        let row = sqlx::query_as::<_, DashboardShareV5Row>(
            r#"INSERT INTO dashboard_shares_v5 (dashboard_id, user_id, permission)
             VALUES ($1, $2, $3)
             ON CONFLICT (dashboard_id, user_id) DO UPDATE SET permission = $3
             RETURNING id, dashboard_id, user_id, permission, created_at"#,
        )
        .bind(input.dashboard_id)
        .bind(input.user_id)
        .bind(input.permission.unwrap_or_else(|| "view".to_string()))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_dashboard_share_v5(
        &self,
        id: Uuid,
    ) -> Result<Option<DashboardShareV5>, sqlx::Error> {
        let row = sqlx::query_as::<_, DashboardShareV5Row>(
            r#"SELECT id, dashboard_id, user_id, permission, created_at
             FROM dashboard_shares_v5 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn list_dashboard_shares_v5(
        &self,
        dashboard_id: Uuid,
    ) -> Result<Vec<DashboardShareV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DashboardShareV5Row>(
            r#"SELECT id, dashboard_id, user_id, permission, created_at
             FROM dashboard_shares_v5 WHERE dashboard_id = $1
             ORDER BY created_at DESC"#,
        )
        .bind(dashboard_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_dashboard_share_v5(
        &self,
        id: Uuid,
        permission: &str,
    ) -> Result<DashboardShareV5, sqlx::Error> {
        let row = sqlx::query_as::<_, DashboardShareV5Row>(
            r#"UPDATE dashboard_shares_v5 SET permission = $2
             WHERE id = $1
             RETURNING id, dashboard_id, user_id, permission, created_at"#,
        )
        .bind(id)
        .bind(permission)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn delete_dashboard_share_v5(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM dashboard_shares_v5 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // V8: Report scheduling v6

    pub async fn create_report_schedule_v6(
        &self,
        input: CreateReportScheduleV6,
    ) -> Result<ReportScheduleV6, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV6Row>(
            r#"INSERT INTO report_schedules_v6 (report_id, cron_expression, enabled, next_run_at)
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

    pub async fn get_report_schedule_v6(
        &self,
        id: Uuid,
    ) -> Result<Option<ReportScheduleV6>, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV6Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v6 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn list_report_schedules_v6(
        &self,
    ) -> Result<Vec<ReportScheduleV6>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReportScheduleV6Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v6 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_report_schedule_v6(
        &self,
        id: Uuid,
        input: UpdateReportScheduleV6,
    ) -> Result<ReportScheduleV6, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV6Row>(
            r#"UPDATE report_schedules_v6 SET
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

    pub async fn delete_report_schedule_v6(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM report_schedules_v6 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_due_schedules_v6(
        &self,
    ) -> Result<Vec<ReportScheduleV6>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReportScheduleV6Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v6 WHERE enabled = true AND next_run_at <= NOW()
             ORDER BY next_run_at ASC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn mark_schedule_executed_v6(
        &self,
        id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE report_schedules_v6 SET last_run_at = NOW() WHERE id = $1"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // V8: Dashboard stats v8

    pub async fn get_dashboard_stats_v8(
        &self,
    ) -> Result<DashboardStatsV8, sqlx::Error> {
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
        #[derive(Debug, sqlx::FromRow)]
        struct ShareStatsRow {
            total_shares: i64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct ScheduleStatsRow {
            total_schedules: i64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct AvgShareRow {
            avg_shares: f64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct ViewStatsRow {
            total_views: i64,
        }

        let dash_stats = sqlx::query_as::<_, StatsRow>(
            r#"SELECT COUNT(*) as total_dashboards,
             COUNT(*) FILTER (WHERE is_public) as public_dashboards
             FROM dashboards"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let report_stats = sqlx::query_as::<_, ReportStatsRow>(
            r#"SELECT COUNT(*) as total_reports,
             COUNT(*) FILTER (WHERE schedule IS NOT NULL) as scheduled_reports
             FROM reports"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let share_stats = sqlx::query_as::<_, ShareStatsRow>(
            r#"SELECT COUNT(*) as total_shares FROM dashboard_shares_v5"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let schedule_stats = sqlx::query_as::<_, ScheduleStatsRow>(
            r#"SELECT COUNT(*) as total_schedules FROM report_schedules_v6"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let avg_shares = sqlx::query_as::<_, AvgShareRow>(
            r#"SELECT COALESCE(COUNT(ds.id)::float / NULLIF(COUNT(DISTINCT d.id), 0), 0.0) as avg_shares
             FROM dashboards d LEFT JOIN dashboard_shares_v5 ds ON d.id = ds.dashboard_id"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let view_stats = sqlx::query_as::<_, ViewStatsRow>(
            r#"SELECT COALESCE(SUM(view_count), 0) as total_views FROM dashboard_analytics"#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DashboardStatsV8 {
            total_dashboards: dash_stats.total_dashboards,
            public_dashboards: dash_stats.public_dashboards,
            total_reports: report_stats.total_reports,
            scheduled_reports: report_stats.scheduled_reports,
            total_shares: share_stats.total_shares,
            total_schedules: schedule_stats.total_schedules,
            avg_shares_per_dashboard: avg_shares.avg_shares,
            total_views: view_stats.total_views,
        })
    }

    // V9: Dashboard sharing v6

    pub async fn create_dashboard_share_v6(
        &self,
        input: CreateDashboardShareV6,
    ) -> Result<DashboardShareV6, sqlx::Error> {
        let row = sqlx::query_as::<_, DashboardShareV6Row>(
            r#"INSERT INTO dashboard_shares_v6 (dashboard_id, user_id, permission)
             VALUES ($1, $2, $3)
             ON CONFLICT (dashboard_id, user_id) DO UPDATE SET permission = $3
             RETURNING id, dashboard_id, user_id, permission, created_at"#,
        )
        .bind(input.dashboard_id)
        .bind(input.user_id)
        .bind(input.permission.unwrap_or_else(|| "view".to_string()))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_dashboard_share_v6(
        &self,
        id: Uuid,
    ) -> Result<Option<DashboardShareV6>, sqlx::Error> {
        let row = sqlx::query_as::<_, DashboardShareV6Row>(
            r#"SELECT id, dashboard_id, user_id, permission, created_at
             FROM dashboard_shares_v6 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn list_dashboard_shares_v6(
        &self,
        dashboard_id: Uuid,
    ) -> Result<Vec<DashboardShareV6>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DashboardShareV6Row>(
            r#"SELECT id, dashboard_id, user_id, permission, created_at
             FROM dashboard_shares_v6 WHERE dashboard_id = $1
             ORDER BY created_at DESC"#,
        )
        .bind(dashboard_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_dashboard_share_v6(
        &self,
        id: Uuid,
        permission: &str,
    ) -> Result<DashboardShareV6, sqlx::Error> {
        let row = sqlx::query_as::<_, DashboardShareV6Row>(
            r#"UPDATE dashboard_shares_v6 SET permission = $2
             WHERE id = $1
             RETURNING id, dashboard_id, user_id, permission, created_at"#,
        )
        .bind(id)
        .bind(permission)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn delete_dashboard_share_v6(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM dashboard_shares_v6 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_dashboards_shared_with_user_v6(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Dashboard>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DashboardRow>(
            r#"SELECT d.id, d.name, d.description, d.widgets, d.layout, d.is_public, d.created_at
             FROM dashboards d
             INNER JOIN dashboard_shares_v6 ds ON d.id = ds.dashboard_id
             WHERE ds.user_id = $1
             ORDER BY ds.created_at DESC"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    // V9: Report scheduling v7

    pub async fn create_report_schedule_v7(
        &self,
        input: CreateReportScheduleV7,
    ) -> Result<ReportScheduleV7, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV7Row>(
            r#"INSERT INTO report_schedules_v7 (report_id, cron_expression, enabled, next_run_at)
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

    pub async fn get_report_schedule_v7(
        &self,
        id: Uuid,
    ) -> Result<Option<ReportScheduleV7>, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV7Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v7 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn list_report_schedules_v7(
        &self,
    ) -> Result<Vec<ReportScheduleV7>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReportScheduleV7Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v7 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_report_schedule_v7(
        &self,
        id: Uuid,
        input: UpdateReportScheduleV7,
    ) -> Result<ReportScheduleV7, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV7Row>(
            r#"UPDATE report_schedules_v7 SET
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

    pub async fn delete_report_schedule_v7(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM report_schedules_v7 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_due_schedules_v7(
        &self,
    ) -> Result<Vec<ReportScheduleV7>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReportScheduleV7Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v7 WHERE enabled = true AND next_run_at <= NOW()
             ORDER BY next_run_at ASC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn mark_schedule_executed_v7(
        &self,
        id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE report_schedules_v7 SET last_run_at = NOW() WHERE id = $1"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // V9: Dashboard stats v9

    pub async fn get_dashboard_stats_v9(
        &self,
    ) -> Result<DashboardStatsV9, sqlx::Error> {
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
        #[derive(Debug, sqlx::FromRow)]
        struct ShareStatsRow {
            total_shares: i64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct ScheduleStatsRow {
            total_schedules: i64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct AvgShareRow {
            avg_shares: f64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct ViewStatsRow {
            total_views: i64,
        }

        let dash_stats = sqlx::query_as::<_, StatsRow>(
            r#"SELECT COUNT(*) as total_dashboards,
             COUNT(*) FILTER (WHERE is_public) as public_dashboards
             FROM dashboards"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let report_stats = sqlx::query_as::<_, ReportStatsRow>(
            r#"SELECT COUNT(*) as total_reports,
             COUNT(*) FILTER (WHERE schedule IS NOT NULL) as scheduled_reports
             FROM reports"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let share_stats = sqlx::query_as::<_, ShareStatsRow>(
            r#"SELECT COUNT(*) as total_shares FROM dashboard_shares_v6"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let schedule_stats = sqlx::query_as::<_, ScheduleStatsRow>(
            r#"SELECT COUNT(*) as total_schedules FROM report_schedules_v7"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let avg_shares = sqlx::query_as::<_, AvgShareRow>(
            r#"SELECT COALESCE(COUNT(ds.id)::float / NULLIF(COUNT(DISTINCT d.id), 0), 0.0) as avg_shares
             FROM dashboards d LEFT JOIN dashboard_shares_v6 ds ON d.id = ds.dashboard_id"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let view_stats = sqlx::query_as::<_, ViewStatsRow>(
            r#"SELECT COALESCE(SUM(view_count), 0) as total_views FROM dashboard_analytics"#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DashboardStatsV9 {
            total_dashboards: dash_stats.total_dashboards,
            public_dashboards: dash_stats.public_dashboards,
            total_reports: report_stats.total_reports,
            scheduled_reports: report_stats.scheduled_reports,
            total_shares: share_stats.total_shares,
            total_schedules: schedule_stats.total_schedules,
            avg_shares_per_dashboard: avg_shares.avg_shares,
            total_views: view_stats.total_views,
        })
    }

    // V10: Dashboard sharing v7

    pub async fn create_dashboard_share_v7(
        &self,
        input: CreateDashboardShareV7,
    ) -> Result<DashboardShareV7, sqlx::Error> {
        let row = sqlx::query_as::<_, DashboardShareV7Row>(
            r#"INSERT INTO dashboard_shares_v7 (dashboard_id, user_id, permission)
             VALUES ($1, $2, $3)
             ON CONFLICT (dashboard_id, user_id) DO UPDATE SET permission = $3
             RETURNING id, dashboard_id, user_id, permission, created_at"#,
        )
        .bind(input.dashboard_id)
        .bind(input.user_id)
        .bind(input.permission.as_deref().unwrap_or("view"))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_dashboard_shares_v7(
        &self,
        dashboard_id: Uuid,
    ) -> Result<Vec<DashboardShareV7>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DashboardShareV7Row>(
            r#"SELECT id, dashboard_id, user_id, permission, created_at
             FROM dashboard_shares_v7 WHERE dashboard_id = $1
             ORDER BY created_at DESC"#,
        )
        .bind(dashboard_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn delete_dashboard_share_v7(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM dashboard_shares_v7 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_dashboard_share_permission_v7(
        &self,
        id: Uuid,
        permission: &str,
    ) -> Result<DashboardShareV7, sqlx::Error> {
        let row = sqlx::query_as::<_, DashboardShareV7Row>(
            r#"UPDATE dashboard_shares_v7 SET permission = $2
             WHERE id = $1
             RETURNING id, dashboard_id, user_id, permission, created_at"#,
        )
        .bind(id)
        .bind(permission)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    // V10: Report scheduling v8

    pub async fn create_report_schedule_v8(
        &self,
        input: CreateReportScheduleV8,
    ) -> Result<ReportScheduleV8, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV8Row>(
            r#"INSERT INTO report_schedules_v8 (report_id, cron_expression, enabled, next_run_at)
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

    pub async fn get_report_schedule_v8(
        &self,
        id: Uuid,
    ) -> Result<Option<ReportScheduleV8>, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV8Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v8 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn list_report_schedules_v8(
        &self,
    ) -> Result<Vec<ReportScheduleV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReportScheduleV8Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v8 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_report_schedule_v8(
        &self,
        id: Uuid,
        input: UpdateReportScheduleV8,
    ) -> Result<ReportScheduleV8, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV8Row>(
            r#"UPDATE report_schedules_v8 SET
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

    pub async fn delete_report_schedule_v8(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM report_schedules_v8 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn mark_schedule_run_v8(
        &self,
        id: Uuid,
    ) -> Result<ReportScheduleV8, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV8Row>(
            r#"UPDATE report_schedules_v8 SET last_run_at = NOW()
             WHERE id = $1
             RETURNING id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    // V10: Dashboard stats v10

    pub async fn get_dashboard_stats_v10(
        &self,
    ) -> Result<DashboardStatsV10, sqlx::Error> {
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
        #[derive(Debug, sqlx::FromRow)]
        struct ShareStatsRow {
            total_shares: i64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct ScheduleStatsRow {
            total_schedules: i64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct AvgShareRow {
            avg_shares: f64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct ViewStatsRow {
            total_views: i64,
        }

        let dash_stats = sqlx::query_as::<_, StatsRow>(
            r#"SELECT COUNT(*) as total_dashboards,
             COUNT(*) FILTER (WHERE is_public) as public_dashboards
             FROM dashboards"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let report_stats = sqlx::query_as::<_, ReportStatsRow>(
            r#"SELECT COUNT(*) as total_reports,
             COUNT(*) FILTER (WHERE schedule IS NOT NULL) as scheduled_reports
             FROM reports"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let share_stats = sqlx::query_as::<_, ShareStatsRow>(
            r#"SELECT COUNT(*) as total_shares FROM dashboard_shares_v7"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let schedule_stats = sqlx::query_as::<_, ScheduleStatsRow>(
            r#"SELECT COUNT(*) as total_schedules FROM report_schedules_v8"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let avg_shares = sqlx::query_as::<_, AvgShareRow>(
            r#"SELECT COALESCE(COUNT(ds.id)::float / NULLIF(COUNT(DISTINCT d.id), 0), 0.0) as avg_shares
             FROM dashboards d LEFT JOIN dashboard_shares_v7 ds ON d.id = ds.dashboard_id"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let view_stats = sqlx::query_as::<_, ViewStatsRow>(
            r#"SELECT COALESCE(SUM(view_count), 0) as total_views FROM dashboard_analytics"#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DashboardStatsV10 {
            total_dashboards: dash_stats.total_dashboards,
            public_dashboards: dash_stats.public_dashboards,
            total_reports: report_stats.total_reports,
            scheduled_reports: report_stats.scheduled_reports,
            total_shares: share_stats.total_shares,
            total_schedules: schedule_stats.total_schedules,
            avg_shares_per_dashboard: avg_shares.avg_shares,
            total_views: view_stats.total_views,
        })
    }

    // V11: Dashboard sharing v8

    pub async fn create_dashboard_share_v8(
        &self,
        input: CreateDashboardShareV8,
    ) -> Result<DashboardShareV8, sqlx::Error> {
        let row = sqlx::query_as::<_, DashboardShareV8Row>(
            r#"INSERT INTO dashboard_shares_v8 (dashboard_id, user_id, permission)
             VALUES ($1, $2, $3)
             ON CONFLICT (dashboard_id, user_id) DO UPDATE SET permission = $3
             RETURNING id, dashboard_id, user_id, permission, created_at"#,
        )
        .bind(input.dashboard_id)
        .bind(input.user_id)
        .bind(input.permission.as_deref().unwrap_or("view"))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_dashboard_shares_v8(
        &self,
        dashboard_id: uuid::Uuid,
    ) -> Result<Vec<DashboardShareV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DashboardShareV8Row>(
            r#"SELECT id, dashboard_id, user_id, permission, created_at
             FROM dashboard_shares_v8 WHERE dashboard_id = $1
             ORDER BY created_at DESC"#,
        )
        .bind(dashboard_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn delete_dashboard_share_v8(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM dashboard_shares_v8 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_dashboard_share_permission_v8(
        &self,
        id: uuid::Uuid,
        permission: &str,
    ) -> Result<DashboardShareV8, sqlx::Error> {
        let row = sqlx::query_as::<_, DashboardShareV8Row>(
            r#"UPDATE dashboard_shares_v8 SET permission = $2
             WHERE id = $1
             RETURNING id, dashboard_id, user_id, permission, created_at"#,
        )
        .bind(id)
        .bind(permission)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_dashboards_shared_with_user_v8(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<Dashboard>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DashboardRow>(
            r#"SELECT d.id, d.name, d.description, d.widgets, d.layout, d.is_public, d.created_at
             FROM dashboards d
             INNER JOIN dashboard_shares_v8 ds ON d.id = ds.dashboard_id
             WHERE ds.user_id = $1
             ORDER BY ds.created_at DESC"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    // V11: Report scheduling v9

    pub async fn create_report_schedule_v9(
        &self,
        input: CreateReportScheduleV9,
    ) -> Result<ReportScheduleV9, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV9Row>(
            r#"INSERT INTO report_schedules_v9 (report_id, cron_expression, enabled, next_run_at)
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

    pub async fn get_report_schedule_v9(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<ReportScheduleV9>, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV9Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v9 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn list_report_schedules_v9(
        &self,
    ) -> Result<Vec<ReportScheduleV9>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReportScheduleV9Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v9 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_report_schedule_v9(
        &self,
        id: uuid::Uuid,
        input: UpdateReportScheduleV9,
    ) -> Result<ReportScheduleV9, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV9Row>(
            r#"UPDATE report_schedules_v9 SET
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

    pub async fn delete_report_schedule_v9(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM report_schedules_v9 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn mark_schedule_run_v9(
        &self,
        id: uuid::Uuid,
    ) -> Result<ReportScheduleV9, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV9Row>(
            r#"UPDATE report_schedules_v9 SET last_run_at = NOW()
             WHERE id = $1
             RETURNING id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_due_schedules_v9(
        &self,
    ) -> Result<Vec<ReportScheduleV9>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReportScheduleV9Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v9 WHERE enabled = true AND next_run_at <= NOW()
             ORDER BY next_run_at ASC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    // V11: Dashboard stats v11

    pub async fn get_dashboard_stats_v11(
        &self,
    ) -> Result<DashboardStatsV11, sqlx::Error> {
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
        #[derive(Debug, sqlx::FromRow)]
        struct ShareStatsRow {
            total_shares: i64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct ScheduleStatsRow {
            total_schedules: i64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct AvgShareRow {
            avg_shares: f64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct ViewStatsRow {
            total_views: i64,
        }

        let dash_stats = sqlx::query_as::<_, StatsRow>(
            r#"SELECT COUNT(*) as total_dashboards,
             COUNT(*) FILTER (WHERE is_public) as public_dashboards
             FROM dashboards"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let report_stats = sqlx::query_as::<_, ReportStatsRow>(
            r#"SELECT COUNT(*) as total_reports,
             COUNT(*) FILTER (WHERE schedule IS NOT NULL) as scheduled_reports
             FROM reports"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let share_stats = sqlx::query_as::<_, ShareStatsRow>(
            r#"SELECT COUNT(*) as total_shares FROM dashboard_shares_v8"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let schedule_stats = sqlx::query_as::<_, ScheduleStatsRow>(
            r#"SELECT COUNT(*) as total_schedules FROM report_schedules_v9"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let avg_shares = sqlx::query_as::<_, AvgShareRow>(
            r#"SELECT COALESCE(COUNT(ds.id)::float / NULLIF(COUNT(DISTINCT d.id), 0), 0.0) as avg_shares
             FROM dashboards d LEFT JOIN dashboard_shares_v8 ds ON d.id = ds.dashboard_id"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let view_stats = sqlx::query_as::<_, ViewStatsRow>(
            r#"SELECT COALESCE(SUM(view_count), 0) as total_views FROM dashboard_analytics"#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DashboardStatsV11 {
            total_dashboards: dash_stats.total_dashboards,
            public_dashboards: dash_stats.public_dashboards,
            total_reports: report_stats.total_reports,
            scheduled_reports: report_stats.scheduled_reports,
            total_shares: share_stats.total_shares,
            total_schedules: schedule_stats.total_schedules,
            avg_shares_per_dashboard: avg_shares.avg_shares,
            total_views: view_stats.total_views,
        })
    }

    // V12: Dashboard sharing v9

    pub async fn share_dashboard_v9(
        &self,
        dashboard_id: uuid::Uuid,
        user_id: uuid::Uuid,
        permission: &str,
    ) -> Result<DashboardShareV9, sqlx::Error> {
        let row = sqlx::query_as::<_, DashboardShareV9Row>(
            r#"INSERT INTO dashboard_shares_v9 (dashboard_id, user_id, permission)
             VALUES ($1, $2, $3)
             ON CONFLICT (dashboard_id, user_id) DO UPDATE SET permission = $3
             RETURNING id, dashboard_id, user_id, permission, created_at"#,
        )
        .bind(dashboard_id)
        .bind(user_id)
        .bind(permission)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_dashboard_shares_v9(
        &self,
        dashboard_id: uuid::Uuid,
    ) -> Result<Vec<DashboardShareV9>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DashboardShareV9Row>(
            r#"SELECT id, dashboard_id, user_id, permission, created_at
             FROM dashboard_shares_v9 WHERE dashboard_id = $1
             ORDER BY created_at DESC"#,
        )
        .bind(dashboard_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn remove_dashboard_share_v9(
        &self,
        dashboard_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM dashboard_shares_v9 WHERE dashboard_id = $1 AND user_id = $2",
        )
        .bind(dashboard_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    // V12: Report scheduling v10

    pub async fn create_report_schedule_v10(
        &self,
        input: CreateReportScheduleV10,
    ) -> Result<ReportScheduleV10, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV10Row>(
            r#"INSERT INTO report_schedules_v10 (report_id, cron_expression, enabled, next_run_at)
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

    pub async fn get_report_schedule_v10(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<ReportScheduleV10>, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV10Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v10 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn list_report_schedules_v10(
        &self,
    ) -> Result<Vec<ReportScheduleV10>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReportScheduleV10Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v10 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_due_schedules_v10(
        &self,
    ) -> Result<Vec<ReportScheduleV10>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReportScheduleV10Row>(
            r#"SELECT id, report_id, cron_expression, enabled, last_run_at, next_run_at, created_at
             FROM report_schedules_v10 WHERE enabled = true AND next_run_at <= NOW()
             ORDER BY next_run_at ASC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn mark_schedule_executed_v10(
        &self,
        id: uuid::Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE report_schedules_v10 SET last_run_at = NOW() WHERE id = $1"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_report_schedule_v10(
        &self,
        id: uuid::Uuid,
        input: UpdateReportScheduleV10,
    ) -> Result<ReportScheduleV10, sqlx::Error> {
        let row = sqlx::query_as::<_, ReportScheduleV10Row>(
            r#"UPDATE report_schedules_v10 SET
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

    pub async fn delete_report_schedule_v10(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM report_schedules_v10 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // V12: Dashboard stats v12

    pub async fn get_dashboard_stats_v12(
        &self,
    ) -> Result<DashboardStatsV12, sqlx::Error> {
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
        #[derive(Debug, sqlx::FromRow)]
        struct ShareStatsRow {
            total_shares: i64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct ScheduleStatsRow {
            total_schedules: i64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct AvgShareRow {
            avg_shares: f64,
        }
        #[derive(Debug, sqlx::FromRow)]
        struct ViewStatsRow {
            total_views: i64,
        }

        let dash_stats = sqlx::query_as::<_, StatsRow>(
            r#"SELECT COUNT(*) as total_dashboards,
             COUNT(*) FILTER (WHERE is_public) as public_dashboards
             FROM dashboards"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let report_stats = sqlx::query_as::<_, ReportStatsRow>(
            r#"SELECT COUNT(*) as total_reports,
             COUNT(*) FILTER (WHERE schedule IS NOT NULL) as scheduled_reports
             FROM reports"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let share_stats = sqlx::query_as::<_, ShareStatsRow>(
            r#"SELECT COUNT(*) as total_shares FROM dashboard_shares_v9"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let schedule_stats = sqlx::query_as::<_, ScheduleStatsRow>(
            r#"SELECT COUNT(*) as total_schedules FROM report_schedules_v10"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let avg_shares = sqlx::query_as::<_, AvgShareRow>(
            r#"SELECT COALESCE(COUNT(ds.id)::float / NULLIF(COUNT(DISTINCT d.id), 0), 0.0) as avg_shares
             FROM dashboards d LEFT JOIN dashboard_shares_v9 ds ON d.id = ds.dashboard_id"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let view_stats = sqlx::query_as::<_, ViewStatsRow>(
            r#"SELECT COALESCE(SUM(view_count), 0) as total_views FROM dashboard_analytics"#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DashboardStatsV12 {
            total_dashboards: dash_stats.total_dashboards,
            public_dashboards: dash_stats.public_dashboards,
            total_reports: report_stats.total_reports,
            scheduled_reports: report_stats.scheduled_reports,
            total_shares: share_stats.total_shares,
            total_schedules: schedule_stats.total_schedules,
            avg_shares_per_dashboard: avg_shares.avg_shares,
            total_views: view_stats.total_views,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
struct DashboardTemplateRow {
    id: Uuid,
    name: String,
    description: String,
    template_type: String,
    config: serde_json::Value,
    is_public: bool,
    author_id: Option<Uuid>,
    usage_count: i64,
    created_at: DateTime<Utc>,
}

impl From<DashboardTemplateRow> for DashboardTemplate {
    fn from(row: DashboardTemplateRow) -> Self {
        DashboardTemplate {
            id: row.id,
            name: row.name,
            description: row.description,
            template_type: row.template_type,
            config: row.config,
            is_public: row.is_public,
            author_id: row.author_id,
            usage_count: row.usage_count,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ReportTemplateRow {
    id: Uuid,
    name: String,
    description: String,
    report_type: String,
    config: serde_json::Value,
    is_public: bool,
    author_id: Option<Uuid>,
    usage_count: i64,
    created_at: DateTime<Utc>,
}

impl From<ReportTemplateRow> for ReportTemplate {
    fn from(row: ReportTemplateRow) -> Self {
        ReportTemplate {
            id: row.id,
            name: row.name,
            description: row.description,
            report_type: row.report_type,
            config: row.config,
            is_public: row.is_public,
            author_id: row.author_id,
            usage_count: row.usage_count,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct DashboardShareV2Row {
    id: Uuid,
    dashboard_id: Uuid,
    user_id: Uuid,
    permission: String,
    created_at: DateTime<Utc>,
}

impl From<DashboardShareV2Row> for DashboardShareV2 {
    fn from(row: DashboardShareV2Row) -> Self {
        DashboardShareV2 {
            id: row.id,
            dashboard_id: row.dashboard_id,
            user_id: row.user_id,
            permission: row.permission,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ReportScheduleV3Row {
    id: Uuid,
    report_id: Uuid,
    cron_expression: String,
    enabled: bool,
    last_run_at: Option<DateTime<Utc>>,
    next_run_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl From<ReportScheduleV3Row> for ReportScheduleV3 {
    fn from(row: ReportScheduleV3Row) -> Self {
        ReportScheduleV3 {
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

#[derive(Debug, sqlx::FromRow)]
struct DashboardShareV3Row {
    id: Uuid,
    dashboard_id: Uuid,
    user_id: Uuid,
    permission: String,
    created_at: DateTime<Utc>,
}

impl From<DashboardShareV3Row> for DashboardShareV3 {
    fn from(row: DashboardShareV3Row) -> Self {
        DashboardShareV3 {
            id: row.id,
            dashboard_id: row.dashboard_id,
            user_id: row.user_id,
            permission: row.permission,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ReportScheduleV4Row {
    id: Uuid,
    report_id: Uuid,
    cron_expression: String,
    enabled: bool,
    last_run_at: Option<DateTime<Utc>>,
    next_run_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl From<ReportScheduleV4Row> for ReportScheduleV4 {
    fn from(row: ReportScheduleV4Row) -> Self {
        ReportScheduleV4 {
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

#[derive(Debug, sqlx::FromRow)]
struct DashboardShareV4Row {
    id: Uuid,
    dashboard_id: Uuid,
    user_id: Uuid,
    permission: String,
    created_at: DateTime<Utc>,
}

impl From<DashboardShareV4Row> for DashboardShareV4 {
    fn from(row: DashboardShareV4Row) -> Self {
        DashboardShareV4 {
            id: row.id,
            dashboard_id: row.dashboard_id,
            user_id: row.user_id,
            permission: row.permission,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ReportScheduleV5Row {
    id: Uuid,
    report_id: Uuid,
    cron_expression: String,
    enabled: bool,
    last_run_at: Option<DateTime<Utc>>,
    next_run_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl From<ReportScheduleV5Row> for ReportScheduleV5 {
    fn from(row: ReportScheduleV5Row) -> Self {
        ReportScheduleV5 {
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

#[derive(Debug, sqlx::FromRow)]
struct DashboardShareV5Row {
    id: Uuid,
    dashboard_id: Uuid,
    user_id: Uuid,
    permission: String,
    created_at: DateTime<Utc>,
}

impl From<DashboardShareV5Row> for DashboardShareV5 {
    fn from(row: DashboardShareV5Row) -> Self {
        DashboardShareV5 {
            id: row.id,
            dashboard_id: row.dashboard_id,
            user_id: row.user_id,
            permission: row.permission,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ReportScheduleV6Row {
    id: Uuid,
    report_id: Uuid,
    cron_expression: String,
    enabled: bool,
    last_run_at: Option<DateTime<Utc>>,
    next_run_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl From<ReportScheduleV6Row> for ReportScheduleV6 {
    fn from(row: ReportScheduleV6Row) -> Self {
        ReportScheduleV6 {
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

#[derive(Debug, sqlx::FromRow)]
struct DashboardShareV6Row {
    id: Uuid,
    dashboard_id: Uuid,
    user_id: Uuid,
    permission: String,
    created_at: DateTime<Utc>,
}

impl From<DashboardShareV6Row> for DashboardShareV6 {
    fn from(row: DashboardShareV6Row) -> Self {
        DashboardShareV6 {
            id: row.id,
            dashboard_id: row.dashboard_id,
            user_id: row.user_id,
            permission: row.permission,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ReportScheduleV7Row {
    id: Uuid,
    report_id: Uuid,
    cron_expression: String,
    enabled: bool,
    last_run_at: Option<DateTime<Utc>>,
    next_run_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl From<ReportScheduleV7Row> for ReportScheduleV7 {
    fn from(row: ReportScheduleV7Row) -> Self {
        ReportScheduleV7 {
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

#[derive(Debug, sqlx::FromRow)]
struct DashboardShareV7Row {
    id: Uuid,
    dashboard_id: Uuid,
    user_id: Uuid,
    permission: String,
    created_at: DateTime<Utc>,
}

impl From<DashboardShareV7Row> for DashboardShareV7 {
    fn from(row: DashboardShareV7Row) -> Self {
        DashboardShareV7 {
            id: row.id,
            dashboard_id: row.dashboard_id,
            user_id: row.user_id,
            permission: row.permission,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ReportScheduleV8Row {
    id: Uuid,
    report_id: Uuid,
    cron_expression: String,
    enabled: bool,
    last_run_at: Option<DateTime<Utc>>,
    next_run_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl From<ReportScheduleV8Row> for ReportScheduleV8 {
    fn from(row: ReportScheduleV8Row) -> Self {
        ReportScheduleV8 {
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

#[derive(Debug, sqlx::FromRow)]
struct DashboardShareV8Row {
    id: uuid::Uuid,
    dashboard_id: uuid::Uuid,
    user_id: uuid::Uuid,
    permission: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<DashboardShareV8Row> for DashboardShareV8 {
    fn from(row: DashboardShareV8Row) -> Self {
        DashboardShareV8 {
            id: row.id,
            dashboard_id: row.dashboard_id,
            user_id: row.user_id,
            permission: row.permission,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ReportScheduleV9Row {
    id: uuid::Uuid,
    report_id: uuid::Uuid,
    cron_expression: String,
    enabled: bool,
    last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    next_run_at: chrono::DateTime<chrono::Utc>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<ReportScheduleV9Row> for ReportScheduleV9 {
    fn from(row: ReportScheduleV9Row) -> Self {
        ReportScheduleV9 {
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

#[derive(Debug, sqlx::FromRow)]
struct DashboardShareV9Row {
    id: uuid::Uuid,
    dashboard_id: uuid::Uuid,
    user_id: uuid::Uuid,
    permission: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<DashboardShareV9Row> for DashboardShareV9 {
    fn from(row: DashboardShareV9Row) -> Self {
        DashboardShareV9 {
            id: row.id,
            dashboard_id: row.dashboard_id,
            user_id: row.user_id,
            permission: row.permission,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ReportScheduleV10Row {
    id: uuid::Uuid,
    report_id: uuid::Uuid,
    cron_expression: String,
    enabled: bool,
    last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    next_run_at: chrono::DateTime<chrono::Utc>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<ReportScheduleV10Row> for ReportScheduleV10 {
    fn from(row: ReportScheduleV10Row) -> Self {
        ReportScheduleV10 {
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
