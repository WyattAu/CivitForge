#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct LocalRepoInfo {
    pub path: String,
    pub branch: Option<String>,
    pub is_bare: bool,
    pub commit_count: usize,
}

#[tauri::command]
pub fn list_local_repos(base_path: String) -> Result<Vec<LocalRepoInfo>, String> {
    Ok(vec![])
}

#[tauri::command]
pub fn get_repo_status(path: String) -> Result<String, String> {
    Ok("".to_string())
}

#[tauri::command]
pub fn clone_repo(url: String, path: String) -> Result<String, String> {
    Ok(path)
}
