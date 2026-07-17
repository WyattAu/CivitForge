#![forbid(unsafe_code)]

use crate::error::{CoreError, Result};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tracing::{debug, warn};

pub fn info_refs(
    repo_path: &PathBuf,
    service: &str,
    git_protocol: Option<&str>,
) -> Result<Vec<u8>> {
    let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "git".to_string());

    let mut cmd = Command::new(&git_bin);
    cmd.arg(service)
        .arg("--stateless-rpc")
        .arg("--advertise-refs")
        .arg(repo_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Forward the Git-Protocol header so git binary handles v2 negotiation
    if let Some(proto) = git_protocol {
        cmd.env("GIT_PROTOCOL", proto);
    }

    let child = cmd
        .spawn()
        .map_err(|e| CoreError::Git(format!("failed to spawn git {service}: {e}")))?;

    let output = child
        .wait_with_output()
        .map_err(|e| CoreError::Git(format!("failed to read git {service} output: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!(status = %output.status, stderr = %stderr, "git info-refs failed");
        return Err(CoreError::Git(format!(
            "git {service} exited with {}: {stderr}",
            output.status
        )));
    }

    let refs_data = output.stdout;

    // For HTTP v1, wrap in service line + flush. For v2, return raw output
    // (git's --advertise-refs with GIT_PROTOCOL=version=2 already produces
    // the correct v2 response).
    let is_v2 = git_protocol.is_some_and(|p| p.contains("version=2"));

    if is_v2 {
        debug!(service = %service, response_len = refs_data.len(), "generated v2 info/refs");
        Ok(refs_data)
    } else {
        let mut response = Vec::new();
        let service_line = format!("# service=git-{service}\n");
        response.extend_from_slice(&pkt_line(&service_line));
        response.extend_from_slice(b"0000");
        response.extend_from_slice(&refs_data);

        debug!(service = %service, response_len = response.len(), "generated v1 info/refs");
        Ok(response)
    }
}

pub fn upload_pack(
    repo_path: &PathBuf,
    input: &[u8],
    git_protocol: Option<&str>,
) -> Result<Vec<u8>> {
    let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "git".to_string());

    let mut cmd = Command::new(&git_bin);
    cmd.arg("upload-pack")
        .arg("--stateless-rpc")
        .arg(repo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(proto) = git_protocol {
        cmd.env("GIT_PROTOCOL", proto);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| CoreError::Git(format!("failed to spawn git upload-pack: {e}")))?;

    {
        let stdin = child.stdin.as_mut().expect("stdin piped");
        stdin
            .write_all(input)
            .map_err(|e| CoreError::Git(format!("failed to write to git upload-pack: {e}")))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| CoreError::Git(format!("failed to read git upload-pack output: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!(status = %output.status, stderr = %stderr, "git upload-pack failed");
        return Err(CoreError::Git(format!(
            "git upload-pack exited with {}: {stderr}",
            output.status
        )));
    }

    debug!(response_len = output.stdout.len(), "upload-pack complete");
    Ok(output.stdout)
}

pub fn receive_pack(
    repo_path: &PathBuf,
    input: &[u8],
    git_protocol: Option<&str>,
) -> Result<Vec<u8>> {
    let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "git".to_string());

    let mut cmd = Command::new(&git_bin);
    cmd.arg("receive-pack")
        .arg("--stateless-rpc")
        .arg(repo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(proto) = git_protocol {
        cmd.env("GIT_PROTOCOL", proto);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| CoreError::Git(format!("failed to spawn git receive-pack: {e}")))?;

    {
        let stdin = child.stdin.as_mut().expect("stdin piped");
        stdin
            .write_all(input)
            .map_err(|e| CoreError::Git(format!("failed to write to git receive-pack: {e}")))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| CoreError::Git(format!("failed to read git receive-pack output: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!(status = %output.status, stderr = %stderr, "git receive-pack failed");
        return Err(CoreError::Git(format!(
            "git receive-pack exited with {}: {stderr}",
            output.status
        )));
    }

    debug!(response_len = output.stdout.len(), "receive-pack complete");
    Ok(output.stdout)
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

pub fn pkt_line(data: &str) -> Vec<u8> {
    let len = data.len() + 4;
    let mut out = format!("{len:04x}").into_bytes();
    out.extend_from_slice(data.as_bytes());
    out
}

#[cfg(test)]
fn parse_pkt_lines(data: &[u8]) -> Vec<Vec<u8>> {
    let mut lines = Vec::new();
    let mut pos = 0;
    while pos + 4 <= data.len() {
        let hex_str = std::str::from_utf8(&data[pos..pos + 4]).unwrap_or("");
        let len = u16::from_str_radix(hex_str, 16).unwrap_or(0) as usize;
        if len == 0 {
            pos += 4;
            continue;
        }
        if len < 4 {
            break;
        }
        if pos + len > data.len() {
            break;
        }
        lines.push(data[pos + 4..pos + len].to_vec());
        pos += len;
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_refs_on_bare_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().to_path_buf();
        gix::init_bare(&repo_path).unwrap();

        let result = info_refs(&repo_path, "upload-pack", None);
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

        let result = info_refs(&repo_path, "receive-pack", None);
        assert!(result.is_ok());
        let data = result.unwrap();
        let text = String::from_utf8_lossy(&data);
        assert!(text.contains("# service=git-receive-pack"));
    }

    #[test]
    fn test_info_refs_invalid_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().to_path_buf().join("nonexistent");
        let result = info_refs(&repo_path, "upload-pack", None);
        assert!(result.is_err(), "nonexistent repo should error");
    }

    #[test]
    fn test_info_refs_v2_protocol() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().to_path_buf();
        gix::init_bare(&repo_path).unwrap();

        let result = info_refs(&repo_path, "upload-pack", Some("version=2"));
        assert!(result.is_ok());
        let data = result.unwrap();
        // v2 response should NOT have the v1 service line wrapper
        let text = String::from_utf8_lossy(&data);
        assert!(
            !text.contains("# service=git-upload-pack"),
            "v2 should not have v1 wrapper"
        );
    }

    #[test]
    fn test_upload_pack_invalid_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().to_path_buf().join("nonexistent");
        let result = upload_pack(&repo_path, &[], None);
        assert!(result.is_err(), "nonexistent repo should error");
    }

    #[test]
    fn test_receive_pack_invalid_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().to_path_buf().join("nonexistent");
        let result = receive_pack(&repo_path, &[], None);
        assert!(result.is_err(), "nonexistent repo should error");
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

    #[test]
    fn test_parse_pkt_lines_single() {
        let input = make_pkt_line("want abc123\n");
        let lines = parse_pkt_lines(&input);
        assert_eq!(lines.len(), 1);
        assert_eq!(String::from_utf8_lossy(&lines[0]), "want abc123\n");
    }

    #[test]
    fn test_parse_pkt_lines_with_flush() {
        let mut input = make_pkt_line("want abc123\n");
        input.extend_from_slice(b"0000");
        input.extend_from_slice(&make_pkt_line("have def456\n"));
        let lines = parse_pkt_lines(&input);
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_parse_pkt_lines_flush_only() {
        let input = b"0000".to_vec();
        let lines = parse_pkt_lines(&input);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_parse_pkt_lines_empty() {
        let lines = parse_pkt_lines(&[]);
        assert!(lines.is_empty());
    }

    fn make_pkt_line(data: &str) -> Vec<u8> {
        pkt_line(data)
    }
}
