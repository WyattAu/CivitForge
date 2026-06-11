//! Artifact serving types and logic.

#![forbid(unsafe_code)]

use serde::Deserialize;
use std::path::PathBuf;

pub fn mime_from_extension(filename: &str) -> String {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "json" => "application/json",
        "yaml" | "yml" => "text/yaml",
        "xml" => "application/xml",
        "csv" => "text/csv",
        "txt" | "log" => "text/plain",
        "html" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "zip" => "application/zip",
        "gz" | "tgz" => "application/gzip",
        "tar" => "application/x-tar",
        "pdf" => "application/pdf",
        "wasm" => "application/wasm",
        "bin" | "exe" | "dll" | "so" => "application/octet-stream",
        _ => "application/octet-stream",
    }
    .to_string()
}

#[derive(Deserialize)]
pub struct ArtifactDownloadQuery {
    pub token: Option<String>,
}

pub fn artifact_storage_path(base_path: &str, owner: &str, repo: &str, artifact_id: &str) -> PathBuf {
    PathBuf::from(base_path)
        .join("artifacts")
        .join(owner)
        .join(repo)
        .join(artifact_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mime_from_extension_common() {
        assert_eq!(mime_from_extension("file.json"), "application/json");
        assert_eq!(mime_from_extension("file.yaml"), "text/yaml");
        assert_eq!(mime_from_extension("file.txt"), "text/plain");
        assert_eq!(mime_from_extension("file.zip"), "application/zip");
        assert_eq!(mime_from_extension("file.gz"), "application/gzip");
        assert_eq!(mime_from_extension("file.png"), "image/png");
        assert_eq!(mime_from_extension("file.pdf"), "application/pdf");
        assert_eq!(
            mime_from_extension("file.unknown"),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_mime_from_extension_case_insensitive() {
        assert_eq!(mime_from_extension("FILE.JSON"), "application/json");
        assert_eq!(mime_from_extension("file.PNG"), "image/png");
    }
}
