#![forbid(unsafe_code)]

use std::collections::HashMap;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct FileAttributes {
    pub size: u64,
    pub mode: u32,
    pub is_dir: bool,
    pub modified: u64,
    pub accessed: u64,
}

impl Default for FileAttributes {
    fn default() -> Self {
        Self {
            size: 0,
            mode: 0o644,
            is_dir: false,
            modified: 0,
            accessed: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FuseResult {
    Ok,
    Error(i32),
    Data(Vec<u8>),
    Entry(FileEntry),
    Entries(Vec<FileEntry>),
    Attributes(FileAttributes),
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub inode: u64,
    pub attr: FileAttributes,
}

#[derive(Debug)]
pub struct FuseOperation {
    inodes: HashMap<String, u64>,
    inode_counter: u64,
    files: HashMap<u64, Vec<u8>>,
    dirs: HashMap<u64, Vec<String>>,
    attributes: HashMap<u64, FileAttributes>,
}

impl FuseOperation {
    pub fn new() -> Self {
        Self {
            inodes: HashMap::new(),
            inode_counter: 1,
            files: HashMap::new(),
            dirs: HashMap::new(),
            attributes: HashMap::new(),
        }
    }

    pub fn lookup(&mut self, parent: u64, name: &str) -> Result<FuseResult, i32> {
        let key = format!("{parent}:{name}");
        if let Some(&inode) = self.inodes.get(&key) {
            let attr = self.attributes.get(&inode).cloned().unwrap_or_default();
            return Ok(FuseResult::Entry(FileEntry {
                name: name.into(),
                inode,
                attr,
            }));
        }
        Err(libc::ENOENT)
    }

    pub fn getattr(&self, inode: u64) -> Result<FuseResult, i32> {
        let attr = self.attributes.get(&inode).cloned().unwrap_or_default();
        Ok(FuseResult::Attributes(attr))
    }

    pub fn read(&self, inode: u64, offset: usize, size: usize) -> Result<FuseResult, i32> {
        let data = self.files.get(&inode).ok_or(libc::ENOENT)?;
        let end = (offset + size).min(data.len());
        if offset >= data.len() {
            return Ok(FuseResult::Data(Vec::new()));
        }
        Ok(FuseResult::Data(data[offset..end].to_vec()))
    }

    pub fn readdir(&self, inode: u64) -> Result<FuseResult, i32> {
        let entries = self.dirs.get(&inode).ok_or(libc::ENOTDIR)?;
        let mut results = vec![
            FileEntry {
                name: ".".into(),
                inode,
                attr: FileAttributes::default(),
            },
            FileEntry {
                name: "..".into(),
                inode: 1,
                attr: FileAttributes::default(),
            },
        ];
        for name in entries {
            let key = format!("{inode}:{name}");
            let child_inode = self.inodes.get(&key).copied().unwrap_or(0);
            let attr = self
                .attributes
                .get(&child_inode)
                .cloned()
                .unwrap_or_default();
            results.push(FileEntry {
                name: name.clone(),
                inode: child_inode,
                attr,
            });
        }
        Ok(FuseResult::Entries(results))
    }

    pub fn create_file(
        &mut self,
        parent: u64,
        name: &str,
        data: Vec<u8>,
        attr: FileAttributes,
    ) -> u64 {
        let inode = self.next_inode();
        let key = format!("{parent}:{name}");
        self.inodes.insert(key, inode);
        self.files.insert(inode, data);
        let attr = FileAttributes {
            size: attr.size,
            ..attr
        };
        self.attributes.insert(inode, attr);
        self.dirs.entry(parent).or_default().push(name.to_string());
        debug!(inode = inode, name = %name, "created file");
        inode
    }

    pub fn create_dir(&mut self, parent: u64, name: &str) -> u64 {
        let inode = self.next_inode();
        let key = format!("{parent}:{name}");
        self.inodes.insert(key, inode);
        self.dirs.insert(inode, Vec::new());
        self.attributes.insert(
            inode,
            FileAttributes {
                is_dir: true,
                mode: 0o755,
                ..Default::default()
            },
        );
        if let Some(children) = self.dirs.get_mut(&parent) {
            children.push(name.to_string());
        }
        debug!(inode = inode, name = %name, "created directory");
        inode
    }

    pub fn unlink(&mut self, parent: u64, name: &str) -> Result<FuseResult, i32> {
        let key = format!("{parent}:{name}");
        let inode = self.inodes.remove(&key).ok_or(libc::ENOENT)?;
        self.files.remove(&inode);
        self.attributes.remove(&inode);
        if let Some(children) = self.dirs.get_mut(&parent) {
            children.retain(|c| c != name);
        }
        debug!(inode = inode, name = %name, "unlinked file");
        Ok(FuseResult::Ok)
    }

    fn next_inode(&mut self) -> u64 {
        let id = self.inode_counter;
        self.inode_counter += 1;
        id
    }
}

impl Default for FuseOperation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_lookup() {
        let mut fs = FuseOperation::new();
        let root_inode = 1;
        let inode = fs.create_file(
            root_inode,
            "test.txt",
            b"hello".to_vec(),
            FileAttributes::default(),
        );
        let result = fs.lookup(root_inode, "test.txt").unwrap();
        match result {
            FuseResult::Entry(e) => assert_eq!(e.inode, inode),
            _ => panic!("expected Entry"),
        }
    }

    #[test]
    fn test_lookup_missing() {
        let mut fs = FuseOperation::new();
        assert_eq!(fs.lookup(1, "missing").unwrap_err(), libc::ENOENT);
    }

    #[test]
    fn test_read_file() {
        let mut fs = FuseOperation::new();
        let data = b"hello world".to_vec();
        let inode = fs.create_file(1, "test.txt", data.clone(), FileAttributes::default());
        let result = fs.read(inode, 0, 5).unwrap();
        match result {
            FuseResult::Data(d) => assert_eq!(d, b"hello".to_vec()),
            _ => panic!("expected Data"),
        }
        let result = fs.read(inode, 6, 5).unwrap();
        match result {
            FuseResult::Data(d) => assert_eq!(d, b"world".to_vec()),
            _ => panic!("expected Data"),
        }
    }

    #[test]
    fn test_read_past_end() {
        let mut fs = FuseOperation::new();
        let inode = fs.create_file(1, "small.txt", b"hi".to_vec(), FileAttributes::default());
        let result = fs.read(inode, 100, 10).unwrap();
        match result {
            FuseResult::Data(d) => assert!(d.is_empty()),
            _ => panic!("expected empty Data"),
        }
    }

    #[test]
    fn test_readdir() {
        let mut fs = FuseOperation::new();
        fs.create_file(1, "a.txt", vec![], FileAttributes::default());
        fs.create_file(1, "b.txt", vec![], FileAttributes::default());
        let result = fs.readdir(1).unwrap();
        match result {
            FuseResult::Entries(entries) => {
                assert_eq!(entries.len(), 4); // ., .., a.txt, b.txt
            }
            _ => panic!("expected Entries"),
        }
    }

    #[test]
    fn test_unlink() {
        let mut fs = FuseOperation::new();
        let root_inode = 1;
        fs.create_file(
            root_inode,
            "del.txt",
            b"remove me".to_vec(),
            FileAttributes::default(),
        );
        assert!(fs.unlink(root_inode, "del.txt").is_ok());
        assert!(fs.lookup(root_inode, "del.txt").is_err());
    }

    #[test]
    fn test_getattr() {
        let mut fs = FuseOperation::new();
        let inode = fs.create_file(
            1,
            "test.txt",
            b"data".to_vec(),
            FileAttributes {
                size: 4,
                mode: 0o644,
                ..Default::default()
            },
        );
        let result = fs.getattr(inode).unwrap();
        match result {
            FuseResult::Attributes(attr) => {
                assert_eq!(attr.size, 4);
                assert_eq!(attr.mode, 0o644);
                assert!(!attr.is_dir);
            }
            _ => panic!("expected Attributes"),
        }
    }

    #[test]
    fn test_create_dir_and_readdir() {
        let mut fs = FuseOperation::new();
        let dir_inode = fs.create_dir(1, "subdir");
        assert!(fs.dirs.contains_key(&dir_inode));
    }
}
