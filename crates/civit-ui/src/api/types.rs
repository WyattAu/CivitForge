#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub use civit_shared::ListResponse;
pub use civit_shared::pagination::Pagination;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: AuthUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    #[serde(default)]
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueResponse {
    pub id: String,
    #[serde(default, alias = "number")]
    pub number: Option<i64>,
    pub title: String,
    pub body: Option<String>,
    #[serde(alias = "status")]
    pub state: String,
    #[serde(default, alias = "author_id")]
    pub author: String,
    #[serde(default)]
    pub labels: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub comments: Vec<CommentResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentResponse {
    pub id: String,
    pub author: String,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssueBody {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommentBody {
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIssueBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPageResponse {
    pub slug: String,
    pub title: String,
    pub content: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPageListItem {
    pub slug: String,
    pub title: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiRevision {
    pub revision: i64,
    pub slug: String,
    pub title: String,
    pub content: String,
    pub author: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWikiPageBody {
    pub slug: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWikiPageBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResultItem>,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub id: String,
    pub full_name: String,
    pub description: Option<String>,
    pub stars: u64,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyResponse {
    pub id: String,
    pub user_id: String,
    pub key_type: String,
    pub public_key: String,
    pub fingerprint: String,
    pub label: String,
    pub created_at: String,
}

// ── Pull Request Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestResponse {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub number: i32,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default, alias = "state")]
    pub status: String,
    #[serde(default)]
    pub draft: bool,
    #[serde(default, alias = "head", alias = "source")]
    pub source_branch: String,
    #[serde(default, alias = "base", alias = "target")]
    pub target_branch: String,
    #[serde(default)]
    pub merge_commit_id: Option<String>,
    #[serde(default)]
    pub head_commit_sha: Option<String>,
    #[serde(default)]
    pub base_commit_sha: Option<String>,
    #[serde(default)]
    pub merge_strategy: String,
    #[serde(default, alias = "user")]
    pub author_id: String,
    #[serde(default)]
    pub repo_id: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub merged_at: Option<String>,
    #[serde(default)]
    pub closed_at: Option<String>,
    #[serde(default)]
    pub mergeable: Option<bool>,
    #[serde(default)]
    pub additions: Option<i64>,
    #[serde(default)]
    pub deletions: Option<i64>,
    #[serde(default)]
    pub changed_files: Option<i64>,
    #[serde(default)]
    pub review_status: Option<String>,
    #[serde(default)]
    pub labels: Vec<PullRequestLabel>,
    #[serde(default)]
    pub assignees: Vec<String>,
    #[serde(default)]
    pub reviewers: Vec<PullRequestReviewer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestLabel {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub repo_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestReviewer {
    #[serde(default)]
    pub user_id: String,
    #[serde(default)]
    pub review_status: String,
    pub submitted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestListResponse {
    #[serde(default, alias = "data")]
    pub items: Vec<PullRequestResponse>,
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
}

fn default_per_page() -> i32 {
    20
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrCommentResponse {
    pub id: String,
    pub author_id: String,
    pub body: String,
    pub commit_sha: Option<String>,
    pub file_path: Option<String>,
    pub line: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePullRequestBody {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    pub source_branch: String,
    pub target_branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
    #[serde(default)]
    pub assignees: Vec<String>,
    #[serde(default)]
    pub reviewers: Vec<String>,
    #[serde(default)]
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePullRequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePrCommentBody {
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergePullRequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResponse {
    pub merged: bool,
    pub message: String,
    pub merge_commit_sha: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeabilityResponse {
    pub mergeable: bool,
    pub merge_strategy: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrFileChange {
    pub path: String,
    pub status: String,
    pub additions: u32,
    pub deletions: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrDiffResponse {
    pub files: Vec<PrFileChange>,
    pub total_additions: u32,
    pub total_deletions: u32,
    pub commit_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineDiffFile {
    pub path: String,
    pub status: String,
    pub additions: u32,
    pub deletions: u32,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub old_line_no: Option<u32>,
    pub new_line_no: Option<u32>,
    pub content: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineDiffResponse {
    pub files: Vec<InlineDiffFile>,
    pub total_additions: u32,
    pub total_deletions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub date: String,
    pub parents: Vec<String>,
    #[serde(default)]
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphResponse {
    pub nodes: Vec<GraphNode>,
    pub branches: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestReviewBody {
    pub reviewers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitReviewBody {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRepoRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<String>,
}

// ── Board Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardResponse {
    pub id: String,
    pub name: String,
    pub repo_id: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub columns: Vec<BoardColumnResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardColumnResponse {
    pub id: String,
    pub name: String,
    pub board_id: String,
    pub position: i32,
    #[serde(default)]
    pub cards: Vec<BoardCardResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardCardResponse {
    pub id: String,
    pub title: String,
    pub column_id: String,
    pub position: i32,
    #[serde(default)]
    pub issue_number: Option<i64>,
    #[serde(default)]
    pub issue_id: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub assignee: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBoardBody {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBoardBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateColumnBody {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateColumnBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCardBody {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_number: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveCardBody {
    pub column_id: String,
    pub position: i32,
}

// ── Pages Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagesSiteResponse {
    pub id: String,
    pub repo_id: String,
    pub url: String,
    pub branch: String,
    pub path: String,
    pub public: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnablePagesBody {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
}

// ── Discussion Enhancement Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscussionLabelResponse {
    pub id: String,
    pub discussion_id: String,
    pub label: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscussionReactionResponse {
    pub id: String,
    pub comment_id: String,
    pub user_id: String,
    pub emoji: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddLabelBody {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddReactionBody {
    pub emoji: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscussionSearchParams {
    pub q: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
}

// ── Code Suggestion Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSuggestionResponse {
    pub id: String,
    pub pr_id: String,
    pub comment_id: Option<String>,
    pub file_path: String,
    pub start_line: i32,
    pub end_line: i32,
    pub suggestion: String,
    pub applied: bool,
    pub author_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCodeSuggestionBody {
    pub file_path: String,
    pub start_line: i32,
    pub end_line: i32,
    pub suggestion: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_id: Option<String>,
}

// ── Search Suggestion Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSuggestionItem {
    pub text: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSuggestResponse {
    pub suggestions: Vec<SearchSuggestionItem>,
}

// ── Search History Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistoryItem {
    pub id: String,
    pub query: String,
    pub result_count: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistoryResponse {
    pub items: Vec<SearchHistoryItem>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSearchHistoryBody {
    pub query: String,
    pub result_count: Option<i64>,
}
