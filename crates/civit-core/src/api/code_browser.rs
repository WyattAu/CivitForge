#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::error::CoreError;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct TreeEntry {
    pub path: String,
    pub entry_type: String,
    pub size: u64,
    #[serde(default)]
    pub last_commit: Option<CommitSummary>,
    /// For submodules: the URL of the submodule (parsed from .gitmodules).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub submodule_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommitSummary {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub time: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedTreeResponse {
    pub entries: Vec<TreeEntry>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BlobResponse {
    pub path: String,
    pub content: String,
    pub size: u64,
    pub encoding: String,
    #[serde(default)]
    pub language: String,
}

/// Response for the language stats endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct LanguageStatsResponse {
    pub languages: Vec<LanguageEntry>,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LanguageEntry {
    pub name: String,
    pub bytes: u64,
    pub percentage: f64,
    #[serde(default)]
    pub color: String,
}

/// Response for the README endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct ReadmeResponse {
    pub path: String,
    pub content: String,
    #[serde(default)]
    pub encoding: String,
}

/// Response for file commit history.
#[derive(Debug, Clone, Serialize)]
pub struct FileCommitEntry {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub time: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileCommitsResponse {
    pub commits: Vec<FileCommitEntry>,
    pub path: String,
    pub total: usize,
}

/// Response for git blame.
#[derive(Debug, Clone, Serialize)]
pub struct BlameLine {
    pub line_number: usize,
    pub content: String,
    #[serde(default)]
    pub commit_id: String,
    #[serde(default)]
    pub commit_message: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub time: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BlameResponse {
    pub lines: Vec<BlameLine>,
    pub path: String,
    pub language: String,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct TreeQueryParams {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub ref_: Option<String>,
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_per_page")]
    pub per_page: usize,
}

impl Default for TreeQueryParams {
    fn default() -> Self {
        Self {
            path: None,
            ref_: None,
            page: default_page(),
            per_page: default_per_page(),
        }
    }
}

#[allow(dead_code)]
fn default_ref() -> Option<String> {
    None
}

fn default_page() -> usize {
    1
}

fn default_per_page() -> usize {
    50
}

pub fn code_browser_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/repos/{owner}/{name}/tree", get(list_tree))
        .route("/api/v1/repos/{owner}/{name}/blob", get(read_blob))
        .route("/api/v1/repos/{owner}/{name}/readme", get(read_readme))
        .route(
            "/api/v1/repos/{owner}/{name}/languages",
            get(language_stats),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/file-commits",
            get(file_commits),
        )
        .route("/api/v1/repos/{owner}/{name}/blame", get(blame_file))
        .route("/api/v1/repos/{owner}/{name}/size", get(repo_size))
        .route("/api/v1/repos/{owner}/{name}/graph", get(commit_graph))
}
/// Convert a gix object error into our CoreError.
fn git_err(e: impl std::fmt::Display) -> CoreError {
    CoreError::Git(e.to_string())
}

/// Parse `.gitmodules` file in the repo and return a map of submdule path → URL.
/// Returns an empty map if .gitmodules doesn't exist or can't be parsed.
fn parse_gitmodules(repo_path: &std::path::Path) -> std::collections::HashMap<String, String> {
    let gitmodules = repo_path.join(".gitmodules");
    let content = match std::fs::read_to_string(&gitmodules) {
        Ok(c) => c,
        Err(_) => return std::collections::HashMap::new(),
    };

    let mut map = std::collections::HashMap::new();
    let mut current_path: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(path) = trimmed.strip_prefix("path = ") {
            current_path = Some(path.trim().to_string());
        } else if let Some(url) = trimmed.strip_prefix("url = ") {
            if let Some(ref path) = current_path {
                map.insert(path.clone(), url.trim().to_string());
            }
        } else if trimmed.starts_with("[submodule") {
            current_path = None;
        }
    }
    map
}

/// List files and directories at a given path in a repo's default branch.
pub async fn list_tree(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(params): Query<TreeQueryParams>,
) -> impl IntoResponse {
    let repo_path = state.git_service.repo_path(&owner, &name);

    if !repo_path.join("HEAD").exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response();
    }

