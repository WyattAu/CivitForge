use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReviewTriggerType {
    #[serde(rename = "pull_request_opened")]
    PullRequestOpened,
    #[serde(rename = "pull_request_updated")]
    PullRequestUpdated,
    #[serde(rename = "pull_request_merged")]
    PullRequestMerged,
    #[serde(rename = "issue_opened")]
    IssueOpened,
    #[serde(rename = "issue_commented")]
    IssueCommented,
    #[serde(rename = "review_requested")]
    ReviewRequested,
}

impl std::fmt::Display for ReviewTriggerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PullRequestOpened => write!(f, "pull_request_opened"),
            Self::PullRequestUpdated => write!(f, "pull_request_updated"),
            Self::PullRequestMerged => write!(f, "pull_request_merged"),
            Self::IssueOpened => write!(f, "issue_opened"),
            Self::IssueCommented => write!(f, "issue_commented"),
            Self::ReviewRequested => write!(f, "review_requested"),
        }
    }
}

impl std::str::FromStr for ReviewTriggerType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pull_request_opened" => Ok(Self::PullRequestOpened),
            "pull_request_updated" => Ok(Self::PullRequestUpdated),
            "pull_request_merged" => Ok(Self::PullRequestMerged),
            "issue_opened" => Ok(Self::IssueOpened),
            "issue_commented" => Ok(Self::IssueCommented),
            "review_requested" => Ok(Self::ReviewRequested),
            _ => Err(format!("unknown review trigger type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ReviewAction {
    #[serde(rename = "assign_reviewer")]
    AssignReviewer { reviewer: String },
    #[serde(rename = "add_label")]
    AddLabel { label: String },
    #[serde(rename = "remove_label")]
    RemoveLabel { label: String },
    #[serde(rename = "comment")]
    Comment { body: String },
    #[serde(rename = "request_review")]
    RequestReview { reviewer: String },
    #[serde(rename = "set_reviewers")]
    SetReviewers { reviewers: Vec<String> },
    #[serde(rename = "reminder")]
    Reminder { message: String, delay_hours: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewAutomationRule {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: ReviewTriggerType,
    pub conditions: serde_json::Value,
    pub actions: Vec<ReviewAction>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReviewRuleRequest {
    pub name: String,
    pub trigger_type: ReviewTriggerType,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<Vec<ReviewAction>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReviewRuleRequest {
    pub name: Option<String>,
    pub trigger_type: Option<ReviewTriggerType>,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<Vec<ReviewAction>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRuleTestResult {
    pub rule_id: Uuid,
    pub matched: bool,
    pub conditions_met: Vec<String>,
    pub conditions_failed: Vec<String>,
    pub actions_to_execute: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRuleExecutionLog {
    pub rule_id: Uuid,
    pub trigger_event: String,
    pub matched: bool,
    pub actions_executed: Vec<String>,
    pub executed_at: DateTime<Utc>,
}
