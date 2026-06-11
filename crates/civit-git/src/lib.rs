#![forbid(unsafe_code)]

pub mod archive;
pub mod blame;
pub mod diff;
pub mod graph;
pub mod operations;
pub mod tree;

pub use archive::{ArchiveFormat, ArchiveResult, generate_archive};
pub use blame::{BlameLine, BlameResult, git_blame};
pub use diff::{DiffEntry, DiffResult, generate_diff};
pub use graph::{CommitGraphNode, CommitGraphEdge, GraphBranchInfo, CommitGraph};
pub use operations::{
    CloneResult, CommitInfo, GitService, MergeResult, MergeStrategy,
};
pub use tree::{BlobResult, LanguageStats, TreeEntry, read_blob, read_tree, language_stats};