    let repo = match gix::open(&repo_path) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    // Handle empty repo (no commits yet)
    let head_id = match repo.head_id() {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::OK, Json(Vec::<TreeEntry>::new())).into_response();
        }
    };

    let commit_obj = match head_id.object() {
        Ok(o) => o,
        Err(_) => {
            return (StatusCode::OK, Json(Vec::<TreeEntry>::new())).into_response();
        }
    };

    let commit = match commit_obj.try_into_commit() {
        Ok(c) => c,
        Err(_) => {
            return (StatusCode::OK, Json(Vec::<TreeEntry>::new())).into_response();
        }
    };

    // Extract HEAD commit summary for tree entries
    let head_summary = {
        let id_hex = head_id.to_hex().to_string();
        let message = commit
            .message()
            .map(|m| m.title.to_string().trim_end().to_string())
            .unwrap_or_default();
        let author = commit
            .author()
            .map(|a| a.name.to_string().trim_end().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());
        let time = commit
            .time()
            .ok()
            .map(|t| {
                let dt = chrono::DateTime::from_timestamp(t.seconds, 0).unwrap_or_default();
                dt.format("%Y-%m-%d").to_string()
            })
            .unwrap_or_default();
        CommitSummary {
            id: id_hex.chars().take(7).collect(),
            message,
            author,
            time,
        }
    };

    let tree_id = match commit.tree_id() {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let tree_obj = match tree_id.object() {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let tree = match tree_obj.try_into_tree() {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    // If a path prefix is given, navigate to that subtree first.
    let prefix = params.path.as_deref().unwrap_or("");
    let tree = if prefix.is_empty() {
        tree // root tree
    } else {
        match tree.lookup_entry_by_path(prefix) {
            Ok(Some(entry)) => {
                let mode = entry.mode();
                if !mode.is_tree() {
                    // The path points to a file, not a directory — return empty
                    return (StatusCode::OK, Json(Vec::<TreeEntry>::new())).into_response();
                }
                let entry_obj = match entry.object() {
                    Ok(o) => o,
                    Err(e) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(git_err(e).error_response()),
                        )
                            .into_response();
                    }
                };
                match entry_obj.try_into_tree() {
                    Ok(t) => t,
                    Err(e) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(git_err(e).error_response()),
                        )
                            .into_response();
                    }
                }
            }
            Ok(None) => {
                // Path doesn't exist in tree
                return (StatusCode::OK, Json(Vec::<TreeEntry>::new())).into_response();
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(git_err(e).error_response()),
                )
                    .into_response();
            }
        }
    };

    let mut entries = Vec::new();

    // Parse .gitmodules to extract submodule URLs
    let submodule_urls: std::collections::HashMap<String, String> = parse_gitmodules(&repo_path);

    for entry_result in tree.iter() {
        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue,
        };
        // Always use relative filename (not full path from root)
        let entry_path = entry.filename().to_string();

        let mode = entry.mode();
        let (entry_type, size) = if mode.is_tree() {
            ("dir".to_string(), 0u64)
        } else if mode.is_blob() {
            let sz = entry
                .object()
                .ok()
                .and_then(|o| o.try_into_blob().ok())
                .map(|b| b.data.len() as u64)
                .unwrap_or(0);
            ("file".to_string(), sz)
        } else if mode.is_link() {
            let sz = entry
                .object()
                .ok()
                .and_then(|o| o.try_into_blob().ok())
                .map(|b| b.data.len() as u64)
                .unwrap_or(0);
            ("symlink".to_string(), sz)
        } else if mode.is_commit() {
            ("submodule".to_string(), 0u64)
        } else {
            ("unknown".to_string(), 0u64)
        };

        let submodule_url = if entry_type == "submodule" {
            submodule_urls.get(&entry_path).cloned().unwrap_or_default()
        } else {
            String::new()
        };

        entries.push(TreeEntry {
            path: entry_path,
            entry_type,
            size,
            last_commit: Some(head_summary.clone()),
            submodule_url,
        });
    }

    entries.sort_by(|a, b| match (&a.entry_type, &b.entry_type) {
        (t, o) if t == "dir" && o != "dir" => std::cmp::Ordering::Less,
        (t, o) if t != "dir" && o == "dir" => std::cmp::Ordering::Greater,
        _ => a.path.cmp(&b.path),
    });

    let total = entries.len();
    let per_page = params.per_page.clamp(1, 500);
    let page = params.page.max(1);
    let start = (page - 1) * per_page;
    let paginated = entries.into_iter().skip(start).take(per_page).collect();
    let total_pages = total.div_ceil(per_page);

    (
        StatusCode::OK,
        Json(PaginatedTreeResponse {
            entries: paginated,
            total,
            page,
            per_page,
            total_pages,
        }),
    )
        .into_response()
}

