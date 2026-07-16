#![forbid(unsafe_code)]

pub mod error;
pub mod migrations;
pub mod models;
pub mod pool;
pub mod repository;
pub mod session;

pub use error::{DbError, Result};
pub use models::{
    ActivityEvent, BranchProtectionRule, EmailVerificationCode, Issue, Org, Pipeline, PrComment,
    PrReviewer, PrStatusCheck, PrTimeline, PullRequest, Release, ReleaseAsset, Repository, SshKey,
    Team, TeamMember, User, WebAuthnCredential, FeatureFlag, FeatureFlagEvent, AdminDashboardConfig,
    DatabaseBackup, DatabaseRecoveryPoint, DataArchive, DataMigration,
    ApiDocsV9, ApiDocsV10, ApiDocsV11, ApiDocsV12,
    RateLimitTierV7, RateLimitTierV8, RateLimitTierV9, RateLimitTierV10,
    RateLimitAlertV5, RateLimitAlertV6, RateLimitAlertV7,
    ApiAnalyticV10, ApiAnalyticV11, ApiAnalyticV12, ApiAnalyticV13,
    TestSuiteMetricV20, TestSuiteBaselineV20,
    CodeQualityMetricV20, CodeQualityThresholdV20,
    PerformanceTestAlertV21, PerformanceTestAlertHistoryV21,
};
pub use pool::DatabasePool;
pub use repository::{DbRepository, OrgUsage};
pub use session::{Session, SessionManager};
