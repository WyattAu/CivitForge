#![forbid(unsafe_code)]

use crate::error::{CoreError, Result};
use std::path::PathBuf;
use tracing::debug;

pub fn info_refs(repo_path: &PathBuf, service: &str) -> Result<Vec<u8>> {
    let repo = gix::open(repo_path).map_err(|e| CoreError::Git(e.to_string()))?;

    let mut output = Vec::new();
    let service_line = format!("# service=git-{service}\n");
    let pkt_header = pkt_line(&service_line);
    output.extend_from_slice(&pkt_header);
    output.extend_from_slice(b"0000");

    let refs_iter = repo
        .references()
        .map_err(|e| CoreError::Git(e.to_string()))?;
    let all_refs = refs_iter.all().map_err(|e| CoreError::Git(e.to_string()))?;
    let mut first = true;
    for r in all_refs {
        let r = r.map_err(|e| CoreError::Git(e.to_string()))?;
        let id = match r.try_id() {
            Some(id) => id,
            None => continue,
        };
        let name = r.name().shorten().to_string();
        let target = id.to_hex().to_string();
        let capabilities = if first {
            first = false;
            format!(
                "{target}\t{name}\0 multi_ack thin-pack side-band side-band-64k ofs-delta shallow deepen-since deepen-not deepen-relative no-progress include-tag multi_ack_detailed no-done symref=HEAD:refs/heads/main agent=git/civitforge\n"
            )
        } else {
            format!("{target}\t{name}\n")
        };
        output.extend_from_slice(&pkt_line(&capabilities));
    }

    output.extend_from_slice(b"0000");
    debug!(service = %service, refs = %output.len(), "generated info/refs");
    Ok(output)
}

pub fn upload_pack(repo_path: &PathBuf, _input: &[u8]) -> Result<Vec<u8>> {
    info_refs(repo_path, "upload-pack")
}

pub fn receive_pack(repo_path: &PathBuf) -> Result<()> {
    gix::open(repo_path).map_err(|e| CoreError::Git(e.to_string()))?;
    debug!(path = %repo_path.display(), "receive-pack initiated");
    Ok(())
}

pub fn list_refs(repo_path: &PathBuf) -> Result<Vec<(String, String)>> {
    let repo = gix::open(repo_path).map_err(|e| CoreError::Git(e.to_string()))?;
    let mut refs = Vec::new();

    let iter = repo
        .references()
        .map_err(|e| CoreError::Git(e.to_string()))?;
    let all_refs = iter.all().map_err(|e| CoreError::Git(e.to_string()))?;
    for r in all_refs {
        let r = r.map_err(|e| CoreError::Git(e.to_string()))?;
        let name = r.name().shorten().to_string();
        let target = match r.try_id() {
            Some(id) => id.to_hex().to_string(),
            None => continue,
        };
        refs.push((target, name));
    }

    Ok(refs)
}

fn pkt_line(data: &str) -> Vec<u8> {
    let len = data.len() + 4;
    let mut out = format!("{len:04x}").into_bytes();
    out.extend_from_slice(data.as_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_refs_on_bare_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().to_path_buf();
        gix::init_bare(&repo_path).unwrap();

        let result = info_refs(&repo_path, "upload-pack");
        assert!(result.is_ok());
        let data = result.unwrap();
        let text = String::from_utf8_lossy(&data);
        assert!(text.contains("# service=git-upload-pack"));
        assert!(text.contains("0000"));
    }

    #[test]
    fn test_info_refs_receive_pack_service() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().to_path_buf();
        gix::init_bare(&repo_path).unwrap();

        let result = info_refs(&repo_path, "receive-pack");
        assert!(result.is_ok());
        let data = result.unwrap();
        let text = String::from_utf8_lossy(&data);
        assert!(text.contains("# service=git-receive-pack"));
    }

    #[test]
    fn test_info_refs_invalid_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().to_path_buf().join("nonexistent");
        let result = info_refs(&repo_path, "upload-pack");
        assert!(result.is_err());
    }

    #[test]
    fn test_upload_pack_on_bare_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().to_path_buf();
        gix::init_bare(&repo_path).unwrap();

        let result = upload_pack(&repo_path, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_receive_pack_on_bare_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().to_path_buf();
        gix::init_bare(&repo_path).unwrap();

        let result = receive_pack(&repo_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_receive_pack_invalid_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().to_path_buf().join("nonexistent");
        let result = receive_pack(&repo_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_refs_on_bare_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().to_path_buf();
        gix::init_bare(&repo_path).unwrap();

        let result = list_refs(&repo_path);
        assert!(result.is_ok());
        let refs = result.unwrap();
        assert!(refs.is_empty());
    }

    #[test]
    fn test_list_refs_invalid_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().to_path_buf().join("nonexistent");
        let result = list_refs(&repo_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_pkt_line_format() {
        let line = pkt_line("test\n");
        let text = String::from_utf8_lossy(&line);
        assert!(text.starts_with("0009"));
        assert!(text.contains("test\n"));
    }

    #[test]
    fn test_pkt_line_empty() {
        let line = pkt_line("");
        assert_eq!(line.len(), 4);
        assert_eq!(&line, b"0004");
    }

    #[test]
    fn test_pkt_line_long() {
        let data = "a".repeat(1000);
        let line = pkt_line(&data);
        let text = String::from_utf8_lossy(&line);
        assert!(text.starts_with("03ec"));
    }
}