/// Read file content at a given path in a repo's default branch.
pub async fn read_blob(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(params): Query<TreeQueryParams>,
) -> impl IntoResponse {
    let file_path = match params.path {
        Some(ref p) if !p.is_empty() => p.clone(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("path query parameter is required".into()).error_response()),
            )
                .into_response();
        }
    };

    let repo_path = state.git_service.repo_path(&owner, &name);

    if !repo_path.join("HEAD").exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response();
    }

    let repo = match gix::open(&repo_path) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    // Handle empty repo (no commits yet)
    let head_id = match repo.head_id() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository is empty".into()).error_response()),
            )
                .into_response();
        }
    };

    let commit_obj = match head_id.object() {
        Ok(o) => o,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository is empty".into()).error_response()),
            )
                .into_response();
        }
    };

    let commit = match commit_obj.try_into_commit() {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository is empty".into()).error_response()),
            )
                .into_response();
        }
    };

    let tree_id = match commit.tree_id() {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let tree_obj = match tree_id.object() {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let tree = match tree_obj.try_into_tree() {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let entry = match tree.lookup_entry_by_path(&file_path) {
        Ok(Some(e)) => e,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound(format!("file not found: {file_path}")).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let mode = entry.mode();
    if mode.is_tree() {
        let entry_obj = match entry.object() {
            Ok(o) => o,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(git_err(e).error_response()),
                )
                    .into_response();
            }
        };
        let subtree = match entry_obj.try_into_tree() {
            Ok(t) => t,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(git_err(e).error_response()),
                )
                    .into_response();
            }
        };

        let mut entries = Vec::new();
        for sub_result in subtree.iter() {
            let sub_entry = match sub_result {
                Ok(e) => e,
                Err(_) => continue,
            };
            let sub_mode = sub_entry.mode();
            let (entry_type, size) = if sub_mode.is_tree() {
                ("dir".to_string(), 0u64)
            } else if sub_mode.is_blob() {
                let sz = sub_entry
                    .object()
                    .ok()
                    .and_then(|o| o.try_into_blob().ok())
                    .map(|b| b.data.len() as u64)
                    .unwrap_or(0);
                ("file".to_string(), sz)
            } else if sub_mode.is_link() {
                ("symlink".to_string(), 0u64)
            } else if sub_mode.is_commit() {
                ("submodule".to_string(), 0u64)
            } else {
                ("unknown".to_string(), 0u64)
            };

            entries.push(TreeEntry {
                path: sub_entry.filename().to_string(),
                entry_type,
                size,
                last_commit: None,
                submodule_url: String::new(),
            });
        }

        entries.sort_by(|a, b| match (&a.entry_type, &b.entry_type) {
            (t, o) if t == "dir" && o != "dir" => std::cmp::Ordering::Less,
            (t, o) if t != "dir" && o == "dir" => std::cmp::Ordering::Greater,
            _ => a.path.cmp(&b.path),
        });

        return (StatusCode::OK, Json(entries)).into_response();
    }

    let blob_obj = match entry.object() {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let blob = match blob_obj.try_into_blob() {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let data = &blob.data;
    let (content, encoding) = match String::from_utf8(data.to_vec()) {
        Ok(s) => (s, "utf-8".to_string()),
        Err(_) => (base64_encode(data), "base64".to_string()),
    };

    let resp = BlobResponse {
        path: file_path.clone(),
        content,
        size: data.len() as u64,
        encoding,
        language: detect_language(&file_path),
    };

    (StatusCode::OK, Json(resp)).into_response()
}

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

/// Detect programming language from file extension.
/// Returns a highlight.js-compatible language identifier.
fn detect_language(path: &str) -> String {
    let ext = file_extension(path);
    match ext.as_str() {
        "rs" => "rust",
        "py" => "python",
        "js" | "mjs" | "cjs" => "javascript",
        "ts" | "mts" | "cts" => "typescript",
        "jsx" => "javascript",
        "tsx" => "typescript",
        "go" => "go",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "cpp",
        "cs" => "csharp",
        "rb" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "scala" => "scala",
        "r" => "r",
        "lua" => "lua",
        "pl" | "pm" => "perl",
        "ex" | "exs" => "elixir",
        "erl" => "erlang",
        "hs" => "haskell",
        "ml" | "mli" => "ocaml",
        "fs" | "fsi" => "fsharp",
        "dart" => "dart",
        "zig" => "zig",
        "nim" => "nim",
        "v" => "v",
        "sol" => "solidity",
        "toml" => "ini",
        "yaml" | "yml" => "yaml",
        "json" => "json",
        "xml" | "xsl" | "xslt" | "xsd" | "svg" | "html" | "htm" => "xml",
        "md" | "markdown" => "markdown",
        "sh" | "bash" | "zsh" | "fish" => "bash",
        "ps1" => "powershell",
        "sql" => "sql",
        "dockerfile" => "dockerfile",
        "cmake" => "cmake",
        "makefile" => "makefile",
        "proto" => "protobuf",
        "graphql" | "gql" => "graphql",
        "css" => "css",
        "scss" | "sass" => "scss",
        "tf" => "hcl",
        "hcl" => "hcl",
        "lock" => "json",
        "mod" => "rust",
        "rs.in" => "rust",
        "gradle" => "groovy",
        "groovy" => "groovy",
        "vue" => "xml",
        "svelte" => "xml",
        _ => "",
    }
    .to_string()
}

/// Get a display-friendly language name from extension.
fn language_display_name(ext: &str) -> &'static str {
    match ext {
        "rs" | "mod" => "Rust",
        "py" | "pyi" => "Python",
        "js" | "mjs" | "cjs" => "JavaScript",
        "ts" | "mts" | "cts" => "TypeScript",
        "jsx" => "JSX",
        "tsx" => "TSX",
        "go" => "Go",
        "java" => "Java",
        "kt" | "kts" => "Kotlin",
        "c" | "h" => "C",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "C++",
        "cs" => "C#",
        "rb" => "Ruby",
        "php" => "PHP",
        "swift" => "Swift",
        "scala" => "Scala",
        "r" => "R",
        "lua" => "Lua",
        "pl" | "pm" => "Perl",
        "ex" | "exs" => "Elixir",
        "erl" => "Erlang",
        "hs" => "Haskell",
        "ml" | "mli" => "OCaml",
        "fs" | "fsi" => "F#",
        "dart" => "Dart",
        "zig" => "Zig",
        "nim" => "Nim",
        "v" => "V",
        "sol" => "Solidity",
        "toml" => "TOML",
        "yaml" | "yml" => "YAML",
        "json" | "lock" => "JSON",
        "xml" | "xsl" | "xslt" | "xsd" | "svg" | "html" | "htm" | "vue" | "svelte" => "XML/HTML",
        "md" | "markdown" => "Markdown",
        "sh" | "bash" | "zsh" | "fish" => "Shell",
        "ps1" => "PowerShell",
        "sql" => "SQL",
        "dockerfile" => "Dockerfile",
        "cmake" => "CMake",
        "makefile" => "Makefile",
        "proto" => "Protocol Buffers",
        "graphql" | "gql" => "GraphQL",
        "css" => "CSS",
        "scss" | "sass" => "SCSS",
        "tf" | "hcl" => "HCL",
        "gradle" | "groovy" => "Groovy",
        _ => "Other",
    }
}

