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
    ApiDocsV9, ApiDocsV10, RateLimitTierV7, RateLimitTierV8, RateLimitAlertV5, ApiAnalyticV10, ApiAnalyticV11,
};
pub use pool::DatabasePool;
pub use repository::{DbRepository, OrgUsage};
pub use session::{Session, SessionManager};
