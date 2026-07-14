#![forbid(unsafe_code)]

pub mod store;
pub mod types;

pub use store::TestSuiteStore;
pub use types::{
    TestSuite, TestSuiteConfig, TestRun, TestRunStatus, TestRunResult,
    CreateTestSuiteRequest, UpdateTestSuiteRequest, CreateTestRunRequest,
    TestSuiteSummary, TestRunHistory,
    TestSuiteConfiguration, CreateTestSuiteConfigRequest, UpdateTestSuiteConfigRequest,
    TestSuiteNotification, CreateTestSuiteNotificationRequest, UpdateTestSuiteNotificationRequest,
    TestSuiteAnalytics, SuiteActivity, FailureTrend, TestSchedule,
    TestSuiteTag, CreateTestSuiteTagRequest,
    TestSuiteDependency, CreateTestSuiteDependencyRequest,
    TestExecutionOrder, ExecutionPlan, TestSuiteDependencySummary,
    TestSuiteMetric, CreateTestSuiteMetricRequest,
    TestSuiteBaseline, CreateTestSuiteBaselineRequest, UpdateTestSuiteBaselineRequest,
    TestSuiteRegression, TestSuitePerformanceAlert,
    TestSuiteMetricsSummary, TestSuitePerformanceReport,
    TestSuiteMetricV2, CreateTestSuiteMetricV2Request,
    TestSuiteBaselineV2, CreateTestSuiteBaselineV2Request, UpdateTestSuiteBaselineV2Request,
    TestSuiteRegressionV2, TestSuitePerformanceAlertV2,
    TestSuiteMetricsSummaryV2, TestSuitePerformanceReportV2,
    TestSuitePerformanceAlertConfig, CreateTestSuiteAlertConfigRequest, UpdateTestSuiteAlertConfigRequest,
    TestSuiteAlertHistory,
    TestSuiteMetricV4, CreateTestSuiteMetricV4Request,
    TestSuiteBaselineV4, CreateTestSuiteBaselineV4Request, UpdateTestSuiteBaselineV4Request,
    TestSuiteRegressionV4, TestSuitePerformanceAlertV4,
    TestSuiteMetricsSummaryV4, TestSuitePerformanceReportV4,
};
