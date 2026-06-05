#![forbid(unsafe_code)]

pub mod git_backend;

pub use git_backend::{SavePageParams, WikiGitBackend, WikiPageEntry, WikiRevision};
