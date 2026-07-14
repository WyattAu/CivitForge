use super::types::*;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct CodeQualityStore {
    pool: PgPool,
}

impl CodeQualityStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record_metric(
        &self,
        repo_id: Uuid,
        req: RecordMetricRequest,
    ) -> Result<QualityMetricReport, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO code_quality_metrics (repo_id, metric_name, metric_value, file_path, measured_at)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(repo_id)
        .bind(&req.metric_name)
        .bind(req.metric_value)
        .bind(&req.file_path)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(QualityMetricReport {
            repo_id,
            metric_name: req.metric_name,
            metric_value: req.metric_value,
            file_path: req.file_path,
        })
    }

    pub async fn get_metrics(
        &self,
        repo_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<QualityMetricReport>, sqlx::Error> {
        let rows = if let Some(name) = metric_name {
            sqlx::query_as::<_, MetricRow>(
                r#"SELECT repo_id, metric_name, metric_value, file_path
                   FROM code_quality_metrics
                   WHERE repo_id = $1 AND metric_name = $2
                   ORDER BY measured_at DESC
                   LIMIT $3 OFFSET $4"#,
            )
            .bind(repo_id)
            .bind(name)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, MetricRow>(
                r#"SELECT repo_id, metric_name, metric_value, file_path
                   FROM code_quality_metrics
                   WHERE repo_id = $1
                   ORDER BY measured_at DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(repo_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows.into_iter().map(QualityMetricReport::from).collect())
    }

    pub async fn get_summary(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<QualityMetricSummary>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SummaryRow>(
            r#"SELECT
                metric_name,
                (SELECT metric_value FROM code_quality_metrics WHERE repo_id = $1 AND metric_name = cqm.metric_name ORDER BY measured_at DESC LIMIT 1) as latest_value,
                AVG(metric_value) as avg_value,
                MIN(metric_value) as min_value,
                MAX(metric_value) as max_value,
                COUNT(*) as measurement_count,
                COUNT(DISTINCT file_path) as files_affected
             FROM code_quality_metrics cqm
             WHERE repo_id = $1
             GROUP BY metric_name
             ORDER BY metric_name"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(QualityMetricSummary::from).collect())
    }

    pub async fn get_trends(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        days: i64,
    ) -> Result<Vec<QualityTrend>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TrendRow>(
            r#"SELECT
                DATE(measured_at) as date,
                AVG(metric_value) as avg_value,
                MIN(metric_value) as min_value,
                MAX(metric_value) as max_value,
                COUNT(*) as measurement_count
             FROM code_quality_metrics
             WHERE repo_id = $1 AND metric_name = $2
               AND measured_at >= NOW() - ($3 || ' days')::INTERVAL
             GROUP BY DATE(measured_at)
             ORDER BY DATE(measured_at) DESC"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .bind(days)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(QualityTrend::from).collect())
    }

    pub async fn get_complexity_analysis(
        &self,
        repo_id: Uuid,
    ) -> Result<ComplexityAnalysis, sqlx::Error> {
        let row = sqlx::query_as::<_, ComplexityRow>(
            r#"SELECT
                COALESCE(AVG(metric_value) FILTER (WHERE metric_name = 'cyclomatic_complexity'), 0) as avg_complexity,
                COALESCE(MAX(metric_value) FILTER (WHERE metric_name = 'cyclomatic_complexity'), 0) as max_complexity,
                COALESCE(AVG(metric_value) FILTER (WHERE metric_name = 'cognitive_complexity'), 0) as avg_cognitive_complexity,
                COUNT(DISTINCT file_path) FILTER (WHERE metric_name = 'cyclomatic_complexity' AND metric_value > 15) as high_complexity_files,
                COUNT(*) as total_measurements
             FROM code_quality_metrics
             WHERE repo_id = $1
               AND metric_name IN ('cyclomatic_complexity', 'cognitive_complexity')
               AND measured_at = (SELECT MAX(measured_at) FROM code_quality_metrics WHERE repo_id = $1 AND metric_name IN ('cyclomatic_complexity', 'cognitive_complexity'))"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ComplexityAnalysis {
            avg_complexity: row.avg_complexity,
            max_complexity: row.max_complexity,
            avg_cognitive_complexity: row.avg_cognitive_complexity,
            high_complexity_files: row.high_complexity_files,
            total_measurements: row.total_measurements,
        })
    }

    pub async fn get_duplication_report(
        &self,
        repo_id: Uuid,
    ) -> Result<DuplicationReport, sqlx::Error> {
        let row = sqlx::query_as::<_, DuplicationRow>(
            r#"SELECT
                COALESCE(AVG(metric_value) FILTER (WHERE metric_name = 'duplication_ratio'), 0) as duplication_ratio,
                COALESCE(SUM(metric_value) FILTER (WHERE metric_name = 'duplicated_lines'), 0) as total_duplicated_lines,
                COUNT(DISTINCT file_path) FILTER (WHERE metric_name = 'duplication_ratio' AND metric_value > 0) as files_with_duplication
             FROM code_quality_metrics
             WHERE repo_id = $1
               AND metric_name IN ('duplication_ratio', 'duplicated_lines')
               AND measured_at = (SELECT MAX(measured_at) FROM code_quality_metrics WHERE repo_id = $1 AND metric_name IN ('duplication_ratio', 'duplicated_lines'))"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(DuplicationReport {
            duplication_ratio: row.duplication_ratio,
            total_duplicated_lines: row.total_duplicated_lines,
            files_with_duplication: row.files_with_duplication,
        })
    }

    pub async fn get_code_smells_report(
        &self,
        repo_id: Uuid,
    ) -> Result<CodeSmellsReport, sqlx::Error> {
        let row = sqlx::query_as::<_, CodeSmellsRow>(
            r#"SELECT
                COALESCE(SUM(metric_value) FILTER (WHERE metric_name = 'code_smells'), 0) as total_smells,
                COALESCE(AVG(metric_value) FILTER (WHERE metric_name = 'smell_density'), 0) as smell_density,
                COUNT(DISTINCT file_path) FILTER (WHERE metric_name = 'code_smells' AND metric_value > 0) as files_with_smells,
                COALESCE(SUM(metric_value) FILTER (WHERE metric_name = 'critical_smells'), 0) as critical_smells,
                COALESCE(SUM(metric_value) FILTER (WHERE metric_name = 'major_smells'), 0) as major_smells,
                COALESCE(SUM(metric_value) FILTER (WHERE metric_name = 'minor_smells'), 0) as minor_smells
             FROM code_quality_metrics
             WHERE repo_id = $1
               AND metric_name IN ('code_smells', 'smell_density', 'critical_smells', 'major_smells', 'minor_smells')
               AND measured_at = (SELECT MAX(measured_at) FROM code_quality_metrics WHERE repo_id = $1 AND metric_name IN ('code_smells', 'smell_density', 'critical_smells', 'major_smells', 'minor_smells'))"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(CodeSmellsReport {
            total_smells: row.total_smells,
            smell_density: row.smell_density,
            files_with_smells: row.files_with_smells,
            critical_smells: row.critical_smells,
            major_smells: row.major_smells,
            minor_smells: row.minor_smells,
        })
    }

    pub async fn get_technical_debt_report(
        &self,
        repo_id: Uuid,
    ) -> Result<TechnicalDebtReport, sqlx::Error> {
        let row = sqlx::query_as::<_, TechDebtRow>(
            r#"SELECT
                COALESCE(SUM(metric_value) FILTER (WHERE metric_name = 'technical_debt_hours'), 0) as total_debt_hours,
                COALESCE(AVG(metric_value) FILTER (WHERE metric_name = 'debt_ratio'), 0) as debt_ratio,
                COALESCE(AVG(metric_value) FILTER (WHERE metric_name = 'debt_per_file'), 0) as debt_per_file,
                COALESCE(AVG(metric_value) FILTER (WHERE metric_name = 'remediation_time_priority'), 0) as remediation_time_priority,
                COUNT(DISTINCT file_path) FILTER (WHERE metric_name = 'technical_debt_hours' AND metric_value > 0) as files_with_debt
             FROM code_quality_metrics
             WHERE repo_id = $1
               AND metric_name IN ('technical_debt_hours', 'debt_ratio', 'debt_per_file', 'remediation_time_priority')
               AND measured_at = (SELECT MAX(measured_at) FROM code_quality_metrics WHERE repo_id = $1 AND metric_name IN ('technical_debt_hours', 'debt_ratio', 'debt_per_file', 'remediation_time_priority'))"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TechnicalDebtReport {
            total_debt_hours: row.total_debt_hours,
            debt_ratio: row.debt_ratio,
            debt_per_file: row.debt_per_file,
            remediation_time_priority: row.remediation_time_priority,
            files_with_debt: row.files_with_debt,
        })
    }
}

