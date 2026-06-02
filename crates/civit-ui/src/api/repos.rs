#![forbid(unsafe_code)]

use super::client::ApiClient;
use super::types::ListResponse;
use civit_shared::pagination::PaginationParams;
use civit_shared::repo::RepoResponse;

pub async fn list_repos(
    client: &ApiClient,
    params: PaginationParams,
) -> Result<ListResponse<RepoResponse>, Box<dyn std::error::Error>> {
    let query = format!(
        "?per_page={}&page={}",
        params.effective_per_page(),
        params.effective_offset() / params.effective_per_page() + 1
    );
    let resp = client.get(&format!("/repos{query}")).await?;
    let data: ListResponse<RepoResponse> = resp.json().await?;
    Ok(data)
}

pub async fn get_repo(
    client: &ApiClient,
    owner: &str,
    name: &str,
) -> Result<RepoResponse, Box<dyn std::error::Error>> {
    let resp = client.get(&format!("/repos/{owner}/{name}")).await?;
    let data: RepoResponse = resp.json().await?;
    Ok(data)
}