/// Approximate color for a language (used in stats bar).
fn language_color(name: &str) -> &'static str {
    match name {
        "Rust" => "#dea584",
        "Python" => "#3572A5",
        "JavaScript" => "#f1e05a",
        "TypeScript" => "#3178c6",
        "Go" => "#00ADD8",
        "Java" => "#b07219",
        "Kotlin" => "#A97BFF",
        "C" => "#555555",
        "C++" => "#f34b7d",
        "C#" => "#178600",
        "Ruby" => "#701516",
        "PHP" => "#4F5D95",
        "Swift" => "#F05138",
        "Scala" => "#c22d40",
        "Shell" => "#89e051",
        "HTML" => "#e34c26",
        "CSS" => "#563d7c",
        "SCSS" => "#c6538c",
        "Markdown" => "#083fa1",
        "JSON" => "#292929",
        "YAML" => "#cb171e",
        "TOML" => "#9c4221",
        "Dart" => "#00B4AB",
        "Zig" => "#ec915c",
        "Lua" => "#000080",
        "SQL" => "#e38c00",
        "Dockerfile" => "#384d54",
        "Makefile" => "#427819",
        "Nix" => "#7e7eff",
        "Elixir" => "#6e4a7e",
        "Haskell" => "#5e5086",
        "Protocol Buffers" => "#7e6fc0",
        _ => "#8b8b8b",
    }
}

/// File extension → bytes, for language stats aggregation.
fn file_extension(path: &str) -> String {
    // Handle special filenames like Dockerfile, Makefile without extensions
    let filename = path.rsplit('/').next().unwrap_or(path);
    let lower = filename.to_lowercase();
    if lower == "dockerfile" {
        return "dockerfile".to_string();
    }
    if lower == "makefile" || lower.ends_with(".mk") {
        return "makefile".to_string();
    }
    if lower == "cmakelists.txt" {
        return "cmake".to_string();
    }
    let ext = path.rsplit('.').next().unwrap_or("");
    ext.to_lowercase()
}

/// Read README file from repo HEAD.
/// Tries README.md, README.rst, README.txt, README in that order.
pub async fn read_readme(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let repo_path = state.git_service.repo_path(&owner, &name);

    if !repo_path.join("HEAD").exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response();
    }

    let repo = match gix::open(&repo_path) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let head_id = match repo.head_id() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository is empty".into()).error_response()),
            )
                .into_response();
        }
    };

    let commit = match head_id.object().ok().and_then(|o| o.try_into_commit().ok()) {
        Some(c) => c,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository is empty".into()).error_response()),
            )
                .into_response();
        }
    };

    let tree_id = match commit.tree_id() {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let tree = match tree_id.object().ok().and_then(|o| o.try_into_tree().ok()) {
        Some(t) => t,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("no tree found".into()).error_response()),
            )
                .into_response();
        }
    };

    // Try README candidates in priority order
    let candidates = [
        "README.md",
        "README.rst",
        "README.txt",
        "README",
        "Readme.md",
    ];
    let mut found_content = None;
    let mut found_path = None;

    for candidate in &candidates {
        if let Ok(Some(entry)) = tree.lookup_entry_by_path(candidate) {
            if let Some(blob) = entry.object().ok().and_then(|o| o.try_into_blob().ok()) {
                let (content, encoding) = match String::from_utf8(blob.data.to_vec()) {
                    Ok(s) => (s, "utf-8".to_string()),
                    Err(_) => (base64_encode(&blob.data), "base64".to_string()),
                };
                found_content = Some((content, encoding));
                found_path = Some(candidate.to_string());
                break;
            }
        }
    }

    match (found_content, found_path) {
        (Some((content, encoding)), Some(readme_path)) => (
            StatusCode::OK,
            Json(ReadmeResponse {
                path: readme_path,
                content,
                encoding,
            }),
        )
            .into_response(),
        _ => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("no README file found".into()).error_response()),
        )
            .into_response(),
    }
}

