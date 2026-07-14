use super::types::*;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct TestCoverageStore {
    pool: PgPool,
}

impl TestCoverageStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upload_coverage(
        &self,
        repo_id: Uuid,
        req: CoverageUploadRequest,
    ) -> Result<CoverageReport, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO test_coverage (repo_id, file_path, line_coverage, branch_coverage, function_coverage, total_lines, covered_lines, measured_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        )
        .bind(repo_id)
        .bind(&req.file_path)
        .bind(req.line_coverage)
        .bind(req.branch_coverage)
        .bind(req.function_coverage)
        .bind(req.total_lines)
        .bind(req.covered_lines)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(CoverageReport {
            repo_id,
            file_path: req.file_path,
            line_coverage: req.line_coverage,
            branch_coverage: req.branch_coverage,
            function_coverage: req.function_coverage,
            total_lines: req.total_lines,
            covered_lines: req.covered_lines,
        })
    }

    pub async fn upload_coverage_v2(
        &self,
        repo_id: Uuid,
        req: CoverageUploadRequestV2,
    ) -> Result<CoverageReportV2, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO test_coverage_v2 (repo_id, file_path, line_coverage, branch_coverage, function_coverage, total_lines, covered_lines, uncovered_lines, measured_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
        )
        .bind(repo_id)
        .bind(&req.file_path)
        .bind(req.line_coverage)
        .bind(req.branch_coverage)
        .bind(req.function_coverage)
        .bind(req.total_lines)
        .bind(req.covered_lines)
        .bind(&req.uncovered_lines)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(CoverageReportV2 {
            repo_id,
            file_path: req.file_path,
            line_coverage: req.line_coverage,
            branch_coverage: req.branch_coverage,
            function_coverage: req.function_coverage,
            total_lines: req.total_lines,
            covered_lines: req.covered_lines,
            uncovered_lines: req.uncovered_lines,
        })
    }

    pub async fn get_uncovered_lines(
        &self,
        repo_id: Uuid,
        file_path: &str,
    ) -> Result<Vec<i32>, sqlx::Error> {
        let row = sqlx::query_scalar::<_, Vec<i32>>(
            r#"SELECT uncovered_lines FROM test_coverage_v2
               WHERE repo_id = $1 AND file_path = $2
               ORDER BY measured_at DESC
               LIMIT 1"#,
        )
        .bind(repo_id)
        .bind(file_path)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.unwrap_or_default())
    }

    pub async fn get_trends_v2(
        &self,
        repo_id: Uuid,
        days: i64,
    ) -> Result<Vec<CoverageTrendV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, CoverageTrendV2Row>(
            r#"SELECT
                DATE(measured_at) as date,
                AVG(line_coverage) as avg_line_coverage,
                AVG(branch_coverage) as avg_branch_coverage,
                AVG(function_coverage) as avg_function_coverage,
                COUNT(DISTINCT file_path) as file_count,
                COALESCE(SUM(array_length(uncovered_lines, 1)), 0) as total_uncovered_lines
             FROM test_coverage_v2
             WHERE repo_id = $1
               AND measured_at >= NOW() - ($2 || ' days')::INTERVAL
             GROUP BY DATE(measured_at)
             ORDER BY DATE(measured_at) DESC"#,
        )
        .bind(repo_id)
        .bind(days)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(CoverageTrendV2::from).collect())
    }

    pub async fn check_enforcement_v2(
        &self,
        repo_id: Uuid,
        config: CoverageEnforcementConfig,
    ) -> Result<CoverageEnforcementResult, sqlx::Error> {
        let row = sqlx::query_as::<_, EnforcementRow>(
            r#"SELECT
                bool_and(
                    line_coverage >= $2
                    AND branch_coverage >= $3
                    AND function_coverage >= $4
                ) as passes,
                COUNT(*) as files_checked,
                COUNT(*) FILTER (WHERE line_coverage >= $2 AND branch_coverage >= $3 AND function_coverage >= $4) as files_passing,
                COUNT(*) FILTER (WHERE line_coverage < $2 OR branch_coverage < $3 OR function_coverage < $4) as files_failing
             FROM test_coverage_v2
             WHERE repo_id = $1
               AND measured_at = (SELECT MAX(measured_at) FROM test_coverage_v2 WHERE repo_id = $1)"#,
        )
        .bind(repo_id)
        .bind(config.min_line_coverage)
        .bind(config.min_branch_coverage)
        .bind(config.min_function_coverage)
        .fetch_one(&self.pool)
        .await?;

        Ok(CoverageEnforcementResult {
            passes: row.passes,
            files_checked: row.files_checked,
            files_passing: row.files_passing,
            files_failing: row.files_failing,
        })
    }

    pub async fn get_summary(&self, repo_id: Uuid) -> Result<CoverageSummary, sqlx::Error> {
        let row = sqlx::query_as::<_, CoverageSummaryRow>(
            r#"SELECT
                $1 as repo_id,
                COALESCE(AVG(line_coverage), 0) as avg_line_coverage,
                COALESCE(AVG(branch_coverage), 0) as avg_branch_coverage,
                COALESCE(AVG(function_coverage), 0) as avg_function_coverage,
                COUNT(DISTINCT file_path) as total_files,
                COALESCE(SUM(total_lines), 0) as total_lines,
                COALESCE(SUM(covered_lines), 0) as total_covered_lines,
                CASE WHEN SUM(total_lines) > 0 THEN SUM(covered_lines)::float / SUM(total_lines)::float * 100 ELSE 0 END as overall_coverage
             FROM test_coverage
             WHERE repo_id = $1
               AND measured_at = (SELECT MAX(measured_at) FROM test_coverage WHERE repo_id = $1)"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(CoverageSummary {
            repo_id: row.repo_id,
            avg_line_coverage: row.avg_line_coverage,
            avg_branch_coverage: row.avg_branch_coverage,
            avg_function_coverage: row.avg_function_coverage,
            total_files: row.total_files,
            total_lines: row.total_lines,
            total_covered_lines: row.total_covered_lines,
            overall_coverage: row.overall_coverage,
        })
    }

    pub async fn get_trends(
        &self,
        repo_id: Uuid,
        days: i64,
    ) -> Result<Vec<CoverageTrend>, sqlx::Error> {
        let rows = sqlx::query_as::<_, CoverageTrendRow>(
            r#"SELECT
                DATE(measured_at) as date,
                AVG(line_coverage) as avg_line_coverage,
                AVG(branch_coverage) as avg_branch_coverage,
                AVG(function_coverage) as avg_function_coverage,
                COUNT(DISTINCT file_path) as file_count
             FROM test_coverage
             WHERE repo_id = $1
               AND measured_at >= NOW() - ($2 || ' days')::INTERVAL
             GROUP BY DATE(measured_at)
             ORDER BY DATE(measured_at) DESC"#,
        )
        .bind(repo_id)
        .bind(days)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(CoverageTrend::from).collect())
    }

    pub async fn check_enforcement(
        &self,
        repo_id: Uuid,
        config: CoverageEnforcementConfig,
    ) -> Result<CoverageEnforcementResult, sqlx::Error> {
        let row = sqlx::query_as::<_, EnforcementRow>(
            r#"SELECT
                bool_and(
                    line_coverage >= $2
                    AND branch_coverage >= $3
                    AND function_coverage >= $4
                ) as passes,
                COUNT(*) as files_checked,
                COUNT(*) FILTER (WHERE line_coverage >= $2 AND branch_coverage >= $3 AND function_coverage >= $4) as files_passing,
                COUNT(*) FILTER (WHERE line_coverage < $2 OR branch_coverage < $3 OR function_coverage < $4) as files_failing
             FROM test_coverage
             WHERE repo_id = $1
               AND measured_at = (SELECT MAX(measured_at) FROM test_coverage WHERE repo_id = $1)"#,
        )
        .bind(repo_id)
        .bind(config.min_line_coverage)
        .bind(config.min_branch_coverage)
        .bind(config.min_function_coverage)
        .fetch_one(&self.pool)
        .await?;

        Ok(CoverageEnforcementResult {
            passes: row.passes,
            files_checked: row.files_checked,
            files_passing: row.files_passing,
            files_failing: row.files_failing,
        })
    }
}

