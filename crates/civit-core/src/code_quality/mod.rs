#![forbid(unsafe_code)]

pub mod store;
pub mod types;

pub use store::CodeQualityStore;
pub use types::{
    MetricCategory, RuleType, Severity, QualityMetricReport, QualityMetricSummary, QualityTrend,
    ComplexityAnalysis, DuplicationReport, CodeSmellsReport, TechnicalDebtReport,
    RecordMetricRequest, QualityRule, CreateQualityRuleRequest, UpdateQualityRuleRequest,
    QualityRuleEnforcementResult,
    QualityRuleV2, CreateQualityRuleV2Request, UpdateQualityRuleV2Request,
    QualityRuleVersion, QualityRuleTestResult, RuleTestRequest,
    RuleVersionDiff, RuleAnalytics, RuleEnforcementTrend,
    QualityRuleV3, CreateQualityRuleV3Request, UpdateQualityRuleV3Request,
    EnforcementType, QualityRuleEnforcement, CreateEnforcementRequest, UpdateEnforcementRequest,
    EnforcementAnalytics, EnforcementTrend, EnforcementThresholdResult,
    CodeQualityMetricV3, RecordMetricV3Request,
    CodeQualityThresholdV2, CreateCodeQualityThresholdV2Request, UpdateCodeQualityThresholdV2Request,
    CodeQualityViolation, CodeQualityEnforcementReportV2, CodeQualityScoreV2,
    CodeQualityMetricSummaryV2,
    CodeQualityMetricV5, RecordMetricV5Request,
    CodeQualityThresholdV4, CreateCodeQualityThresholdV4Request, UpdateCodeQualityThresholdV4Request,
    CodeQualityViolationV2, CodeQualityEnforcementReportV3, CodeQualityScoreV3,
    CodeQualityMetricSummaryV3,
    CodeQualityMetricV9, RecordMetricV9Request,
    CodeQualityThresholdV8, CreateCodeQualityThresholdV8Request, UpdateCodeQualityThresholdV8Request,
    CodeQualityViolationV3, CodeQualityEnforcementReportV4, CodeQualityScoreV4,
    CodeQualityMetricSummaryV4,
    CodeQualityMetricV16, RecordMetricV16Request,
    CodeQualityThresholdV15, CreateCodeQualityThresholdV15Request, UpdateCodeQualityThresholdV15Request,
    CodeQualityViolationV16, CodeQualityEnforcementReportV5, CodeQualityScoreV5,
    CodeQualityMetricSummaryV5,
    CodeQualityMetricV18, RecordMetricV18Request,
    CodeQualityThresholdV18, CreateCodeQualityThresholdV18Request, UpdateCodeQualityThresholdV18Request,
    CodeQualityViolationV18, CodeQualityEnforcementReportV6, CodeQualityScoreV6,
    CodeQualityMetricSummaryV6,
    CodeQualityRuleV19, CreateCodeQualityRuleV19Request, UpdateCodeQualityRuleV19Request,
    CodeQualityRuleUsageV19, RecordRuleUsageV19Request, RuleUsageSummaryV19,
    CustomRuleCreationV22, CreateCustomRuleV22Request, RuleTestResultV22,
    RuleEffectivenessAnalysisV22, RuleEffectivenessTrendV22,
};
