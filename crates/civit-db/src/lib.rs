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
};
pub use pool::DatabasePool;
pub use repository::{DbRepository, OrgUsage};
pub use session::{Session, SessionManager};