#[derive(sqlx::FromRow)]
struct CoverageSummaryRow {
    repo_id: Uuid,
    avg_line_coverage: f64,
    avg_branch_coverage: f64,
    avg_function_coverage: f64,
    total_files: i64,
    total_lines: i64,
    total_covered_lines: i64,
    overall_coverage: f64,
}

#[derive(sqlx::FromRow)]
struct CoverageTrendRow {
    date: chrono::NaiveDate,
    avg_line_coverage: f64,
    avg_branch_coverage: f64,
    avg_function_coverage: f64,
    file_count: i64,
}

impl From<CoverageTrendRow> for CoverageTrend {
    fn from(row: CoverageTrendRow) -> Self {
        Self {
            date: row.date,
            avg_line_coverage: row.avg_line_coverage,
            avg_branch_coverage: row.avg_branch_coverage,
            avg_function_coverage: row.avg_function_coverage,
            file_count: row.file_count,
        }
    }
}

#[derive(sqlx::FromRow)]
struct CoverageTrendV2Row {
    date: chrono::NaiveDate,
    avg_line_coverage: f64,
    avg_branch_coverage: f64,
    avg_function_coverage: f64,
    file_count: i64,
    total_uncovered_lines: i64,
}

impl From<CoverageTrendV2Row> for CoverageTrendV2 {
    fn from(row: CoverageTrendV2Row) -> Self {
        Self {
            date: row.date,
            avg_line_coverage: row.avg_line_coverage,
            avg_branch_coverage: row.avg_branch_coverage,
            avg_function_coverage: row.avg_function_coverage,
            file_count: row.file_count,
            total_uncovered_lines: row.total_uncovered_lines,
        }
    }
}

#[derive(sqlx::FromRow)]
struct EnforcementRow {
    passes: bool,
    files_checked: i64,
    files_passing: i64,
    files_failing: i64,
}
