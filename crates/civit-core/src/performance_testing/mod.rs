#![forbid(unsafe_code)]

pub mod store;
pub mod types;

pub use store::PerformanceTestStore;
pub use types::{
    TestType, TestStatus, PerformanceTestConfig, PerformanceTestResults,
    CreatePerformanceTestRequest, PerformanceTestSummary, PerformanceTestRecord,
    TestConfigEntry, CreateTestConfigRequest, TestResultMetric, RecordTestResultRequest,
    PercentileAnalysis, PerformanceComparison, MetricComparison,
    PerformanceBaseline, CreatePerformanceBaselineRequest, UpdatePerformanceBaselineRequest,
    PerformanceRegression, RegressionStatusUpdate,
    PerformanceTrendData, RecordTrendDataRequest, PerformanceTrendAnalysis,
    PerformanceAlert, PerformanceBaselineSummary,
    PerformanceTestAlertConfig, CreateAlertConfigRequest, UpdateAlertConfigRequest,
    PerformanceAlertHistory, AlertNotification, AlertAnalytics, AlertTriggerTrend,
    PerformanceTestAlertConfigV3, CreateAlertConfigV3Request, UpdateAlertConfigV3Request,
    PerformanceAlertHistoryV3, AlertNotificationV3, AlertAnalyticsV3, AlertTriggerTrendV3,
    PerformanceTestAlertConfigV5, CreateAlertConfigV5Request, UpdateAlertConfigV5Request,
    PerformanceAlertHistoryV5, AlertNotificationV5, AlertAnalyticsV5, AlertTriggerTrendV5,
    PerformanceTestAlertConfigV9, CreateAlertConfigV9Request, UpdateAlertConfigV9Request,
    PerformanceAlertHistoryV9, AlertNotificationV9, AlertAnalyticsV9, AlertTriggerTrendV9,
    PerformanceTestAlertConfigV16, CreateAlertConfigV16Request, UpdateAlertConfigV16Request,
    PerformanceAlertHistoryV16, AlertNotificationV16, AlertAnalyticsV16, AlertTriggerTrendV16,
    PerformanceTestAlertConfigV18, CreateAlertConfigV18Request, UpdateAlertConfigV18Request,
    PerformanceAlertHistoryV18, AlertNotificationV18, AlertAnalyticsV18, AlertTriggerTrendV18,
    PerformanceTestComparisonV20, CreatePerformanceTestComparisonV20Request,
    PerformanceTestRegressionsV20, CreatePerformanceTestRegressionsV20Request,
    UpdatePerformanceTestRegressionsV20Request,
    ComparisonAnalysisResultV20, RegressionDetectionResultV20,
    PerformanceBudgetV23, CreatePerformanceBudgetV23Request, UpdatePerformanceBudgetV23Request,
    PerformanceBudgetCheckV23, PerformanceTrendAnalysisV23,
};
