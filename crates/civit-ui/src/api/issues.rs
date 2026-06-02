#![forbid(unsafe_code)]

use super::client::ApiClient;
use super::types::{IssueResponse, ListResponse};
use civit_shared::pagination::PaginationParams;

pub async fn list_issues(
    client: &ApiClient,
    owner: &str,
    repo: &str,
    params: PaginationParams,
) -> Result<ListResponse<IssueResponse>, Box<dyn std::error::Error>> {
    let query = format!(
        "?per_page={}&page={}",
        params.effective_per_page(),
        params.effective_offset() / params.effective_per_page() + 1
    );
    let resp = client
        .get(&format!("/repos/{owner}/{repo}/issues{query}"))
        .await?;
    let data: ListResponse<IssueResponse> = resp.json().await?;
    Ok(data)
}

pub async fn get_issue(
    client: &ApiClient,
    owner: &str,
    repo: &str,
    issue_number: i64,
) -> Result<IssueResponse, Box<dyn std::error::Error>> {
    let resp = client
        .get(&format!("/repos/{owner}/{repo}/issues/{issue_number}"))
        .await?;
    let data: IssueResponse = resp.json().await?;
    Ok(data)
}
