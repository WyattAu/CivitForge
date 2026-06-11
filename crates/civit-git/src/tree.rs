use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeEntry {
    pub path: String,
    pub entry_type: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobResult {
    pub path: String,
    pub content: String,
    pub size: u64,
    pub encoding: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    pub languages: Vec<LanguageEntry>,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageEntry {
    pub name: String,
    pub bytes: u64,
    pub percentage: f64,
    pub color: String,
}

pub fn read_tree(repo_path: &Path, ref_name: &str, path_prefix: &str) -> Result<Vec<TreeEntry>> {
    let repo = gix::open(repo_path).context("failed to open repo")?;

    let head_id = match ref_name {
        "HEAD" => repo.head_id().ok(),
        _ => repo.rev_parse_single(ref_name).ok(),
    };

    let head_id = match head_id {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    let commit_obj = head_id.object()?;
    let commit = commit_obj.try_into_commit()?;
    let tree_id = commit.tree_id()?;
    let tree_obj = tree_id.object()?;
    let tree = tree_obj.try_into_tree()?;

    let tree = if path_prefix.is_empty() {
        tree
    } else {
        match tree.lookup_entry_by_path(path_prefix)? {
            Some(entry) => {
                if !entry.mode().is_tree() {
                    return Ok(Vec::new());
                }
                entry.object()?.try_into_tree()?
            }
            None => return Ok(Vec::new()),
        }
    };

    let mut entries = Vec::new();
    for entry_result in tree.iter() {
        let entry = entry_result?;
        let filename = entry.filename().to_string();
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
            ("symlink".to_string(), 0u64)
        } else if mode.is_commit() {
            ("submodule".to_string(), 0u64)
        } else {
            ("unknown".to_string(), 0u64)
        };

        entries.push(TreeEntry {
            path: filename,
            entry_type,
            size,
        });
    }

    entries.sort_by(|a, b| match (&a.entry_type, &b.entry_type) {
        (t, o) if t == "dir" && o != "dir" => std::cmp::Ordering::Less,
        (t, o) if t != "dir" && o == "dir" => std::cmp::Ordering::Greater,
        _ => a.path.cmp(&b.path),
    });

    Ok(entries)
}

pub fn read_blob(repo_path: &Path, ref_name: &str, file_path: &str) -> Result<BlobResult> {
    let repo = gix::open(repo_path).context("failed to open repo")?;

    let rev_id = repo.rev_parse_single(ref_name)?;
    let commit_obj = rev_id.object()?;
    let commit = commit_obj.try_into_commit()?;
    let tree_id = commit.tree_id()?;
    let tree_obj = tree_id.object()?;
    let tree = tree_obj.try_into_tree()?;

    let entry = tree
        .lookup_entry_by_path(file_path)?
        .ok_or_else(|| anyhow::anyhow!("file not found: {file_path}"))?;

    if entry.mode().is_tree() {
        return Err(anyhow::anyhow!("path is a directory, not a file: {file_path}"));
    }

    let blob_obj = entry.object()?;
    let blob = blob_obj.try_into_blob()?;

    let data = &blob.data;
    let (content, encoding) = match String::from_utf8(data.to_vec()) {
        Ok(s) => (s, "utf-8".to_string()),
        Err(_) => (base64_encode(data), "base64".to_string()),
    };

    Ok(BlobResult {
        path: file_path.to_string(),
        content,
        size: data.len() as u64,
        encoding,
        language: detect_language(file_path),
    })
}

pub fn language_stats(repo_path: &Path) -> Result<LanguageStats> {
    let repo = gix::open(repo_path).context("failed to open repo")?;

    let head_id = match repo.head_id() {
        Ok(id) => id,
        Err(_) => {
            return Ok(LanguageStats {
                languages: Vec::new(),
                total_bytes: 0,
            });
        }
    };

    let commit = match head_id.object().ok().and_then(|o| o.try_into_commit().ok()) {
        Some(c) => c,
        None => {
            return Ok(LanguageStats {
                languages: Vec::new(),
                total_bytes: 0,
            });
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
            return Ok(LanguageStats {
                languages: Vec::new(),
                total_bytes: 0,
            });
        }
    };

    let mut lang_bytes: HashMap<String, u64> = HashMap::new();
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

    languages.sort_by(|a, b| b.bytes.cmp(&a.bytes));

    Ok(LanguageStats {
        languages,
        total_bytes,
    })
}

fn collect_tree_sizes(
    tree: &gix::Tree<'_>,
    prefix: &str,
    lang_bytes: &mut HashMap<String, u64>,
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

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

pub fn detect_language(path: &str) -> String {
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

fn file_extension(path: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language("main.rs"), "rust");
        assert_eq!(detect_language("app.py"), "python");
        assert_eq!(detect_language("index.tsx"), "typescript");
        assert_eq!(detect_language("style.css"), "css");
        assert_eq!(detect_language("Cargo.toml"), "ini");
        assert_eq!(detect_language("Makefile"), "makefile");
    }

    #[test]
    fn test_file_extension() {
        assert_eq!(file_extension("src/main.rs"), "rs");
        assert_eq!(file_extension("Dockerfile"), "dockerfile");
        assert_eq!(file_extension("CMakeLists.txt"), "cmake");
        assert_eq!(file_extension("style.scss"), "scss");
    }

    #[test]
    fn test_tree_entry_serialization() {
        let entry = TreeEntry {
            path: "src/main.rs".into(),
            entry_type: "file".into(),
            size: 1024,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"entry_type\":\"file\""));
        assert!(json.contains("\"size\":1024"));
    }

    #[test]
    fn test_blob_result_serialization() {
        let resp = BlobResult {
            path: "README.md".into(),
            content: "Hello World".into(),
            size: 11,
            encoding: "utf-8".into(),
            language: "markdown".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"encoding\":\"utf-8\""));
        assert!(json.contains("\"language\":\"markdown\""));
    }
}
