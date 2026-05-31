#![forbid(unsafe_code)]

pub mod activitypub;
pub mod forgefed;
pub mod multimaster;
pub mod sync;
pub mod webfinger;

pub use activitypub::{Activity, Actor, InboxHandler};
pub use forgefed::{
    FederatedIssue, FederatedPR, FederatedRepo, ForgeFedActivity, ForgeFedProcessor,
    IdempotencyTracker, ProcessingOutcome,
};
pub use multimaster::{
    ConflictEntry, ConflictResolution, ConflictStrategy, DeltaCompressor, SyncCheckpoint,
};
pub use sync::DagSyncEngine;
pub use webfinger::{HttpSignature, Link, WebFingerResponse};