#[derive(sqlx::FromRow)]
struct MetricRow {
    repo_id: Uuid,
    metric_name: String,
    metric_value: f64,
    file_path: Option<String>,
}

impl From<MetricRow> for QualityMetricReport {
    fn from(row: MetricRow) -> Self {
        Self {
            repo_id: row.repo_id,
            metric_name: row.metric_name,
            metric_value: row.metric_value,
            file_path: row.file_path,
        }
    }
}

#[derive(sqlx::FromRow)]
struct SummaryRow {
    metric_name: String,
    latest_value: f64,
    avg_value: f64,
    min_value: f64,
    max_value: f64,
    measurement_count: i64,
    files_affected: i64,
}

impl From<SummaryRow> for QualityMetricSummary {
    fn from(row: SummaryRow) -> Self {
        Self {
            metric_name: row.metric_name,
            latest_value: row.latest_value,
            avg_value: row.avg_value,
            min_value: row.min_value,
            max_value: row.max_value,
            measurement_count: row.measurement_count,
            files_affected: row.files_affected,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TrendRow {
    date: chrono::NaiveDate,
    avg_value: f64,
    min_value: f64,
    max_value: f64,
    measurement_count: i64,
}

impl From<TrendRow> for QualityTrend {
    fn from(row: TrendRow) -> Self {
        Self {
            date: row.date,
            avg_value: row.avg_value,
            min_value: row.min_value,
            max_value: row.max_value,
            measurement_count: row.measurement_count,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ComplexityRow {
    avg_complexity: f64,
    max_complexity: f64,
    avg_cognitive_complexity: f64,
    high_complexity_files: i64,
    total_measurements: i64,
}

#[derive(sqlx::FromRow)]
struct DuplicationRow {
    duplication_ratio: f64,
    total_duplicated_lines: f64,
    files_with_duplication: i64,
}

#[derive(sqlx::FromRow)]
struct CodeSmellsRow {
    total_smells: f64,
    smell_density: f64,
    files_with_smells: i64,
    critical_smells: f64,
    major_smells: f64,
    minor_smells: f64,
}

#[derive(sqlx::FromRow)]
struct TechDebtRow {
    total_debt_hours: f64,
    debt_ratio: f64,
    debt_per_file: f64,
    remediation_time_priority: f64,
    files_with_debt: i64,
}