/// Get language breakdown for all files in the repo.
pub async fn language_stats(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let repo_path = state.git_service.repo_path(&owner, &name);

    if !repo_path.join("HEAD").exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response();
    }

    let repo = match gix::open(&repo_path) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let head_id = match repo.head_id() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::OK,
                Json(LanguageStatsResponse {
                    languages: Vec::new(),
                    total_bytes: 0,
                }),
            )
                .into_response();
        }
    };

    let commit = match head_id.object().ok().and_then(|o| o.try_into_commit().ok()) {
        Some(c) => c,
        None => {
            return (
                StatusCode::OK,
                Json(LanguageStatsResponse {
                    languages: Vec::new(),
                    total_bytes: 0,
                }),
            )
                .into_response();
        }
    };

    let tree = match commit
        .tree_id()
        .ok()
        .and_then(|id| id.object().ok())
        .and_then(|o| o.try_into_tree().ok())
    {
        Some(t) => t,
        None => {
            return (
                StatusCode::OK,
                Json(LanguageStatsResponse {
                    languages: Vec::new(),
                    total_bytes: 0,
                }),
            )
                .into_response();
        }
    };

    // Walk the entire tree recursively and aggregate by language
    let mut lang_bytes: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    collect_tree_sizes(&tree, "", &mut lang_bytes);

    let total_bytes: u64 = lang_bytes.values().sum();
    let mut languages: Vec<LanguageEntry> = lang_bytes
        .into_iter()
        .map(|(ext, bytes)| {
            let display = language_display_name(&ext);
            let percentage = if total_bytes > 0 {
                (bytes as f64 / total_bytes as f64) * 100.0
            } else {
                0.0
            };
            LanguageEntry {
                name: display.to_string(),
                bytes,
                percentage,
                color: language_color(display).to_string(),
            }
        })
        .collect();

    // Sort by bytes descending
    languages.sort_by(|a, b| b.bytes.cmp(&a.bytes));

    (
        StatusCode::OK,
        Json(LanguageStatsResponse {
            languages,
            total_bytes,
        }),
    )
        .into_response()
}

/// Recursively walk tree and aggregate file sizes by extension.
fn collect_tree_sizes(
    tree: &gix::Tree<'_>,
    prefix: &str,
    lang_bytes: &mut std::collections::HashMap<String, u64>,
) {
    for entry_result in tree.iter() {
        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue,
        };
        let mode = entry.mode();
        if mode.is_tree() {
            if let Some(subtree) = entry.object().ok().and_then(|o| o.try_into_tree().ok()) {
                let sub_prefix = if prefix.is_empty() {
                    entry.filename().to_string()
                } else {
                    format!("{}/{}", prefix, entry.filename())
                };
                collect_tree_sizes(&subtree, &sub_prefix, lang_bytes);
            }
        } else if mode.is_blob() {
            let size = entry
                .object()
                .ok()
                .and_then(|o| o.try_into_blob().ok())
                .map(|b| b.data.len() as u64)
                .unwrap_or(0);
            let full_path = if prefix.is_empty() {
                entry.filename().to_string()
            } else {
                format!("{}/{}", prefix, entry.filename())
            };
            let ext = file_extension(&full_path);
            *lang_bytes.entry(ext).or_insert(0) += size;
        }
    }
}

/// Get commit history for a specific file path using git log.
pub async fn file_commits(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(params): Query<TreeQueryParams>,
) -> impl IntoResponse {
    let file_path = match params.path {
        Some(ref p) if !p.is_empty() => p.clone(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("path query parameter is required".into()).error_response()),
            )
                .into_response();
        }
    };

    let repo_path = state.git_service.repo_path(&owner, &name);

    if !repo_path.join("HEAD").exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response();
    }

    // Use git log subprocess for reliable commit history
    let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "/usr/bin/git".to_string());
    let ref_name = params.ref_.as_deref().unwrap_or("HEAD");

    let output = tokio::process::Command::new(&git_bin)
        .current_dir(&repo_path)
        .args([
            "log",
            "--format=%H%n%s%n%an%n%aI%n---",
            "-n",
            "50",
            ref_name,
            "--",
            &file_path,
        ])
        .output()
        .await;

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(format!("failed to run git log: {e}")).error_response()),
            )
                .into_response();
        }
    };

    if !output.status.success() {
        return (
            StatusCode::OK,
            Json(FileCommitsResponse {
                commits: Vec::new(),
                path: file_path,
                total: 0,
            }),
        )
            .into_response();
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();

    for block in stdout.split("---") {
        let lines: Vec<&str> = block.lines().collect();
        if lines.len() >= 4 {
            commits.push(FileCommitEntry {
                id: lines[0].chars().take(7).collect(),
                message: lines[1].to_string(),
                author: lines[2].to_string(),
                time: lines[3].to_string(),
            });
        }
    }

    let total = commits.len();
    (
        StatusCode::OK,
        Json(FileCommitsResponse {
            commits,
            path: file_path,
            total,
        }),
    )
        .into_response()
}

