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
};
