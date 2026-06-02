#![forbid(unsafe_code)]

pub mod activitypub;
pub mod delivery;
pub mod forgefed;
pub mod http_signatures;
pub mod inbox_outbox;
pub mod multimaster;
pub mod sync;
pub mod webfinger;

pub use activitypub::{Activity, Actor, InboxHandler};
pub use delivery::{DeliveryResult, FederationDeliveryConfig, FederationDeliveryService};
pub use forgefed::{
    CrossInstanceIdentityResolver, FederatedFork, FederatedIssue, FederatedPR, FederatedPRReview,
    FederatedRepo, FederatedStar, ForgeFedActivity, ForgeFedProcessor, IdempotencyTracker,
    IssueState, PRReviewState, PRState, ProcessingOutcome,
};
pub use multimaster::{
    BandwidthOptimizer, ConflictEntry, ConflictResolution, ConflictStrategy, DeltaCompressor,
    IncrementalSyncEngine, PartitionStatus, PartitionTracker, SyncCheckpoint, SyncDelta,
};
pub use sync::DagSyncEngine;
pub use webfinger::{HttpSignature, Link, WebFingerResponse};
