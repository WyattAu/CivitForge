#![forbid(unsafe_code)]

use std::path::PathBuf;

use crate::error::CoreError;

#[derive(Debug, Clone)]
pub struct WikiRevision {
    pub sha: String,
    pub author: String,
    pub email: String,
    pub message: String,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct WikiPageEntry {
    pub slug: String,
    pub title: String,
    pub commit_sha: String,
}

pub struct SavePageParams<'a> {
    pub slug: &'a str,
    pub title: &'a str,
    pub content: &'a str,
    pub author: &'a str,
    pub email: &'a str,
    pub message: &'a str,
}

#[derive(Clone)]
pub struct WikiGitBackend {
    base_path: PathBuf,
}

impl WikiGitBackend {
    pub fn new(base_path: PathBuf) -> Result<Self, CoreError> {
        std::fs::create_dir_all(&base_path)?;
        Ok(Self { base_path })
    }

    pub fn wiki_repo_path(&self, repo_id: &str) -> PathBuf {
        self.base_path.join(format!("{repo_id}.wiki.git"))
    }

    pub fn init_wiki_repo(&self, repo_id: &str) -> Result<(), CoreError> {
        let path = self.wiki_repo_path(repo_id);
        if path.join("HEAD").exists() {
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        gix::init_bare(&path).map_err(|e| CoreError::Git(format!("init bare wiki repo: {e}")))?;
        Ok(())
    }

    fn open_repo(&self, repo_id: &str) -> Result<gix::Repository, CoreError> {
        let path = self.wiki_repo_path(repo_id);
        gix::open(&path).map_err(|e| CoreError::Git(format!("open wiki repo {repo_id}: {e}")))
    }

    fn build_tree(
        repo: &gix::Repository,
        existing_tree_id: Option<gix::ObjectId>,
        slug: &str,
        blob_id: gix::ObjectId,
    ) -> Result<gix::ObjectId, CoreError> {
        let filename = format!("{slug}.md");

        let mut entries: Vec<gix::objs::tree::Entry> = Vec::new();
        if let Some(tree_id) = existing_tree_id
            && let Ok(tree) = repo.find_tree(tree_id)
        {
            for e in tree.iter().flatten() {
                let name = e.filename().to_string();
                if name != filename {
                    entries.push(gix::objs::tree::Entry {
                        mode: e.mode(),
                        filename: e.filename().to_owned(),
                        oid: e.oid().to_owned(),
                    });
                }
            }
        }
        entries.push(gix::objs::tree::Entry {
            mode: gix::objs::tree::EntryKind::Blob.into(),
            filename: filename.into(),
            oid: blob_id,
        });
        entries.sort();

        let tree = gix::objs::Tree { entries };
        let tree_id = repo
            .write_object(&tree)
            .map_err(|e| CoreError::Git(format!("write tree: {e}")))?;
        Ok(tree_id.detach())
    }

    fn build_tree_without_file(
        repo: &gix::Repository,
        tree_id: gix::ObjectId,
        slug: &str,
    ) -> Result<gix::ObjectId, CoreError> {
        let filename = format!("{slug}.md");
        let tree = repo
            .find_tree(tree_id)
            .map_err(|e| CoreError::Git(format!("find tree: {e}")))?;
        let mut entries: Vec<gix::objs::tree::Entry> = Vec::new();
        for e in tree.iter().flatten() {
            let name = e.filename().to_string();
            if name != filename {
                entries.push(gix::objs::tree::Entry {
                    mode: e.mode(),
                    filename: e.filename().to_owned(),
                    oid: e.oid().to_owned(),
                });
            }
        }
        entries.sort();
        let tree_obj = gix::objs::Tree { entries };
        let tree_id = repo
            .write_object(&tree_obj)
            .map_err(|e| CoreError::Git(format!("write tree: {e}")))?;
        Ok(tree_id.detach())
    }

    fn get_head_tree_id(repo: &gix::Repository) -> Result<Option<gix::ObjectId>, CoreError> {
        match repo.head_id() {
            Ok(id) => {
                let commit = repo
                    .find_commit(id.detach())
                    .map_err(|e| CoreError::Git(format!("find HEAD commit: {e}")))?;
                Ok(Some(
                    commit
                        .tree_id()
                        .map_err(|e| CoreError::Git(format!("get tree id: {e}")))?
                        .detach(),
                ))
            }
            Err(_) => Ok(None),
        }
    }

    fn commit_to_repo(
        repo: &gix::Repository,
        tree_id: gix::ObjectId,
        parent_id: Option<gix::ObjectId>,
        author: &str,
        email: &str,
        message: &str,
    ) -> Result<String, CoreError> {
        let time = gix::date::Time::new(chrono::Utc::now().timestamp(), 0);
        let sig = gix::actor::Signature {
            name: author.to_string().into(),
            email: email.to_string().into(),
            time,
        };

        let parents: smallvec::SmallVec<[gix::ObjectId; 1]> =
            parent_id.map_or(smallvec::SmallVec::new(), |p| smallvec::smallvec![p]);

        let commit = gix::objs::Commit {
            tree: tree_id,
            parents,
            author: sig.clone(),
            committer: sig,
            encoding: None,
            message: message.to_string().into(),
            extra_headers: Default::default(),
        };
        let commit_id = repo
            .write_object(&commit)
            .map_err(|e| CoreError::Git(format!("write commit: {e}")))?;

        Self::update_head_ref(repo, commit_id.detach(), parent_id)?;

        Ok(commit_id.to_hex().to_string())
    }

    fn update_head_ref(
        repo: &gix::Repository,
        new_id: gix::ObjectId,
        parent_id: Option<gix::ObjectId>,
    ) -> Result<(), CoreError> {
        use gix::refs::Target;
        use gix::refs::transaction::{Change, PreviousValue, RefEdit};

        let head_name = match repo.head_name() {
            Ok(Some(name)) => name.to_owned(),
            Ok(None) | Err(_) => {
                let full_name: gix::refs::FullName =
                    "refs/heads/main".try_into().unwrap_or_else(|_| {
                        gix::refs::FullName::try_from("refs/heads/master").expect("operation should succeed")
                    });
                full_name
            }
        };

        let expected = match parent_id {
            Some(p) => PreviousValue::ExistingMustMatch(Target::Object(p)),
            None => PreviousValue::MustNotExist,
        };

        let edit = RefEdit {
            change: Change::Update {
                log: gix::refs::transaction::LogChange {
                    mode: gix::refs::transaction::RefLog::AndReference,
                    force_create_reflog: false,
                    message: "commit".into(),
                },
                expected,
                new: Target::Object(new_id),
            },
            name: head_name,
            deref: true,
        };

        repo.edit_reference(edit)
            .map_err(|e| CoreError::Git(format!("update ref: {e}")))?;

        Ok(())
    }

    pub fn save_page(
        &self,
        repo_id: &str,
        params: SavePageParams,
    ) -> Result<(String, String), CoreError> {
        self.init_wiki_repo(repo_id)?;
        let repo = self.open_repo(repo_id)?;

        let blob_id = repo
            .write_blob(params.content.as_bytes())
            .map_err(|e| CoreError::Git(format!("write blob: {e}")))?;

        let existing_tree_id = Self::get_head_tree_id(&repo)?;
        let parent_id = repo.head_id().ok().map(|id| id.detach());

        let tree_id = Self::build_tree(&repo, existing_tree_id, params.slug, blob_id.detach())?;

        let full_message = if params.title.is_empty() {
            params.message.to_string()
        } else {
            format!("{}: {}", params.message, params.title)
        };

        let commit_sha = Self::commit_to_repo(
            &repo,
            tree_id,
            parent_id,
            params.author,
            params.email,
            &full_message,
        )?;
        let timestamp = chrono::DateTime::from_timestamp(chrono::Utc::now().timestamp(), 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default()
            .to_string();

        Ok((commit_sha, timestamp))
    }

    pub fn get_page(
        &self,
        repo_id: &str,
        slug: &str,
    ) -> Result<Option<(String, String)>, CoreError> {
        let path = self.wiki_repo_path(repo_id);
        if !path.join("HEAD").exists() {
            return Ok(None);
        }
        let repo = self.open_repo(repo_id)?;
        let head_id = match repo.head_id() {
            Ok(id) => id,
            Err(_) => return Ok(None),
        };

        let commit = repo
            .find_commit(head_id.detach())
            .map_err(|e| CoreError::Git(format!("find HEAD commit: {e}")))?;
        let commit_sha = head_id.to_hex().to_string();

        let tree = commit
            .tree()
            .map_err(|e| CoreError::Git(format!("get commit tree: {e}")))?;
        let filename = format!("{slug}.md");
        let entry = tree.find_entry(filename.as_bytes());

        match entry {
            Some(entry) => {
                let obj = entry
                    .object()
                    .map_err(|e| CoreError::Git(format!("get blob object: {e}")))?;
                let blob = obj
                    .try_into_blob()
                    .map_err(|e| CoreError::Git(format!("expected blob: {e}")))?;
                let content = String::from_utf8_lossy(&blob.data).to_string();
                Ok(Some((content, commit_sha)))
            }
            None => Ok(None),
        }
    }

    pub fn get_page_history(
        &self,
        repo_id: &str,
        slug: &str,
    ) -> Result<Vec<WikiRevision>, CoreError> {
        let path = self.wiki_repo_path(repo_id);
        if !path.join("HEAD").exists() {
            return Ok(Vec::new());
        }
        let repo = self.open_repo(repo_id)?;
        let head_id = match repo.head_id() {
            Ok(id) => id,
            Err(_) => return Ok(Vec::new()),
        };

        let filename = format!("{slug}.md");
        let mut revisions = Vec::new();
        let mut current_id = head_id.detach();

        loop {
            let commit_obj = repo
                .find_object(current_id)
                .map_err(|e| CoreError::Git(format!("find commit object: {e}")))?;
            let commit = commit_obj
                .try_into_commit()
                .map_err(|e| CoreError::Git(format!("non-commit object: {e}")))?;

            let sha = commit.id().to_hex().to_string();
            let tree = commit
                .tree()
                .map_err(|e| CoreError::Git(format!("get tree: {e}")))?;

            let current_blob_id: Option<Vec<u8>> = tree
                .find_entry(filename.as_bytes())
                .map(|e| e.oid().as_bytes().to_vec());

            let parent_ids: Vec<gix::ObjectId> =
                commit.parent_ids().map(|id| id.detach()).collect();

            let parent_blob_id: Option<Vec<u8>> = match parent_ids.first() {
                Some(parent_id) => {
                    let parent_commit = repo
                        .find_object(*parent_id)
                        .map_err(|e| CoreError::Git(format!("find parent commit: {e}")))?;
                    let parent_commit = parent_commit
                        .try_into_commit()
                        .map_err(|e| CoreError::Git(format!("non-commit parent: {e}")))?;
                    let parent_tree = parent_commit
                        .tree()
                        .map_err(|e| CoreError::Git(format!("get parent tree: {e}")))?;
                    parent_tree
                        .find_entry(filename.as_bytes())
                        .map(|e| e.oid().as_bytes().to_vec())
                }
                None => None,
            };

            let file_changed = match (&current_blob_id, &parent_blob_id) {
                (None, None) => false,
                (Some(_), None) | (None, Some(_)) => true,
                (Some(a), Some(b)) => a != b,
            };

            if file_changed {
                let author = commit.author().ok();
                let time = commit.time().ok();
                revisions.push(WikiRevision {
                    sha: sha.clone(),
                    author: author
                        .as_ref()
                        .map(|a| a.name.to_string())
                        .unwrap_or_default(),
                    email: author
                        .as_ref()
                        .map(|a| a.email.to_string())
                        .unwrap_or_default(),
                    message: commit
                        .message()
                        .map(|m| m.summary().to_string())
                        .unwrap_or_default(),
                    timestamp: time
                        .map(|t| {
                            chrono::DateTime::from_timestamp(t.seconds, 0)
                                .map(|dt| dt.to_rfc3339())
                                .unwrap_or_default()
                        })
                        .unwrap_or_default(),
                });
            }

            match parent_ids.first() {
                Some(p) => current_id = *p,
                None => break,
            }
        }

        revisions.reverse();
        Ok(revisions)
    }

    pub fn delete_page(
        &self,
        repo_id: &str,
        slug: &str,
        author: &str,
        email: &str,
        message: &str,
    ) -> Result<String, CoreError> {
        let repo = self.open_repo(repo_id)?;
        let head_id = repo
            .head_id()
            .map_err(|e| CoreError::Git(format!("get head: {e}")))?;
        let parent_id = Some(head_id.detach());

        let commit = repo
            .find_commit(head_id.detach())
            .map_err(|e| CoreError::Git(format!("find commit: {e}")))?;
        let tree_id = commit
            .tree_id()
            .map_err(|e| CoreError::Git(format!("get tree id: {e}")))?;

        let new_tree_id = Self::build_tree_without_file(&repo, tree_id.detach(), slug)?;
        let full_message = format!("Delete {slug}: {message}");

        let commit_sha =
            Self::commit_to_repo(&repo, new_tree_id, parent_id, author, email, &full_message)?;
        Ok(commit_sha)
    }

    pub fn list_pages(&self, repo_id: &str) -> Result<Vec<WikiPageEntry>, CoreError> {
        let path = self.wiki_repo_path(repo_id);
        if !path.join("HEAD").exists() {
            return Ok(Vec::new());
        }
        let repo = self.open_repo(repo_id)?;
        let head_id = match repo.head_id() {
            Ok(id) => id,
            Err(_) => return Ok(Vec::new()),
        };
        let commit_sha = head_id.to_hex().to_string();

        let commit = repo
            .find_commit(head_id.detach())
            .map_err(|e| CoreError::Git(format!("find commit: {e}")))?;
        let tree = commit
            .tree()
            .map_err(|e| CoreError::Git(format!("get tree: {e}")))?;

        let mut pages = Vec::new();
        for entry in tree.iter() {
            let entry = entry.map_err(|e| CoreError::Git(format!("read tree entry: {e}")))?;
            let name = entry.filename().to_string();
            if name.ends_with(".md") {
                let slug = name.trim_end_matches(".md").to_string();
                let title = slug.replace('-', " ");
                let obj = entry
                    .object()
                    .map_err(|e| CoreError::Git(format!("get blob: {e}")))?;
                let blob = obj
                    .try_into_blob()
                    .map_err(|e| CoreError::Git(format!("not a blob: {e}")))?;
                let blob_content = String::from_utf8_lossy(&blob.data).to_string();
                let first_line = blob_content.lines().next().unwrap_or("");
                let title_from_content = first_line.trim_start_matches('#').trim().to_string();
                let title = if title_from_content.is_empty() {
                    title
                } else {
                    title_from_content
                };
                pages.push(WikiPageEntry {
                    slug,
                    title,
                    commit_sha: commit_sha.clone(),
                });
            }
        }

        Ok(pages)
    }

    pub fn page_exists(&self, repo_id: &str, slug: &str) -> Result<bool, CoreError> {
        match self.get_page(repo_id, slug) {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(_) => Ok(false),
        }
    }

    pub fn get_diff(
        &self,
        repo_id: &str,
        slug: &str,
        sha1: &str,
        sha2: &str,
    ) -> Result<String, CoreError> {
        let repo = self.open_repo(repo_id)?;
        let filename = format!("{slug}.md");

        let oid1 = gix::ObjectId::from_hex(sha1.as_bytes())
            .map_err(|e| CoreError::Git(format!("invalid sha1: {e}")))?;
        let oid2 = gix::ObjectId::from_hex(sha2.as_bytes())
            .map_err(|e| CoreError::Git(format!("invalid sha2: {e}")))?;

        let content1 = Self::content_at_commit(&repo, &oid1, &filename)?;
        let content2 = Self::content_at_commit(&repo, &oid2, &filename)?;

        Ok(unified_diff(&content1, &content2))
    }

    fn content_at_commit(
        repo: &gix::Repository,
        commit_oid: &gix::ObjectId,
        filename: &str,
    ) -> Result<String, CoreError> {
        let commit = repo
            .find_commit(*commit_oid)
            .map_err(|e| CoreError::Git(format!("find commit: {e}")))?;
        let tree = commit
            .tree()
            .map_err(|e| CoreError::Git(format!("get tree: {e}")))?;

        match tree.find_entry(filename.as_bytes()) {
            Some(entry) => {
                let obj = entry
                    .object()
                    .map_err(|e| CoreError::Git(format!("get blob: {e}")))?;
                let blob = obj
                    .try_into_blob()
                    .map_err(|e| CoreError::Git(format!("not blob: {e}")))?;
                Ok(String::from_utf8_lossy(&blob.data).to_string())
            }
            None => Ok(String::new()),
        }
    }

    pub fn search_content(
        &self,
        repo_id: &str,
        query: &str,
    ) -> Result<Vec<(String, String)>, CoreError> {
        let pages = self.list_pages(repo_id)?;
        let mut results = Vec::new();
        for page in &pages {
            if let Ok(Some((content, _))) = self.get_page(repo_id, &page.slug)
                && (content.contains(query) || page.title.contains(query))
            {
                results.push((page.slug.clone(), page.title.clone()));
            }
        }
        Ok(results)
    }
}

fn unified_diff(old: &str, new: &str) -> String {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    let hunks = compute_line_diff(&old_lines, &new_lines);
    let mut output = String::new();
    for hunk in &hunks {
        output.push_str(&hunk.to_string());
    }
    if output.is_empty() {
        output.push_str("--- No differences ---\n");
    }
    output
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DiffLine<'a> {
    Context(&'a str),
    Added(&'a str),
    Removed(&'a str),
}

#[derive(Debug)]
struct DiffHunk<'a> {
    old_start: usize,
    old_count: usize,
    new_start: usize,
    new_count: usize,
    lines: Vec<DiffLine<'a>>,
}

impl<'a> std::fmt::Display for DiffHunk<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "@@ -{},{} +{},{} @@",
            self.old_start + 1,
            self.old_count,
            self.new_start + 1,
            self.new_count
        )?;
        for line in &self.lines {
            match line {
                DiffLine::Context(s) => writeln!(f, " {s}")?,
                DiffLine::Added(s) => writeln!(f, "+{s}")?,
                DiffLine::Removed(s) => writeln!(f, "-{s}")?,
            }
        }
        Ok(())
    }
}

fn compute_line_diff<'a>(old: &[&'a str], new: &[&'a str]) -> Vec<DiffHunk<'a>> {
    if old.is_empty() && new.is_empty() {
        return vec![];
    }

    let lcs = lcs_lines(old, new);
    let mut hunks: Vec<DiffHunk<'a>> = Vec::new();
    let mut current_hunk: Option<DiffHunk<'a>> = None;

    let mut oi = 0usize;
    let mut ni = 0usize;
    let mut li = 0usize;

    while oi < old.len() || ni < new.len() {
        let old_done = oi >= old.len();
        let new_done = ni >= new.len();
        let lcs_done = li >= lcs.len();

        if !old_done && !new_done && !lcs_done && old[oi] == lcs[li] && new[ni] == lcs[li] {
            let line = DiffLine::Context(old[oi]);
            if let Some(ref mut h) = current_hunk {
                h.lines.push(line);
                h.old_count += 1;
                h.new_count += 1;
            }
            oi += 1;
            ni += 1;
            li += 1;
        } else {
            let removed_start = oi;
            let added_start = ni;

            while oi < old.len() && (li >= lcs.len() || old[oi] != lcs[li]) {
                oi += 1;
            }

            while ni < new.len() && (li >= lcs.len() || new[ni] != lcs[li]) {
                ni += 1;
            }

            if li < lcs.len() {
                li += 1;
            }

            if current_hunk.is_none() {
                current_hunk = Some(DiffHunk {
                    old_start: removed_start.saturating_sub(3).min(removed_start),
                    old_count: 0,
                    new_start: added_start.saturating_sub(3).min(added_start),
                    new_count: 0,
                    lines: Vec::new(),
                });
            }

            if let Some(ref mut h) = current_hunk {
                for line in &old[removed_start..oi] {
                    h.lines.push(DiffLine::Removed(line));
                    h.old_count += 1;
                }
                for line in &new[added_start..ni] {
                    h.lines.push(DiffLine::Added(line));
                    h.new_count += 1;
                }
            }
        }
    }

    if let Some(h) = current_hunk {
        hunks.push(h);
    }

    hunks
}

fn lcs_lines<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<&'a str> {
    let m = a.len();
    let n = b.len();

    if m > 5000 || n > 5000 {
        let mut result = Vec::new();
        let mut bi = 0usize;
        for line in a {
            if bi < n && b[bi] == *line {
                result.push(*line);
                bi += 1;
            }
        }
        return result;
    }

    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if a[i - 1] == b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut result = Vec::new();
    let (mut i, mut j) = (m, n);
    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            result.push(a[i - 1]);
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    result.reverse();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sp<'a>(slug: &'a str, title: &'a str, content: &'a str) -> SavePageParams<'a> {
        SavePageParams {
            slug,
            title,
            content,
            author: "a",
            email: "a@x.com",
            message: "create",
        }
    }

    fn make_backend() -> WikiGitBackend {
        let tmp = tempfile::tempdir().unwrap();
        WikiGitBackend::new(tmp.path().to_path_buf()).unwrap()
    }

    #[test]
    fn test_init_repo_idempotent() {
        let backend = make_backend();
        backend.init_wiki_repo("repo1").unwrap();
        backend.init_wiki_repo("repo1").unwrap();
        let path = backend.wiki_repo_path("repo1");
        assert!(path.join("HEAD").exists());
    }

    #[test]
    fn test_wiki_repo_path_format() {
        let backend = make_backend();
        let path = backend.wiki_repo_path("abc-123");
        assert!(path.to_string_lossy().contains("abc-123.wiki.git"));
    }

    #[test]
    fn test_save_and_read_page_roundtrip() {
        let backend = make_backend();
        let (sha, ts) = backend
            .save_page(
                "repo1",
                SavePageParams {
                    slug: "home",
                    title: "Home",
                    content: "# Home\n\nWelcome!",
                    author: "alice",
                    email: "alice@example.com",
                    message: "create page",
                },
            )
            .unwrap();
        assert!(!sha.is_empty());
        assert!(!ts.is_empty());
        assert_eq!(sha.len(), 40);

        let (content, commit_sha) = backend.get_page("repo1", "home").unwrap().unwrap();
        assert_eq!(content, "# Home\n\nWelcome!");
        assert_eq!(commit_sha, sha);
    }

    #[test]
    fn test_read_nonexistent_page() {
        let backend = make_backend();
        backend.init_wiki_repo("repo1").unwrap();
        let result = backend.get_page("repo1", "nonexistent");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_multiple_pages_same_repo() {
        let backend = make_backend();
        backend
            .save_page("repo1", sp("home", "Home", "# Home"))
            .unwrap();
        backend
            .save_page("repo1", sp("about", "About", "# About"))
            .unwrap();

        let (c1, _) = backend.get_page("repo1", "home").unwrap().unwrap();
        assert_eq!(c1, "# Home");
        let (c2, _) = backend.get_page("repo1", "about").unwrap().unwrap();
        assert_eq!(c2, "# About");
    }

    #[test]
    fn test_page_update_new_sha() {
        let backend = make_backend();
        let (sha1, _) = backend
            .save_page("repo1", sp("home", "Home", "v1"))
            .unwrap();
        let (sha2, _) = backend
            .save_page(
                "repo1",
                SavePageParams {
                    slug: "home",
                    title: "Home",
                    content: "v2",
                    author: "a",
                    email: "a@x.com",
                    message: "update page",
                },
            )
            .unwrap();
        assert_ne!(sha1, sha2);
        let (content, sha) = backend.get_page("repo1", "home").unwrap().unwrap();
        assert_eq!(content, "v2");
        assert_eq!(sha, sha2);
    }

    #[test]
    fn test_delete_page() {
        let backend = make_backend();
        backend
            .save_page("repo1", sp("home", "Home", "content"))
            .unwrap();
        assert!(backend.page_exists("repo1", "home").unwrap());

        let sha = backend
            .delete_page("repo1", "home", "a", "a@x.com", "remove")
            .unwrap();
        assert!(!sha.is_empty());
        assert!(backend.get_page("repo1", "home").unwrap().is_none());
    }

    #[test]
    fn test_delete_nonexistent_page() {
        let backend = make_backend();
        backend.init_wiki_repo("repo1").unwrap();
        let result = backend.delete_page("repo1", "ghost", "a", "a@x.com", "del");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_pages() {
        let backend = make_backend();
        backend
            .save_page("repo1", sp("home", "Home", "# Home"))
            .unwrap();
        backend
            .save_page(
                "repo1",
                SavePageParams {
                    slug: "about",
                    title: "About",
                    content: "# About Us",
                    author: "a",
                    email: "a@x.com",
                    message: "create",
                },
            )
            .unwrap();

        let pages = backend.list_pages("repo1").unwrap();
        assert_eq!(pages.len(), 2);
        let slugs: Vec<&str> = pages.iter().map(|p| p.slug.as_str()).collect();
        assert!(slugs.contains(&"home"));
        assert!(slugs.contains(&"about"));
    }

    #[test]
    fn test_list_pages_empty_repo() {
        let backend = make_backend();
        let pages = backend.list_pages("repo1").unwrap();
        assert!(pages.is_empty());
    }

    #[test]
    fn test_page_history() {
        let backend = make_backend();
        backend
            .save_page("repo1", sp("home", "Home", "v1"))
            .unwrap();
        backend
            .save_page(
                "repo1",
                SavePageParams {
                    slug: "home",
                    title: "Home",
                    content: "v2",
                    author: "b",
                    email: "b@x.com",
                    message: "update",
                },
            )
            .unwrap();
        backend
            .save_page(
                "repo1",
                SavePageParams {
                    slug: "home",
                    title: "Home",
                    content: "v3",
                    author: "c",
                    email: "c@x.com",
                    message: "fix",
                },
            )
            .unwrap();

        let history = backend.get_page_history("repo1", "home").unwrap();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].message, "create: Home");
        assert_eq!(history[1].message, "update: Home");
        assert_eq!(history[2].message, "fix: Home");
        assert_eq!(history[0].author, "a");
        assert_eq!(history[1].author, "b");
        assert_eq!(history[2].author, "c");
    }

    #[test]
    fn test_page_history_empty_repo() {
        let backend = make_backend();
        let history = backend.get_page_history("repo1", "home").unwrap();
        assert!(history.is_empty());
    }

    #[test]
    fn test_page_exists_true() {
        let backend = make_backend();
        backend
            .save_page("repo1", sp("home", "Home", "content"))
            .unwrap();
        assert!(backend.page_exists("repo1", "home").unwrap());
    }

    #[test]
    fn test_page_exists_false() {
        let backend = make_backend();
        assert!(!backend.page_exists("repo1", "nope").unwrap());
    }

    #[test]
    fn test_diff_between_versions() {
        let backend = make_backend();
        let (sha1, _) = backend
            .save_page(
                "repo1",
                SavePageParams {
                    slug: "home",
                    title: "Home",
                    content: "line1\nline2",
                    author: "a",
                    email: "a@x.com",
                    message: "create",
                },
            )
            .unwrap();
        let (sha2, _) = backend
            .save_page(
                "repo1",
                SavePageParams {
                    slug: "home",
                    title: "Home",
                    content: "line1\nline3",
                    author: "a",
                    email: "a@x.com",
                    message: "update",
                },
            )
            .unwrap();

        let diff = backend.get_diff("repo1", "home", &sha1, &sha2).unwrap();
        assert!(diff.contains("-line2"));
        assert!(diff.contains("+line3"));
    }

    #[test]
    fn test_diff_same_version() {
        let backend = make_backend();
        let (sha, _) = backend
            .save_page("repo1", sp("home", "Home", "same"))
            .unwrap();

        let diff = backend.get_diff("repo1", "home", &sha, &sha).unwrap();
        assert!(diff.contains("No differences"));
    }

    #[test]
    fn test_search_content() {
        let backend = make_backend();
        backend
            .save_page(
                "repo1",
                SavePageParams {
                    slug: "install",
                    title: "Installation",
                    content: "How to install the tool",
                    author: "a",
                    email: "a@x.com",
                    message: "create",
                },
            )
            .unwrap();
        backend
            .save_page(
                "repo1",
                SavePageParams {
                    slug: "usage",
                    title: "Usage",
                    content: "How to use the tool",
                    author: "a",
                    email: "a@x.com",
                    message: "create",
                },
            )
            .unwrap();

        let results = backend.search_content("repo1", "install").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "install");
    }

    #[test]
    fn test_search_content_multiple_matches() {
        let backend = make_backend();
        backend
            .save_page(
                "repo1",
                SavePageParams {
                    slug: "a",
                    title: "Page A",
                    content: "common content here",
                    author: "a",
                    email: "a@x.com",
                    message: "create",
                },
            )
            .unwrap();
        backend
            .save_page(
                "repo1",
                SavePageParams {
                    slug: "b",
                    title: "Page B",
                    content: "common other",
                    author: "a",
                    email: "a@x.com",
                    message: "create",
                },
            )
            .unwrap();

        let results = backend.search_content("repo1", "common").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_unicode_content() {
        let backend = make_backend();
        let content = "# \u{65e5}\u{672c}\u{8a9e}\n\n\u{3053}\u{3093}\u{306b}\u{3061}\u{306f}\u{4e16}\u{754c}\n\n\u{2728} \u{2192} \u{2603}";
        backend
            .save_page("repo1", sp("ja", "日本語", content))
            .unwrap();
        let (retrieved, _) = backend.get_page("repo1", "ja").unwrap().unwrap();
        assert_eq!(retrieved, content);
    }

    #[test]
    fn test_large_content() {
        let backend = make_backend();
        let content: String = (0..1000)
            .map(|i| format!("Line {i}: some content here"))
            .collect::<Vec<_>>()
            .join("\n");
        backend
            .save_page(
                "repo1",
                SavePageParams {
                    slug: "big",
                    title: "Big Page",
                    content: &content,
                    author: "a",
                    email: "a@x.com",
                    message: "create",
                },
            )
            .unwrap();
        let (retrieved, _) = backend.get_page("repo1", "big").unwrap().unwrap();
        assert_eq!(retrieved, content);
    }

    #[test]
    fn test_empty_content() {
        let backend = make_backend();
        backend
            .save_page("repo1", sp("empty", "Empty", ""))
            .unwrap();
        let (content, _) = backend.get_page("repo1", "empty").unwrap().unwrap();
        assert_eq!(content, "");
    }

    #[test]
    fn test_author_email_metadata() {
        let backend = make_backend();
        backend
            .save_page(
                "repo1",
                SavePageParams {
                    slug: "meta",
                    title: "Meta",
                    content: "content",
                    author: "TestAuthor",
                    email: "test@example.org",
                    message: "create page",
                },
            )
            .unwrap();
        let history = backend.get_page_history("repo1", "meta").unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].author, "TestAuthor");
        assert_eq!(history[0].email, "test@example.org");
    }

    #[test]
    fn test_commit_message_format() {
        let backend = make_backend();
        let (_, _) = backend
            .save_page(
                "repo1",
                SavePageParams {
                    slug: "msg",
                    title: "My Title",
                    content: "c",
                    author: "a",
                    email: "a@x.com",
                    message: "create page",
                },
            )
            .unwrap();
        let history = backend.get_page_history("repo1", "msg").unwrap();
        assert_eq!(history[0].message, "create page: My Title");
    }

    #[test]
    fn test_multiple_repos_isolation() {
        let backend = make_backend();
        backend
            .save_page(
                "repo1",
                SavePageParams {
                    slug: "home",
                    title: "Home",
                    content: "repo1 content",
                    author: "a",
                    email: "a@x.com",
                    message: "create",
                },
            )
            .unwrap();
        backend
            .save_page(
                "repo2",
                SavePageParams {
                    slug: "home",
                    title: "Home",
                    content: "repo2 content",
                    author: "a",
                    email: "a@x.com",
                    message: "create",
                },
            )
            .unwrap();

        let (c1, _) = backend.get_page("repo1", "home").unwrap().unwrap();
        let (c2, _) = backend.get_page("repo2", "home").unwrap().unwrap();
        assert_eq!(c1, "repo1 content");
        assert_eq!(c2, "repo2 content");
    }

    #[test]
    fn test_list_pages_after_delete() {
        let backend = make_backend();
        backend
            .save_page("repo1", sp("a", "A", "content a"))
            .unwrap();
        backend
            .save_page("repo1", sp("b", "B", "content b"))
            .unwrap();
        backend
            .delete_page("repo1", "a", "a", "a@x.com", "del")
            .unwrap();

        let pages = backend.list_pages("repo1").unwrap();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].slug, "b");
    }

    #[test]
    fn test_history_only_for_target_page() {
        let backend = make_backend();
        backend.save_page("repo1", sp("a", "A", "a v1")).unwrap();
        backend.save_page("repo1", sp("b", "B", "b v1")).unwrap();
        backend
            .save_page(
                "repo1",
                SavePageParams {
                    slug: "a",
                    title: "A",
                    content: "a v2",
                    author: "a",
                    email: "a@x.com",
                    message: "update",
                },
            )
            .unwrap();

        let hist_a = backend.get_page_history("repo1", "a").unwrap();
        assert_eq!(hist_a.len(), 2);
        let hist_b = backend.get_page_history("repo1", "b").unwrap();
        assert_eq!(hist_b.len(), 1);
    }

    #[test]
    fn test_diff_hunk_format() {
        let hunk = DiffHunk {
            old_start: 0,
            old_count: 1,
            new_start: 0,
            new_count: 1,
            lines: vec![DiffLine::Removed("old"), DiffLine::Added("new")],
        };
        let s = format!("{hunk}");
        assert!(s.starts_with("@@"));
        assert!(s.contains("-old"));
        assert!(s.contains("+new"));
    }

    #[test]
    fn test_lcs_basic() {
        let a: Vec<&str> = vec!["a", "b", "c", "d"];
        let b: Vec<&str> = vec!["a", "c", "d", "e"];
        let lcs = lcs_lines(&a, &b);
        assert_eq!(lcs, vec!["a", "c", "d"]);
    }

    #[test]
    fn test_lcs_empty() {
        let a: Vec<&str> = vec![];
        let b: Vec<&str> = vec!["x", "y"];
        let lcs = lcs_lines(&a, &b);
        assert!(lcs.is_empty());
    }

    #[test]
    fn test_unified_diff_no_changes() {
        let diff = unified_diff("line1\nline2\nline3", "line1\nline2\nline3");
        assert!(diff.contains("No differences"));
    }

    #[test]
    fn test_unified_diff_added_lines() {
        let old = "line1\nline3";
        let new = "line1\nline2\nline3";
        let diff = unified_diff(old, new);
        assert!(diff.contains("+line2"));
        assert!(diff.contains("@@"));
    }
}
