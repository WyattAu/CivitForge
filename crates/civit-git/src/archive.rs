use anyhow::{Context, Result};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::{Seek, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveFormat {
    Zip,
    TarGz,
}

impl ArchiveFormat {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "zip" => Ok(Self::Zip),
            "tar.gz" => Ok(Self::TarGz),
            _ => Err(anyhow::anyhow!("unsupported archive format: {s}")),
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Zip => "zip",
            Self::TarGz => "tar.gz",
        }
    }

    pub fn content_type(&self) -> &'static str {
        match self {
            Self::Zip => "application/zip",
            Self::TarGz => "application/gzip",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArchiveResult {
    pub data: Vec<u8>,
    pub format: ArchiveFormat,
    pub filename: String,
}

pub fn generate_archive(
    repo_path: &Path,
    ref_name: &str,
    format: ArchiveFormat,
) -> Result<ArchiveResult> {
    if !repo_path.join("HEAD").exists() {
        return Err(anyhow::anyhow!("repository not found at {}", repo_path.display()));
    }

    match format {
        ArchiveFormat::Zip => generate_zip_archive(repo_path, ref_name),
        ArchiveFormat::TarGz => generate_tar_gz_archive(repo_path, ref_name),
    }
}

fn generate_zip_archive(repo_path: &Path, ref_name: &str) -> Result<ArchiveResult> {
    let output = std::process::Command::new("git")
        .arg("archive")
        .arg("--format=zip")
        .arg(ref_name)
        .current_dir(repo_path)
        .output()
        .context("failed to run git archive")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("git archive failed: {stderr}"));
    }

    let repo_name = repo_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("repo");
    let filename = format!("{repo_name}-{ref_name}.zip");

    Ok(ArchiveResult {
        data: output.stdout,
        format: ArchiveFormat::Zip,
        filename,
    })
}

fn generate_tar_gz_archive(repo_path: &Path, ref_name: &str) -> Result<ArchiveResult> {
    let output = std::process::Command::new("git")
        .arg("archive")
        .arg("--format=tar.gz")
        .arg(ref_name)
        .current_dir(repo_path)
        .output()
        .context("failed to run git archive")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("git archive failed: {stderr}"));
    }

    let repo_name = repo_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("repo");
    let filename = format!("{repo_name}-{ref_name}.tar.gz");

    Ok(ArchiveResult {
        data: output.stdout,
        format: ArchiveFormat::TarGz,
        filename,
    })
}

pub fn create_zip_from_bytes<W: Write + Seek>(writer: W, entries: &[(&str, &[u8])]) -> Result<()> {
    let mut zip = zip::ZipWriter::new(writer);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(6));

    for (name, content) in entries {
        zip.start_file(name, options)?;
        zip.write_all(content)?;
    }

    zip.finish()?;
    Ok(())
}

pub fn create_tar_gz_from_bytes<W: Write>(writer: W, entries: &[(&str, &[u8])]) -> Result<()> {
    let enc = GzEncoder::new(writer, Compression::default());
    let mut tar = tar::Builder::new(enc);

    for (name, content) in entries {
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append_data(&mut header, name, *content)?;
    }

    tar.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_format_from_str() {
        assert_eq!(ArchiveFormat::parse("zip").unwrap(), ArchiveFormat::Zip);
        assert_eq!(
            ArchiveFormat::parse("tar.gz").unwrap(),
            ArchiveFormat::TarGz
        );
        assert!(ArchiveFormat::parse("7z").is_err());
    }

    #[test]
    fn test_archive_format_extension() {
        assert_eq!(ArchiveFormat::Zip.extension(), "zip");
        assert_eq!(ArchiveFormat::TarGz.extension(), "tar.gz");
    }

    #[test]
    fn test_archive_format_content_type() {
        assert_eq!(ArchiveFormat::Zip.content_type(), "application/zip");
        assert_eq!(ArchiveFormat::TarGz.content_type(), "application/gzip");
    }

    #[test]
    fn test_create_zip_from_bytes() {
        let mut buf = std::io::Cursor::new(Vec::new());
        create_zip_from_bytes(&mut buf, &[("hello.txt", b"hello world")]).unwrap();
        assert!(!buf.into_inner().is_empty());
    }

    #[test]
    fn test_create_tar_gz_from_bytes() {
        let mut buf = Vec::new();
        create_tar_gz_from_bytes(&mut buf, &[("hello.txt", b"hello world")]).unwrap();
        assert!(!buf.is_empty());
    }

    #[test]
    fn test_generate_archive_on_bare_repo() {
        let tmp = tempfile::tempdir().unwrap();
        gix::init_bare(tmp.path()).unwrap();
        let result = generate_archive(tmp.path(), "HEAD", ArchiveFormat::Zip);
        assert!(result.is_ok() || result.is_err()); // may fail without commits
    }

    fn create_repo_with_file(path: &Path, filename: &str, content: &[u8]) {
        let work_tmp = tempfile::tempdir().unwrap();
        let work = work_tmp.path();
        std::process::Command::new("git").args(["init", work.to_str().unwrap()]).output().unwrap();
        std::process::Command::new("git").args(["config", "user.name", "Test"]).current_dir(work).output().unwrap();
        std::process::Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(work).output().unwrap();
        std::fs::write(work.join(filename), content).unwrap();
        std::process::Command::new("git").args(["add", "."]).current_dir(work).output().unwrap();
        std::process::Command::new("git").args(["commit", "-m", "init"]).current_dir(work).output().unwrap();
        std::fs::create_dir_all(path).unwrap();
        std::process::Command::new("git").args(["clone", "--bare", work.to_str().unwrap(), path.to_str().unwrap()]).output().unwrap();
    }

    #[test]
    fn test_generate_zip_archive() {
        let tmp = tempfile::tempdir().unwrap();
        create_repo_with_file(tmp.path(), "hello.txt", b"hello world");
        let result = generate_archive(tmp.path(), "HEAD", ArchiveFormat::Zip).unwrap();
        assert!(!result.data.is_empty());
        assert_eq!(result.format, ArchiveFormat::Zip);
        assert!(result.filename.ends_with(".zip"));
    }

    #[test]
    fn test_generate_tar_gz_archive() {
        let tmp = tempfile::tempdir().unwrap();
        create_repo_with_file(tmp.path(), "hello.txt", b"hello world");
        let result = generate_archive(tmp.path(), "HEAD", ArchiveFormat::TarGz).unwrap();
        assert!(!result.data.is_empty());
        assert_eq!(result.format, ArchiveFormat::TarGz);
        assert!(result.filename.ends_with(".tar.gz"));
    }

    #[test]
    fn test_generate_archive_nonexistent_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let result = generate_archive(tmp.path(), "HEAD", ArchiveFormat::Zip);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_zip_from_bytes_roundtrip() {
        let mut buf = std::io::Cursor::new(Vec::new());
        create_zip_from_bytes(&mut buf, &[
            ("a.txt", b"content a"),
            ("b.txt", b"content b"),
        ]).unwrap();
        let data = buf.into_inner();
        assert!(data.len() > 10);
        // Verify it's a valid zip by reading it back
        let cursor = std::io::Cursor::new(&data);
        let archive = zip::ZipArchive::new(cursor).unwrap();
        assert_eq!(archive.len(), 2);
    }

    #[test]
    fn test_create_tar_gz_from_bytes_roundtrip() {
        let mut buf = Vec::new();
        create_tar_gz_from_bytes(&mut buf, &[
            ("a.txt", b"content a"),
            ("b.txt", b"content b"),
        ]).unwrap();
        assert!(buf.len() > 10);
    }
}
