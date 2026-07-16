use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub repo_id: Uuid,
    pub file_path: String,
    pub line_coverage: f64,
    pub branch_coverage: f64,
    pub function_coverage: f64,
    pub total_lines: i32,
    pub covered_lines: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReportV2 {
    pub repo_id: Uuid,
    pub file_path: String,
    pub line_coverage: f64,
    pub branch_coverage: f64,
    pub function_coverage: f64,
    pub total_lines: i32,
    pub covered_lines: i32,
    pub uncovered_lines: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageSummary {
    pub repo_id: Uuid,
    pub avg_line_coverage: f64,
    pub avg_branch_coverage: f64,
    pub avg_function_coverage: f64,
    pub total_files: i64,
    pub total_lines: i64,
    pub total_covered_lines: i64,
    pub overall_coverage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageTrend {
    pub date: chrono::NaiveDate,
    pub avg_line_coverage: f64,
    pub avg_branch_coverage: f64,
    pub avg_function_coverage: f64,
    pub file_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageTrendV2 {
    pub date: chrono::NaiveDate,
    pub avg_line_coverage: f64,
    pub avg_branch_coverage: f64,
    pub avg_function_coverage: f64,
    pub file_count: i64,
    pub total_uncovered_lines: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageEnforcementResult {
    pub passes: bool,
    pub files_checked: i64,
    pub files_passing: i64,
    pub files_failing: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageUploadRequest {
    pub file_path: String,
    pub line_coverage: f64,
    pub branch_coverage: f64,
    pub function_coverage: f64,
    pub total_lines: i32,
    pub covered_lines: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageUploadRequestV2 {
    pub file_path: String,
    pub line_coverage: f64,
    pub branch_coverage: f64,
    pub function_coverage: f64,
    pub total_lines: i32,
    pub covered_lines: i32,
    pub uncovered_lines: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageEnforcementConfig {
    pub min_line_coverage: f64,
    pub min_branch_coverage: f64,
    pub min_function_coverage: f64,
}
