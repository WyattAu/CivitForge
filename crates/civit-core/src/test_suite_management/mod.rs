#![forbid(unsafe_code)]

pub mod store;
pub mod types;

pub use store::TestSuiteStore;
pub use types::{
    CoverageGapAnalysisV22, CreateFlakyTestDetectionV19Request, CreateTestRunRequest,
    CreateTestSuiteAlertConfigRequest, CreateTestSuiteBaselineRequest,
    CreateTestSuiteBaselineV2Request, CreateTestSuiteBaselineV4Request,
    CreateTestSuiteBaselineV8Request, CreateTestSuiteBaselineV15Request,
    CreateTestSuiteConfigRequest, CreateTestSuiteDependencyRequest, CreateTestSuiteMetricRequest,
    CreateTestSuiteMetricV2Request, CreateTestSuiteMetricV4Request, CreateTestSuiteMetricV8Request,
    CreateTestSuiteMetricV15Request, CreateTestSuiteNotificationRequest, CreateTestSuiteRequest,
    CreateTestSuiteTagRequest, CreateTestSuiteTrendV19Request, ExecutionPlan, FailureTrend,
    FlakyTestDetectionV19, FlakyTestSummaryV19, SuiteActivity, TestExecutionOrder,
    TestOptimizationSuggestionV22, TestRun, TestRunHistory, TestRunResult, TestRunStatus,
    TestSchedule, TestSuite, TestSuiteAlertHistory, TestSuiteAnalytics, TestSuiteBaseline,
    TestSuiteBaselineV2, TestSuiteBaselineV4, TestSuiteBaselineV8, TestSuiteBaselineV15,
    TestSuiteConfig, TestSuiteConfiguration, TestSuiteDependency, TestSuiteDependencySummary,
    TestSuiteMetric, TestSuiteMetricV2, TestSuiteMetricV4, TestSuiteMetricV8, TestSuiteMetricV15,
    TestSuiteMetricsSummary, TestSuiteMetricsSummaryV2, TestSuiteMetricsSummaryV4,
    TestSuiteMetricsSummaryV8, TestSuiteMetricsSummaryV15, TestSuiteNotification,
    TestSuitePerformanceAlert, TestSuitePerformanceAlertConfig, TestSuitePerformanceAlertV2,
    TestSuitePerformanceAlertV4, TestSuitePerformanceAlertV8, TestSuitePerformanceAlertV15,
    TestSuitePerformanceReport, TestSuitePerformanceReportV2, TestSuitePerformanceReportV4,
    TestSuitePerformanceReportV8, TestSuitePerformanceReportV15, TestSuiteRegression,
    TestSuiteRegressionV2, TestSuiteRegressionV4, TestSuiteRegressionV8, TestSuiteRegressionV15,
    TestSuiteSummary, TestSuiteTag, TestSuiteTrendAnalysisV19, TestSuiteTrendV19,
    UpdateFlakyTestDetectionV19Request, UpdateTestSuiteAlertConfigRequest,
    UpdateTestSuiteBaselineRequest, UpdateTestSuiteBaselineV2Request,
    UpdateTestSuiteBaselineV4Request, UpdateTestSuiteBaselineV8Request,
    UpdateTestSuiteBaselineV15Request, UpdateTestSuiteConfigRequest,
    UpdateTestSuiteNotificationRequest, UpdateTestSuiteRequest,
};
