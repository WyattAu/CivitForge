#![forbid(unsafe_code)]

use crate::error::{CoreError, Result};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use std::collections::{HashSet, VecDeque};
use std::io::Write;
use std::path::PathBuf;
use tracing::debug;

use gix::prelude::Find;

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

pub fn upload_pack(repo_path: &PathBuf, input: &[u8]) -> Result<Vec<u8>> {
    let repo = gix::open(repo_path).map_err(|e| CoreError::Git(e.to_string()))?;
    let wants = parse_wants(input)?;

    if wants.is_empty() {
        debug!("upload-pack: no wants received");
        return Ok(Vec::new());
    }

    let pack_data = build_packfile(&repo, &wants)?;

    let mut response = Vec::new();
    response.extend_from_slice(&pkt_line("NAK\n"));
    response.extend_from_slice(b"0000");
    response.extend_from_slice(&pack_data);

    debug!(
        wants = wants.len(),
        pack_bytes = pack_data.len(),
        "upload-pack complete"
    );
    Ok(response)
}

pub fn receive_pack(repo_path: &PathBuf, input: &[u8]) -> Result<Vec<u8>> {
    let _repo = gix::open(repo_path).map_err(|e| CoreError::Git(e.to_string()))?;
    let updates = parse_ref_updates(input)?;

    let mut result = String::new();
    for update in &updates {
        let ctx = crate::git::PushContext {
            repo_path: repo_path.clone(),
            old_sha: update.old_sha.clone(),
            new_sha: update.new_sha.clone(),
            ref_name: update.ref_name.clone(),
            pusher: "anonymous".into(),
        };

        let hook_result = crate::git::HookRunner::new().run_hooks(&ctx)?;
        if hook_result.accepted {
            result.push_str(&format!("ok {}\n", update.ref_name));
        } else {
            result.push_str(&format!("ng {} {}\n", update.ref_name, hook_result.message));
        }
    }

    result.push('\n');
    debug!(updates = updates.len(), "receive-pack processed");
    Ok(result.into_bytes())
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

fn parse_wants(input: &[u8]) -> Result<Vec<gix::ObjectId>> {
    let lines = parse_pkt_lines(input);
    let mut wants = Vec::new();
    for line in &lines {
        let text = String::from_utf8_lossy(line);
        if let Some(rest) = text.strip_prefix("want ") {
            let hex: String = rest.chars().take(40).collect();
            if hex.len() == 40 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
                let oid = gix::ObjectId::from_hex(hex.as_bytes())
                    .map_err(|e| CoreError::Git(format!("invalid want SHA: {e}")))?;
                wants.push(oid);
            }
        }
    }
    Ok(wants)
}

struct RefUpdate {
    old_sha: String,
    new_sha: String,
    ref_name: String,
}

fn parse_ref_updates(input: &[u8]) -> Result<Vec<RefUpdate>> {
    let lines = parse_pkt_lines(input);
    let mut updates = Vec::new();
    for line in &lines {
        let text = String::from_utf8_lossy(line);
        let parts: Vec<&str> = text.splitn(3, ' ').collect();
        if parts.len() >= 3 {
            let ref_name = parts[2]
                .split('\0')
                .next()
                .unwrap_or(parts[2])
                .trim_end()
                .to_string();
            if ref_name.starts_with("refs/") || ref_name == "HEAD" {
                updates.push(RefUpdate {
                    old_sha: parts[0].to_string(),
                    new_sha: parts[1].to_string(),
                    ref_name,
                });
            }
        }
    }
    Ok(updates)
}