/// Get git blame for a specific file path.
pub async fn blame_file(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(params): Query<TreeQueryParams>,
) -> impl IntoResponse {
    let file_path = match params.path {
        Some(ref p) if !p.is_empty() => p.clone(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("path query parameter is required".into()).error_response()),
            )
                .into_response();
        }
    };

    let repo_path = state.git_service.repo_path(&owner, &name);

    if !repo_path.join("HEAD").exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response();
    }

    let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "/usr/bin/git".to_string());
    let ref_name = params.ref_.as_deref().unwrap_or("HEAD");

    let output = tokio::process::Command::new(&git_bin)
        .current_dir(&repo_path)
        .args(["blame", "--porcelain", ref_name, "--", &file_path])
        .output()
        .await;

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(format!("failed to run git blame: {e}")).error_response()),
            )
                .into_response();
        }
    };

    if !output.status.success() {
        return (
            StatusCode::OK,
            Json(BlameResponse {
                lines: Vec::new(),
                path: file_path.clone(),
                language: detect_language(&file_path),
            }),
        )
            .into_response();
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = Vec::new();
    let mut current_commit_id = String::new();
    let mut current_message = String::new();
    let mut current_author = String::new();
    let mut current_time = String::new();

    for line_str in stdout.lines() {
        // Porcelain blame format:
        // <hex> <orig_lineno> <lineno> [<num_lines>]
        // author <name>
        // author-time <unix-timestamp>
        // author-tz <tz>
        // summary <message>
        // filename <path>
        // <line content>
        if let Some(author) = line_str.strip_prefix("author ") {
            current_author = author.to_string();
        } else if let Some(ts_str) = line_str.strip_prefix("author-time ") {
            let ts: i64 = ts_str.parse().unwrap_or(0);
            let dt = chrono::DateTime::from_timestamp(ts, 0).unwrap_or_default();
            current_time = dt.format("%Y-%m-%d").to_string();
        } else if let Some(msg) = line_str.strip_prefix("summary ") {
            current_message = msg.to_string();
        } else if !line_str.is_empty()
            && !line_str.starts_with("author")
            && !line_str.starts_with("committer")
            && !line_str.starts_with("previous ")
            && !line_str.starts_with("filename ")
            && !line_str.contains('\t')
        {
            // This is a header line (hex + lineno info)
            let parts: Vec<&str> = line_str.split_whitespace().collect();
            if !parts.is_empty() {
                current_commit_id = parts[0].chars().take(7).collect();
            }
        } else if let Some(content) = line_str.strip_prefix('\t') {
            // Content line (starts with tab in porcelain mode)
            lines.push(BlameLine {
                line_number: 0, // Will be filled after
                content: content.to_string(),
                commit_id: current_commit_id.clone(),
                commit_message: current_message.clone(),
                author: current_author.clone(),
                time: current_time.clone(),
            });
        }
    }

    // Fill in line numbers
    for (i, blame_line) in lines.iter_mut().enumerate() {
        blame_line.line_number = i + 1;
    }

    (
        StatusCode::OK,
        Json(BlameResponse {
            lines,
            path: file_path.clone(),
            language: detect_language(&file_path),
        }),
    )
        .into_response()
}

/// Response for the repo size endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct RepoSizeResponse {
    pub size_bytes: u64,
    pub size_human: String,
}

// ── Commit Graph Types ──

#[derive(Debug, Clone, Serialize)]
pub struct GraphNode {
    pub id: String,
    pub message: String,
    pub author: String,
    pub date: String,
    pub parents: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphBranch {
    pub name: String,
    pub head: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommitGraphResponse {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub branches: Vec<GraphBranch>,
}

/// GET /repos/{owner}/{name}/graph
/// Returns commit DAG for D3.js force-directed graph rendering.
pub async fn commit_graph(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let repo_path = state.git_service.repo_path(&owner, &name);

    if !repo_path.join("HEAD").exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response();
    }

    let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "/usr/bin/git".to_string());

