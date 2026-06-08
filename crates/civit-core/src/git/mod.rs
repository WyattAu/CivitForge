#![forbid(unsafe_code)]

pub mod hooks;
pub mod http;
pub mod operations;

pub use hooks::{HookResult, HookRunner, PreReceiveHook, PushContext, RefNameValidator};
pub use operations::{CloneResult, CommitInfo, GitService, MergeResult, MergeStrategy};