fn build_packfile(repo: &gix::Repository, wants: &[gix::ObjectId]) -> Result<Vec<u8>> {
    let mut objects: Vec<(gix::ObjectId, gix::object::Kind, Vec<u8>)> = Vec::new();
    let mut visited: HashSet<gix::ObjectId> = HashSet::new();
    let mut queue: VecDeque<gix::ObjectId> = wants.iter().copied().collect();

    while let Some(oid) = queue.pop_front() {
        if !visited.insert(oid) {
            continue;
        }

        let (kind, data) = find_object(repo, &oid)?;

        match kind {
            gix::object::Kind::Commit => {
                queue_tree_and_parents(&data, &mut queue)?;
            }
            gix::object::Kind::Tree => {
                queue_tree_entries(&data, &mut queue);
            }
            gix::object::Kind::Blob | gix::object::Kind::Tag => {}
        }

        objects.push((oid, kind, data));
    }

    let mut pack = Vec::new();
    pack.extend_from_slice(b"PACK");
    pack.extend_from_slice(&2u32.to_be_bytes());
    pack.extend_from_slice(&(objects.len() as u32).to_be_bytes());

    for (_oid, kind, data) in &objects {
        pack.extend_from_slice(&encode_pack_entry(*kind, data));
    }

    let mut hasher = Sha1::new();
    hasher.update(&pack);
    pack.extend_from_slice(&hasher.finalize());

    Ok(pack)
}

fn find_object(
    repo: &gix::Repository,
    oid: &gix::ObjectId,
) -> Result<(gix::object::Kind, Vec<u8>)> {
    let id_hex = oid.to_hex().to_string();
    let mut buf = Vec::new();
    let obj = repo
        .objects
        .try_find(oid, &mut buf)
        .map_err(|e| CoreError::Git(format!("object lookup: {e}")))?
        .ok_or_else(|| CoreError::Git(format!("object not found: {id_hex}")))?;
    Ok((obj.kind, obj.data.to_vec()))
}

fn queue_tree_and_parents(commit_data: &[u8], queue: &mut VecDeque<gix::ObjectId>) -> Result<()> {
    let text = String::from_utf8_lossy(commit_data);
    for line in text.lines() {
        if let Some(hex) = line.strip_prefix("tree ") {
            let hex: String = hex.chars().take(40).collect();
            if hex.len() == 40 {
                let oid = gix::ObjectId::from_hex(hex.as_bytes())
                    .map_err(|e| CoreError::Git(format!("invalid tree SHA: {e}")))?;
                queue.push_back(oid);
            }
        } else if let Some(hex) = line.strip_prefix("parent ") {
            let hex: String = hex.chars().take(40).collect();
            if hex.len() == 40 {
                let oid = gix::ObjectId::from_hex(hex.as_bytes())
                    .map_err(|e| CoreError::Git(format!("invalid parent SHA: {e}")))?;
                queue.push_back(oid);
            }
        }
    }
    Ok(())
}

fn queue_tree_entries(tree_data: &[u8], queue: &mut VecDeque<gix::ObjectId>) {
    let mut pos = 0;
    while pos < tree_data.len() {
        if let Some(null_idx) = tree_data[pos..].iter().position(|&b| b == 0) {
            let sha_start = pos + null_idx + 1;
            if sha_start + 20 > tree_data.len() {
                break;
            }
            let oid = gix::ObjectId::from_bytes_or_panic(&tree_data[sha_start..sha_start + 20]);
            queue.push_back(oid);
            pos = sha_start + 20;
        } else {
            break;
        }
    }
}

fn encode_pack_entry(kind: gix::object::Kind, data: &[u8]) -> Vec<u8> {
    let type_num: u8 = match kind {
        gix::object::Kind::Commit => 1,
        gix::object::Kind::Tree => 2,
        gix::object::Kind::Blob => 3,
        gix::object::Kind::Tag => 4,
    };

    let size = data.len();
    let mut header = encode_pack_header(type_num, size);

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(data).unwrap();
    let compressed = encoder.finish().unwrap();

    header.extend_from_slice(&compressed);
    header
}