    // Get commit data with parent info
    let log_output = match tokio::process::Command::new(&git_bin)
        .current_dir(&repo_path)
        .args(["log", "--all", "--format=%H|%s|%an|%aI|%P", "-n", "200"])
        .output()
        .await
    {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(format!("failed to run git log: {e}")).error_response()),
            )
                .into_response();
        }
    };

    if !log_output.status.success() {
        return (
            StatusCode::OK,
            Json(CommitGraphResponse {
                nodes: Vec::new(),
                edges: Vec::new(),
                branches: Vec::new(),
            }),
        )
            .into_response();
    }

    let stdout = String::from_utf8_lossy(&log_output.stdout);
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.splitn(5, '|').collect();
        if parts.len() < 4 {
            continue;
        }
        let id = parts[0].to_string();
        let short_id: String = id.chars().take(7).collect();
        let message = parts[1].to_string();
        let author = parts[2].to_string();
        let date = parts[3].to_string();
        let parents: Vec<String> = parts
            .get(4)
            .map(|s| s.split_whitespace().map(|p| p.to_string()).collect())
            .unwrap_or_default();

        if !seen.contains(&id) {
            seen.insert(id.clone());
            nodes.push(GraphNode {
                id: short_id,
                message,
                author,
                date,
                parents: parents
                    .iter()
                    .map(|p| {
                        let mut short = p.chars().take(7).collect::<String>();
                        // Try to find matching node
                        if !seen.contains(p) {
                            short = p.chars().take(7).collect();
                        }
                        short
                    })
                    .collect(),
            });
        }

        for parent_id in &parents {
            let parent_short: String = parent_id.chars().take(7).collect();
            let child_short: String = id.chars().take(7).collect();
            edges.push(GraphEdge {
                from: parent_short,
                to: child_short,
            });
        }
    }

    // Get branch heads
    let branch_output = tokio::process::Command::new(&git_bin)
        .current_dir(&repo_path)
        .args(["branch", "--format=%(refname:short)|%(objectname)"])
        .output()
        .await;

    let branches = match branch_output {
        Ok(o) if o.status.success() => {
            let out = String::from_utf8_lossy(&o.stdout);
            out.lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.splitn(2, '|').collect();
                    if parts.len() == 2 {
                        Some(GraphBranch {
                            name: parts[0].to_string(),
                            head: parts[1].chars().take(7).collect(),
                        })
                    } else {
                        None
                    }
                })
                .collect()
        }
        _ => Vec::new(),
    };

    let resp = CommitGraphResponse {
        nodes,
        edges,
        branches,
    };
    (StatusCode::OK, Json(resp)).into_response()
}

