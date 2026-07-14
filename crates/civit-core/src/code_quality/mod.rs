#![forbid(unsafe_code)]

pub mod store;
pub mod types;

pub use store::CodeQualityStore;
pub use types::{
    MetricCategory, RuleType, Severity, QualityMetricReport, QualityMetricSummary, QualityTrend,
    ComplexityAnalysis, DuplicationReport, CodeSmellsReport, TechnicalDebtReport,
    RecordMetricRequest, QualityRule, CreateQualityRuleRequest, UpdateQualityRuleRequest,
    QualityRuleEnforcementResult,
};
