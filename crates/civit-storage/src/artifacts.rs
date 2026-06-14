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

pub fn artifact_storage_path(
    base_path: &str,
    owner: &str,
    repo: &str,
    artifact_id: &str,
) -> PathBuf {
    PathBuf::from(base_path)
        .join("artifacts")
        .join(owner)
        .join(repo)
        .join(artifact_id)
}

pub fn generate_presigned_url(base_url: &str, path: &str, expires_in_seconds: u64) -> String {
    if let Some(pos) = base_url.find('?') {
        let (base, query) = base_url.split_at(pos);
        format!("{base}/{path}{query}&expires={expires_in_seconds}")
    } else {
        format!("{base_url}/{path}?expires={expires_in_seconds}")
    }
}

pub fn validate_presigned_url(url: &str) -> Result<u64, String> {
    let expires_pos = url.find("expires=").ok_or("missing expires parameter")?;
    let expires_str = &url[expires_pos + 8..];
    expires_str
        .parse::<u64>()
        .map_err(|_| "invalid expires value".into())
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

    #[test]
    fn test_mime_from_extension_yml() {
        assert_eq!(mime_from_extension("config.yml"), "text/yaml");
    }

    #[test]
    fn test_mime_from_extension_jpeg() {
        assert_eq!(mime_from_extension("photo.jpeg"), "image/jpeg");
    }

    #[test]
    fn test_mime_from_extension_tgz() {
        assert_eq!(mime_from_extension("archive.tgz"), "application/gzip");
    }

    #[test]
    fn test_mime_from_extension_binary_types() {
        assert_eq!(mime_from_extension("app.bin"), "application/octet-stream");
        assert_eq!(mime_from_extension("app.exe"), "application/octet-stream");
        assert_eq!(mime_from_extension("lib.dll"), "application/octet-stream");
        assert_eq!(mime_from_extension("lib.so"), "application/octet-stream");
    }

    #[test]
    fn test_mime_from_extension_web_types() {
        assert_eq!(mime_from_extension("index.html"), "text/html");
        assert_eq!(mime_from_extension("style.css"), "text/css");
        assert_eq!(mime_from_extension("app.js"), "application/javascript");
        assert_eq!(mime_from_extension("icon.svg"), "image/svg+xml");
        assert_eq!(mime_from_extension("module.wasm"), "application/wasm");
    }

    #[test]
    fn test_mime_from_extension_no_extension() {
        assert_eq!(mime_from_extension("Makefile"), "application/octet-stream");
    }

    #[test]
    fn test_mime_from_extension_empty_string() {
        assert_eq!(mime_from_extension(""), "application/octet-stream");
    }

    #[test]
    fn test_mime_from_extension_multiple_dots() {
        assert_eq!(mime_from_extension("archive.tar.gz"), "application/gzip");
        assert_eq!(mime_from_extension("file.config.json"), "application/json");
    }

    #[test]
    fn test_artifact_storage_path() {
        let path = artifact_storage_path("/data", "owner1", "repo1", "art123");
        assert_eq!(path, PathBuf::from("/data/artifacts/owner1/repo1/art123"));
    }

    #[test]
    fn test_artifact_storage_path_nested() {
        let path = artifact_storage_path("/storage/root", "org", "project", "id-42");
        assert_eq!(
            path,
            PathBuf::from("/storage/root/artifacts/org/project/id-42")
        );
    }

    #[test]
    fn test_artifact_storage_path_relative() {
        let path = artifact_storage_path(".", "u", "r", "a");
        assert_eq!(path, PathBuf::from("./artifacts/u/r/a"));
    }

    #[test]
    fn test_generate_presigned_url() {
        let url = generate_presigned_url("https://storage.example.com", "file.zip", 3600);
        assert_eq!(url, "https://storage.example.com/file.zip?expires=3600");
    }

    #[test]
    fn test_generate_presigned_url_existing_query() {
        let url = generate_presigned_url("https://storage.example.com?token=abc", "file.zip", 7200);
        assert_eq!(
            url,
            "https://storage.example.com/file.zip?token=abc&expires=7200"
        );
    }

    #[test]
    fn test_validate_presigned_url() {
        let url = "https://storage.example.com/file.zip?expires=3600";
        assert_eq!(validate_presigned_url(url), Ok(3600));
    }

    #[test]
    fn test_validate_presigned_url_missing_expires() {
        let url = "https://storage.example.com/file.zip";
        assert!(validate_presigned_url(url).is_err());
    }

    #[test]
    fn test_validate_presigned_url_invalid_expires() {
        let url = "https://storage.example.com/file.zip?expires=notanumber";
        assert!(validate_presigned_url(url).is_err());
    }

    #[test]
    fn test_artifact_download_query() {
        let q: ArtifactDownloadQuery = serde_json::from_str(r#"{"token":"abc123"}"#).unwrap();
        assert_eq!(q.token, Some("abc123".into()));
    }

    #[test]
    fn test_artifact_download_query_no_token() {
        let q: ArtifactDownloadQuery = serde_json::from_str(r#"{}"#).unwrap();
        assert!(q.token.is_none());
    }
}