/// Get the on-disk size of a repository.
pub async fn repo_size(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let repo_path = state.git_service.repo_path(&owner, &name);

    if !repo_path.join("HEAD").exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response();
    }

    let size_bytes = match std::fs::metadata(&repo_path).and_then(|_m| {
        fn dir_size(path: &std::path::Path) -> std::io::Result<u64> {
            let mut total = 0u64;
            if path.is_dir() {
                for entry in std::fs::read_dir(path)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        total += dir_size(&path)?;
                    } else {
                        total += entry.metadata()?.len();
                    }
                }
            } else {
                total = std::fs::metadata(path)?.len();
            }
            Ok(total)
        }
        dir_size(&repo_path)
    }) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Internal(format!("failed to compute size: {e}")).error_response()),
            )
                .into_response();
        }
    };

    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    let size_human = if size_bytes < KB {
        format!("{size_bytes} B")
    } else if size_bytes < MB {
        format!("{:.1} KB", size_bytes as f64 / KB as f64)
    } else if size_bytes < GB {
        format!("{:.1} MB", size_bytes as f64 / MB as f64)
    } else {
        format!("{:.2} GB", size_bytes as f64 / GB as f64)
    };

    (
        StatusCode::OK,
        Json(RepoSizeResponse {
            size_bytes,
            size_human,
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_entry_serialization() {
        let entry = TreeEntry {
            path: "src/main.rs".into(),
            entry_type: "file".into(),
            size: 1024,
            last_commit: Some(CommitSummary {
                id: "abc123".into(),
                message: "fix: update".into(),
                author: "test".into(),
                time: "2024-01-01".into(),
            }),
            submodule_url: String::new(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"entry_type\":\"file\""));
        assert!(json.contains("\"size\":1024"));
        assert!(json.contains("\"last_commit\""));
    }

    #[test]
    fn test_tree_entry_dir() {
        let entry = TreeEntry {
            path: "src".into(),
            entry_type: "dir".into(),
            size: 0,
            last_commit: None,
            submodule_url: String::new(),
        };
        assert_eq!(entry.entry_type, "dir");
    }

    #[test]
    fn test_blob_response_serialization() {
        let resp = BlobResponse {
            path: "README.md".into(),
            content: "Hello World".into(),
            size: 11,
            encoding: "utf-8".into(),
            language: "markdown".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"encoding\":\"utf-8\""));
        assert!(json.contains("\"content\":\"Hello World\""));
        assert!(json.contains("\"language\":\"markdown\""));
    }

    #[test]
    fn test_tree_query_params_defaults() {
        let params = TreeQueryParams::default();
        assert!(params.path.is_none());
        assert!(params.ref_.is_none());
        assert_eq!(params.page, 1);
        assert_eq!(params.per_page, 50);
    }

    #[test]
    fn test_tree_query_params_parse() {
        let json = r#"{"path":"src/main.rs","page":2,"per_page":100}"#;
        let params: TreeQueryParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.path.as_deref(), Some("src/main.rs"));
        assert_eq!(params.page, 2);
        assert_eq!(params.per_page, 100);
    }

    #[test]
    fn test_tree_sorting_dirs_first() {
        let mut entries = vec![
            TreeEntry {
                path: "a.rs".into(),
                entry_type: "file".into(),
                size: 1,
                last_commit: None,
                submodule_url: String::new(),
            },
            TreeEntry {
                path: "src".into(),
                entry_type: "dir".into(),
                size: 0,
                last_commit: None,
                submodule_url: String::new(),
            },
            TreeEntry {
                path: "b.rs".into(),
                entry_type: "file".into(),
                size: 2,
                last_commit: None,
                submodule_url: String::new(),
            },
        ];
        entries.sort_by(|a, b| match (&a.entry_type, &b.entry_type) {
            (t, o) if t == "dir" && o != "dir" => std::cmp::Ordering::Less,
            (t, o) if t != "dir" && o == "dir" => std::cmp::Ordering::Greater,
            _ => a.path.cmp(&b.path),
        });
        assert_eq!(entries[0].path, "src");
        assert_eq!(entries[1].path, "a.rs");
        assert_eq!(entries[2].path, "b.rs");
    }

    #[test]
    fn test_base64_encode() {
        use base64::Engine;
        let encoded = base64_encode(b"hello");
        assert_eq!(
            encoded,
            base64::engine::general_purpose::STANDARD.encode(b"hello")
        );
    }

    #[test]
    fn test_code_browser_routes_type() {
        fn _assert_routes() -> Router<AppState> {
            code_browser_routes()
        }
    }

    #[test]
    fn test_tree_entry_unknown_type() {
        let entry = TreeEntry {
            path: "special".into(),
            entry_type: "unknown".into(),
            size: 0,
            last_commit: None,
            submodule_url: String::new(),
        };
        assert_eq!(entry.entry_type, "unknown");
    }

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language("main.rs"), "rust");
        assert_eq!(detect_language("app.py"), "python");
        assert_eq!(detect_language("index.tsx"), "typescript");
        assert_eq!(detect_language("style.css"), "css");
        assert_eq!(detect_language("Cargo.toml"), "ini");
        assert_eq!(detect_language("Makefile"), "makefile");
        assert_eq!(detect_language(".gitignore"), "");
        assert_eq!(detect_language("Dockerfile"), "dockerfile");
    }

    #[test]
    fn test_language_display_name() {
        assert_eq!(language_display_name("rs"), "Rust");
        assert_eq!(language_display_name("py"), "Python");
        assert_eq!(language_display_name("toml"), "TOML");
        assert_eq!(language_display_name("xyz"), "Other");
    }

    #[test]
    fn test_file_extension() {
        assert_eq!(file_extension("src/main.rs"), "rs");
        assert_eq!(file_extension("Dockerfile"), "dockerfile");
        assert_eq!(file_extension("Makefile"), "makefile");
        assert_eq!(file_extension("CMakeLists.txt"), "cmake");
        assert_eq!(file_extension("Cargo.lock"), "lock");
    }

    #[test]
    fn test_language_stats_response_serialization() {
        let resp = LanguageStatsResponse {
            languages: vec![LanguageEntry {
                name: "Rust".to_string(),
                bytes: 1000,
                percentage: 80.0,
                color: "#dea584".to_string(),
            }],
            total_bytes: 1250,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"name\":\"Rust\""));
        assert!(json.contains("\"percentage\":80"));
    }

    #[test]
    fn test_blame_line_serialization() {
        let line = BlameLine {
            line_number: 1,
            content: "fn main() {}".to_string(),
            commit_id: "abc1234".to_string(),
            commit_message: "init".to_string(),
            author: "alice".to_string(),
            time: "2024-01-01".to_string(),
        };
        let json = serde_json::to_string(&line).unwrap();
        assert!(json.contains("\"line_number\":1"));
        assert!(json.contains("\"commit_id\":\"abc1234\""));
    }

    #[test]
    fn test_file_commits_response_serialization() {
        let resp = FileCommitsResponse {
            commits: vec![FileCommitEntry {
                id: "abc123".to_string(),
                message: "init".to_string(),
                author: "alice".to_string(),
                time: "2024-01-01T00:00:00+00:00".to_string(),
            }],
            path: "main.rs".to_string(),
            total: 1,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"total\":1"));
        assert!(json.contains("\"path\":\"main.rs\""));
    }

    #[test]
    fn test_repo_size_response_serialization() {
        let resp = RepoSizeResponse {
            size_bytes: 1048576,
            size_human: "1.0 MB".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"size_bytes\":1048576"));
        assert!(json.contains("\"size_human\":\"1.0 MB\""));
    }
}