fn encode_pack_header(obj_type: u8, mut size: usize) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut byte = (obj_type << 4) | ((size & 0x0F) as u8);
    size >>= 4;
    if size > 0 {
        byte |= 0x80;
    }
    bytes.push(byte);
    while size > 0 {
        let mut b = (size & 0x7F) as u8;
        size >>= 7;
        if size > 0 {
            b |= 0x80;
        }
        bytes.push(b);
    }
    bytes
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
        let data = result.unwrap();
        assert!(data.is_empty(), "empty wants should return empty response");
    }

    #[test]
    fn test_upload_pack_invalid_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().to_path_buf().join("nonexistent");
        let result = upload_pack(&repo_path, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_receive_pack_on_bare_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().to_path_buf();
        gix::init_bare(&repo_path).unwrap();

        let result = receive_pack(&repo_path, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_receive_pack_invalid_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().to_path_buf().join("nonexistent");
        let result = receive_pack(&repo_path, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_receive_pack_ref_updates() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().to_path_buf();
        gix::init_bare(&repo_path).unwrap();

        let old_sha = "a".repeat(40);
        let new_sha = "b".repeat(40);
        let pkt = make_pkt_line(&format!("{old_sha} {new_sha} refs/heads/main\n"));

        let result = receive_pack(&repo_path, &pkt);
        assert!(result.is_ok());
        let data = result.unwrap();
        let text = String::from_utf8_lossy(&data);
        assert!(text.contains("ok refs/heads/main"));
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

    #[test]
    fn test_parse_wants_single() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let input = make_pkt_line(&format!("want {sha}\n"));
        let wants = parse_wants(&input).unwrap();
        assert_eq!(wants.len(), 1);
        assert_eq!(wants[0].to_hex().to_string(), sha);
    }

    #[test]
    fn test_parse_wants_multiple() {
        let sha1 = "0123456789abcdef0123456789abcdef01234567";
        let sha2 = "abcdef0123456789abcdef0123456789abcdef01";
        let mut input = make_pkt_line(&format!("want {sha1}\n"));
        input.extend_from_slice(b"0000");
        input.extend_from_slice(&make_pkt_line(&format!("want {sha2}\n")));
        let wants = parse_wants(&input).unwrap();
        assert_eq!(wants.len(), 2);
    }

    #[test]
    fn test_parse_wants_with_capabilities() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let input = make_pkt_line(&format!("want {sha} multi_ack thin-pack\n"));
        let wants = parse_wants(&input).unwrap();
        assert_eq!(wants.len(), 1);
        assert_eq!(wants[0].to_hex().to_string(), sha);
    }

    #[test]
    fn test_parse_wants_empty() {
        let wants = parse_wants(&[]).unwrap();
        assert!(wants.is_empty());
    }

    #[test]
    fn test_parse_ref_updates_single() {
        let old = "a".repeat(40);
        let new = "b".repeat(40);
        let input = make_pkt_line(&format!("{old} {new} refs/heads/main\n"));
        let updates = parse_ref_updates(&input).unwrap();
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].ref_name, "refs/heads/main");
        assert_eq!(updates[0].old_sha, old);
        assert_eq!(updates[0].new_sha, new);
    }

    #[test]
    fn test_parse_ref_updates_with_capabilities() {
        let old = "0".repeat(40);
        let new = "1".repeat(40);
        let input = make_pkt_line(&format!("{old} {new} refs/heads/feature\0report-status\n"));
        let updates = parse_ref_updates(&input).unwrap();
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].ref_name, "refs/heads/feature");
    }

    #[test]
    fn test_parse_ref_updates_skips_invalid() {
        let input = make_pkt_line("not-a-valid-update\n");
        let updates = parse_ref_updates(&input).unwrap();
        assert!(updates.is_empty());
    }

    #[test]
    fn test_encode_pack_header_small() {
        let bytes = encode_pack_header(1, 10);
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], 0x1A);
    }

    #[test]
    fn test_encode_pack_header_two_bytes() {
        let bytes = encode_pack_header(3, 0x1F);
        assert_eq!(bytes.len(), 2);
        assert_eq!(bytes[0], 0xBF);
        assert_eq!(bytes[1], 0x01);
    }

    #[test]
    fn test_encode_pack_header_very_large() {
        let size: usize = 0x10000000;
        let bytes = encode_pack_header(1, size);
        assert!(bytes.len() > 2);
        let mut decoded_size: usize = (bytes[0] & 0x0F) as usize;
        decoded_size |= ((bytes[1] & 0x7F) as usize) << 4;
        decoded_size |= ((bytes[2] & 0x7F) as usize) << 11;
        decoded_size |= ((bytes[3] & 0x7F) as usize) << 18;
        decoded_size |= ((bytes[4] & 0x7F) as usize) << 25;
        assert_eq!(decoded_size, size);
    }

    #[test]
    fn test_encode_pack_entry_blob() {
        let data = b"hello world";
        let entry = encode_pack_entry(gix::object::Kind::Blob, data);
        assert!(!entry.is_empty());
        let first_byte = entry[0];
        let type_num = (first_byte >> 4) & 0x07;
        assert_eq!(type_num, 3);
    }

    #[test]
    fn test_encode_pack_entry_commit() {
        let data = b"tree deadbeef\nauthor someone\n";
        let entry = encode_pack_entry(gix::object::Kind::Commit, data);
        let first_byte = entry[0];
        let type_num = (first_byte >> 4) & 0x07;
        assert_eq!(type_num, 1);
    }

    #[test]
    fn test_encode_pack_entry_roundtrip() {
        let data = b"test blob content";
        let entry = encode_pack_entry(gix::object::Kind::Blob, data);
        let header_len = encode_pack_header(3, data.len()).len();
        let compressed = &entry[header_len..];

        let mut decoder = flate2::read::ZlibDecoder::new(compressed);
        let mut decompressed = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut decompressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_build_packfile_empty_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().to_path_buf();
        gix::init_bare(&repo_path).unwrap();
        let repo = gix::open(&repo_path).unwrap();

        let sha_hex = "0123456789abcdef0123456789abcdef01234567";
        let oid = gix::ObjectId::from_hex(sha_hex.as_bytes()).unwrap();

        let result = build_packfile(&repo, &[oid]);
        assert!(result.is_err(), "should fail for nonexistent object");
    }

    #[test]
    fn test_queue_tree_entries() {
        let tree_data = b"100644 file.txt\0\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13\x14";
        let mut queue = VecDeque::new();
        queue_tree_entries(tree_data, &mut queue);
        assert_eq!(queue.len(), 1);
        let sha_start = "100644 file.txt\0".len();
        let expected = gix::ObjectId::from_bytes_or_panic(&tree_data[sha_start..sha_start + 20]);
        assert_eq!(queue[0].as_bytes(), expected.as_bytes());
    }

    #[test]
    fn test_queue_tree_entries_multiple() {
        let mut tree_data = Vec::new();
        tree_data.extend_from_slice(b"100644 a.txt\0");
        tree_data.extend_from_slice(&[0xAA; 20]);
        tree_data.extend_from_slice(b"40000 dir\0");
        tree_data.extend_from_slice(&[0xBB; 20]);

        let mut queue = VecDeque::new();
        queue_tree_entries(&tree_data, &mut queue);
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn test_queue_tree_entries_empty() {
        let mut queue = VecDeque::new();
        queue_tree_entries(b"", &mut queue);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_queue_tree_and_parents() {
        let tree_sha = "abcdef0123456789abcdef0123456789abcdef01";
        let parent_sha = "0123456789abcdef0123456789abcdef01234567";
        let commit_data = format!(
            "tree {tree_sha}\nparent {parent_sha}\nauthor a <a@b> 0 +0000\ncommitter c <c@d> 0 +0000\n\nmessage\n"
        );

        let mut queue = VecDeque::new();
        queue_tree_and_parents(commit_data.as_bytes(), &mut queue).unwrap();
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn test_upload_pack_response_format() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().to_path_buf();
        gix::init_bare(&repo_path).unwrap();

        let sha_hex = "0123456789abcdef0123456789abcdef01234567";
        let input = make_pkt_line(&format!("want {sha_hex}\n"));
        let result = upload_pack(&repo_path, &input);
        assert!(result.is_err(), "nonexistent object should error");
    }

    fn make_pkt_line(data: &str) -> Vec<u8> {
        pkt_line(data)
    }
}
