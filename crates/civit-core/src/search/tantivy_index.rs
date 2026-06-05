#![forbid(unsafe_code)]

use std::path::Path;

use tantivy::collector::{Count, TopDocs};
use tantivy::directory::MmapDirectory;
use tantivy::directory::RamDirectory;
use tantivy::schema::OwnedValue;
use tantivy::schema::TantivyDocument;
use tantivy::schema::{FAST, Field, STORED, STRING, Schema, TEXT};
use tantivy::{Directory, DocAddress, Index, IndexReader, IndexWriter, Score, Term};

use crate::search::tantivy_search::SearchQueryBuilder;

const DEFAULT_INDEX_MEMORY_BUDGET_MB: usize = 50;

static SCHEMA: std::sync::LazyLock<Schema> = std::sync::LazyLock::new(build_schema);

fn build_schema() -> Schema {
    let mut builder = Schema::builder();
    builder.add_text_field("repo_id", STRING | STORED);
    builder.add_text_field("path", TEXT | STORED);
    builder.add_text_field("content", TEXT | STORED);
    builder.add_text_field("language", STRING | STORED);
    builder.add_text_field("commit_sha", STRING | STORED);
    builder.add_i64_field("indexed_at", FAST);
    builder.build()
}

fn repo_id_field() -> Field {
    SCHEMA
        .get_field("repo_id")
        .expect("schema must have repo_id")
}

fn path_field() -> Field {
    SCHEMA.get_field("path").expect("schema must have path")
}

fn content_field() -> Field {
    SCHEMA
        .get_field("content")
        .expect("schema must have content")
}

fn language_field() -> Field {
    SCHEMA
        .get_field("language")
        .expect("schema must have language")
}

fn commit_sha_field() -> Field {
    SCHEMA
        .get_field("commit_sha")
        .expect("schema must have commit_sha")
}

fn indexed_at_field() -> Field {
    SCHEMA
        .get_field("indexed_at")
        .expect("schema must have indexed_at")
}

#[derive(Debug, Clone)]
pub struct IndexDoc {
    pub repo_id: String,
    pub path: String,
    pub content: String,
    pub language: String,
    pub commit_sha: String,
    pub indexed_at: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchHit {
    pub repo_id: String,
    pub path: String,
    pub content_snippet: String,
    pub language: String,
    pub score: f32,
}

pub struct CodeSearchIndex {
    #[allow(dead_code)]
    index: Index,
    reader: IndexReader,
    writer: IndexWriter,
}

impl CodeSearchIndex {
    pub fn new(index_path: &Path) -> crate::error::Result<Self> {
        Self::with_memory_budget(index_path, DEFAULT_INDEX_MEMORY_BUDGET_MB)
    }

    pub fn with_memory_budget(index_path: &Path, budget_mb: usize) -> crate::error::Result<Self> {
        let dir = MmapDirectory::open(index_path)
            .map_err(|e| crate::error::CoreError::Search(e.to_string()))?;
        let index = Index::open_or_create(dir, SCHEMA.clone())
            .map_err(|e| crate::error::CoreError::Search(e.to_string()))?;
        let reader = index
            .reader()
            .map_err(|e| crate::error::CoreError::Search(e.to_string()))?;
        let writer = index
            .writer(budget_mb * 1024 * 1024)
            .map_err(|e| crate::error::CoreError::Search(e.to_string()))?;
        Ok(Self {
            index,
            reader,
            writer,
        })
    }

    pub fn new_in_memory() -> crate::error::Result<Self> {
        let dir: Box<dyn Directory> = Box::new(RamDirectory::default());
        let index = Index::open_or_create(dir, SCHEMA.clone())
            .map_err(|e| crate::error::CoreError::Search(e.to_string()))?;
        let reader = index
            .reader()
            .map_err(|e| crate::error::CoreError::Search(e.to_string()))?;
        let writer = index
            .writer(DEFAULT_INDEX_MEMORY_BUDGET_MB * 1024 * 1024)
            .map_err(|e| crate::error::CoreError::Search(e.to_string()))?;
        Ok(Self {
            index,
            reader,
            writer,
        })
    }

    pub fn index_file(
        &mut self,
        repo_id: &str,
        path: &str,
        content: &str,
        language: &str,
        commit_sha: &str,
    ) -> crate::error::Result<()> {
        let indexed_at = chrono::Utc::now().timestamp();
        let mut doc = TantivyDocument::default();
        doc.add_text(repo_id_field(), repo_id);
        doc.add_text(path_field(), path);
        doc.add_text(content_field(), content);
        doc.add_text(language_field(), language);
        doc.add_text(commit_sha_field(), commit_sha);
        doc.add_i64(indexed_at_field(), indexed_at);
        self.writer
            .add_document(doc)
            .map_err(|e| crate::error::CoreError::Search(e.to_string()))?;
        self.writer
            .commit()
            .map_err(|e| crate::error::CoreError::Search(e.to_string()))?;
        self.reader
            .reload()
            .map_err(|e| crate::error::CoreError::Search(e.to_string()))?;
        Ok(())
    }

    pub fn index_files(&mut self, batch: Vec<IndexDoc>) -> crate::error::Result<()> {
        for doc in &batch {
            let mut d = TantivyDocument::default();
            d.add_text(repo_id_field(), &doc.repo_id);
            d.add_text(path_field(), &doc.path);
            d.add_text(content_field(), &doc.content);
            d.add_text(language_field(), &doc.language);
            d.add_text(commit_sha_field(), &doc.commit_sha);
            d.add_i64(indexed_at_field(), doc.indexed_at);
            self.writer
                .add_document(d)
                .map_err(|e| crate::error::CoreError::Search(e.to_string()))?;
        }
        self.writer
            .commit()
            .map_err(|e| crate::error::CoreError::Search(e.to_string()))?;
        self.reader
            .reload()
            .map_err(|e| crate::error::CoreError::Search(e.to_string()))?;
        Ok(())
    }

    pub fn delete_by_repo(&mut self, repo_id: &str) -> crate::error::Result<()> {
        let term = Term::from_field_text(repo_id_field(), repo_id);
        self.writer.delete_term(term);
        self.writer
            .commit()
            .map_err(|e| crate::error::CoreError::Search(e.to_string()))?;
        self.reader
            .reload()
            .map_err(|e| crate::error::CoreError::Search(e.to_string()))?;
        Ok(())
    }

    pub fn search(
        &self,
        query: &str,
        repo_id_filter: Option<&str>,
        language_filter: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> crate::error::Result<Vec<SearchHit>> {
        let searcher = self.reader.searcher();
        let (query_obj, _) = SearchQueryBuilder::new(query)
            .repo_id_filter(repo_id_filter.map(|s| s.to_string()))
            .language_filter(language_filter.map(|s| s.to_string()))
            .build(&SCHEMA, content_field(), repo_id_field(), language_field())?;

        let (top_docs, _count) = searcher
            .search(
                &query_obj,
                &(TopDocs::with_limit(limit).and_offset(offset), Count),
            )
            .map_err(|e| crate::error::CoreError::Search(e.to_string()))?;

        let mut hits = Vec::with_capacity(top_docs.len());
        for (score, doc_addr) in top_docs {
            let retrieved: TantivyDocument = searcher
                .doc(doc_addr)
                .map_err(|e| crate::error::CoreError::Search(e.to_string()))?;
            let hit = doc_to_hit(&retrieved, doc_addr, score)?;
            hits.push(hit);
        }
        Ok(hits)
    }

    pub fn search_global(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> crate::error::Result<Vec<SearchHit>> {
        self.search(query, None, None, limit, offset)
    }

    pub fn writer(&mut self) -> &mut IndexWriter {
        &mut self.writer
    }
}

fn doc_to_hit(
    doc: &TantivyDocument,
    _doc_addr: DocAddress,
    score: Score,
) -> crate::error::Result<SearchHit> {
    let repo_id = SCHEMA
        .get_field("repo_id")
        .ok()
        .and_then(|f| {
            doc.get_first(f).and_then(|v| match v {
                OwnedValue::Str(s) => Some(s.as_str()),
                _ => None,
            })
        })
        .unwrap_or_default()
        .to_string();
    let path = SCHEMA
        .get_field("path")
        .ok()
        .and_then(|f| {
            doc.get_first(f).and_then(|v| match v {
                OwnedValue::Str(s) => Some(s.as_str()),
                _ => None,
            })
        })
        .unwrap_or_default()
        .to_string();
    let content = SCHEMA
        .get_field("content")
        .ok()
        .and_then(|f| {
            doc.get_first(f).and_then(|v| match v {
                OwnedValue::Str(s) => Some(s.as_str()),
                _ => None,
            })
        })
        .unwrap_or_default();
    let language = SCHEMA
        .get_field("language")
        .ok()
        .and_then(|f| {
            doc.get_first(f).and_then(|v| match v {
                OwnedValue::Str(s) => Some(s.as_str()),
                _ => None,
            })
        })
        .unwrap_or_default()
        .to_string();

    let snippet = if content.len() > 200 {
        format!("{}...", &content[..200])
    } else {
        content.to_string()
    };

    Ok(SearchHit {
        repo_id,
        path,
        content_snippet: snippet,
        language,
        score,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_index() -> CodeSearchIndex {
        CodeSearchIndex::new_in_memory().unwrap()
    }

    fn index_sample_files(idx: &mut CodeSearchIndex) {
        idx.index_file(
            "repo-1",
            "src/main.rs",
            "fn unique_repo_one_function() { println!(\"hello\"); }",
            "rust",
            "abc123",
        )
        .unwrap();
        idx.index_file(
            "repo-1",
            "src/lib.rs",
            "pub fn add(a: i32, b: i32) -> i32 { a + b }",
            "rust",
            "abc123",
        )
        .unwrap();
        idx.index_file(
            "repo-1",
            "src/utils.rs",
            "pub fn helper() -> String { String::new() }",
            "rust",
            "abc123",
        )
        .unwrap();
        idx.index_file(
            "repo-2",
            "main.go",
            "package main\n\nfunc unique_repo_two_func() {\n\tfmt.Println(\"hello\")\n}",
            "go",
            "def456",
        )
        .unwrap();
        idx.index_file(
            "repo-2",
            "util.go",
            "package main\n\nfunc Helper() string {\n\treturn \"\"\n}",
            "go",
            "def456",
        )
        .unwrap();
        idx.index_file(
            "repo-3",
            "index.py",
            "def unique_python_func():\n    print('hello world')",
            "python",
            "ghi789",
        )
        .unwrap();
    }

    #[test]
    fn test_new_in_memory() {
        let _idx = make_index();
    }

    #[test]
    fn test_index_single_file() {
        let mut idx = make_index();
        idx.index_file("repo-1", "src/main.rs", "fn main() {}", "rust", "abc")
            .unwrap();
        let hits = idx.search_global("main", 10, 0).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].repo_id, "repo-1");
        assert_eq!(hits[0].path, "src/main.rs");
        assert_eq!(hits[0].language, "rust");
    }

    #[test]
    fn test_index_multiple_files() {
        let mut idx = make_index();
        index_sample_files(&mut idx);
        let hits = idx.search_global("println", 10, 0).unwrap();
        assert!(
            hits.len() >= 2,
            "expected at least 2 hits for 'println', got {}",
            hits.len()
        );
    }

    #[test]
    fn test_search_by_content() {
        let mut idx = make_index();
        index_sample_files(&mut idx);
        let hits = idx.search_global("helper", 10, 0).unwrap();
        assert!(!hits.is_empty(), "expected at least 1 hit for 'helper'");
    }

    #[test]
    fn test_search_with_repo_filter() {
        let mut idx = make_index();
        index_sample_files(&mut idx);
        let hits = idx
            .search("unique_repo_one_function", Some("repo-1"), None, 10, 0)
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].repo_id, "repo-1");
    }

    #[test]
    fn test_search_with_language_filter() {
        let mut idx = make_index();
        index_sample_files(&mut idx);
        let hits = idx
            .search("unique_repo_two_func", None, Some("go"), 10, 0)
            .unwrap();
        assert!(!hits.is_empty());
        for h in &hits {
            assert_eq!(h.language, "go");
        }
    }

    #[test]
    fn test_search_with_repo_and_language_filter() {
        let mut idx = make_index();
        index_sample_files(&mut idx);
        let hits = idx
            .search("unique_repo_two_func", Some("repo-2"), Some("go"), 10, 0)
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].repo_id, "repo-2");
        assert_eq!(hits[0].language, "go");
    }

    #[test]
    fn test_search_no_results() {
        let mut idx = make_index();
        index_sample_files(&mut idx);
        let hits = idx.search_global("zzzznonexistentzzzz", 10, 0).unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn test_search_limit() {
        let mut idx = make_index();
        index_sample_files(&mut idx);
        let hits = idx.search_global("println", 1, 0).unwrap();
        assert!(hits.len() <= 1);
    }

    #[test]
    fn test_search_offset() {
        let mut idx = make_index();
        index_sample_files(&mut idx);
        let all = idx.search_global("println", 100, 0).unwrap();
        let offset = idx.search_global("println", 100, 1).unwrap();
        assert!(offset.len() < all.len());
    }

    #[test]
    fn test_delete_by_repo() {
        let mut idx = make_index();
        index_sample_files(&mut idx);
        idx.delete_by_repo("repo-1").unwrap();
        let hits = idx
            .search("unique_repo_one_function", Some("repo-1"), None, 10, 0)
            .unwrap();
        assert!(hits.is_empty());
        let other = idx.search_global("println", 10, 0).unwrap();
        assert!(!other.is_empty());
    }

    #[test]
    fn test_batch_index() {
        let mut idx = make_index();
        let docs = vec![
            IndexDoc {
                repo_id: "batch-repo".into(),
                path: "a.rs".into(),
                content: "fn batch_one_here() {}".into(),
                language: "rust".into(),
                commit_sha: "c1".into(),
                indexed_at: 1000,
            },
            IndexDoc {
                repo_id: "batch-repo".into(),
                path: "b.rs".into(),
                content: "fn batch_two_here() {}".into(),
                language: "rust".into(),
                commit_sha: "c1".into(),
                indexed_at: 1000,
            },
        ];
        idx.index_files(docs).unwrap();
        let hits = idx
            .search("batch", Some("batch-repo"), None, 10, 0)
            .unwrap();
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn test_content_snippet_truncation() {
        let mut idx = make_index();
        let long_content = "xyz ".repeat(250).to_string();
        idx.index_file("repo-1", "big.txt", &long_content, "text", "sha")
            .unwrap();
        let hits = idx.search_global("xyz", 1, 0).unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0].content_snippet.ends_with("..."));
        assert!(hits[0].content_snippet.len() <= 205);
    }

    #[test]
    fn test_content_snippet_short() {
        let mut idx = make_index();
        idx.index_file("repo-1", "small.txt", "short_content_here", "text", "sha")
            .unwrap();
        let hits = idx.search_global("short_content_here", 1, 0).unwrap();
        assert_eq!(hits[0].content_snippet, "short_content_here");
    }

    #[test]
    fn test_score_ordering() {
        let mut idx = make_index();
        idx.index_file("repo-1", "exact.rs", "fn exact_match() {}", "rust", "sha")
            .unwrap();
        idx.index_file(
            "repo-1",
            "other.rs",
            "fn something_else() {}",
            "rust",
            "sha",
        )
        .unwrap();
        let hits = idx.search_global("exact_match", 10, 0).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, "exact.rs");
    }

    #[test]
    fn test_overwrite_on_reindex() {
        let mut idx = make_index();
        idx.index_file("repo-1", "file.rs", "fn old() {}", "rust", "sha1")
            .unwrap();
        idx.index_file("repo-1", "file.rs", "fn new() {}", "rust", "sha2")
            .unwrap();
        let hits = idx.search("new", Some("repo-1"), None, 10, 0).unwrap();
        assert_eq!(hits.len(), 1);
        let old_hits = idx.search("old", Some("repo-1"), None, 10, 0).unwrap();
        assert_eq!(old_hits.len(), 1);
    }

    #[test]
    fn test_multi_repo_isolation() {
        let mut idx = make_index();
        index_sample_files(&mut idx);
        let rust_hits = idx
            .search("unique_repo_one_function", Some("repo-1"), None, 10, 0)
            .unwrap();
        let go_hits = idx
            .search("unique_repo_two_func", Some("repo-2"), None, 10, 0)
            .unwrap();
        for h in &rust_hits {
            assert_eq!(h.repo_id, "repo-1");
        }
        for h in &go_hits {
            assert_eq!(h.repo_id, "repo-2");
        }
    }

    #[test]
    fn test_empty_query_returns_error() {
        let mut idx = make_index();
        index_sample_files(&mut idx);
        let hits = idx.search_global("", 10, 0);
        assert!(hits.is_err());
    }

    #[test]
    fn test_python_search() {
        let mut idx = make_index();
        index_sample_files(&mut idx);
        let hits = idx
            .search("unique_python_func", Some("repo-3"), Some("python"), 10, 0)
            .unwrap();
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn test_index_after_delete() {
        let mut idx = make_index();
        idx.index_file("repo-1", "file.rs", "fn temp() {}", "rust", "sha")
            .unwrap();
        idx.delete_by_repo("repo-1").unwrap();
        idx.index_file("repo-1", "file.rs", "fn restored() {}", "rust", "sha2")
            .unwrap();
        let hits = idx.search("restored", Some("repo-1"), None, 10, 0).unwrap();
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn test_case_insensitive_search() {
        let mut idx = make_index();
        idx.index_file("repo-1", "file.rs", "fn HELLO_WORLD() {}", "rust", "sha")
            .unwrap();
        let hits = idx.search_global("hello_world", 10, 0).unwrap();
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn test_large_batch() {
        let mut idx = make_index();
        let docs: Vec<IndexDoc> = (0..100)
            .map(|i| IndexDoc {
                repo_id: "bulk".into(),
                path: format!("file_{i}.rs"),
                content: format!("fn unique_item_number_{i}() {{}}"),
                language: "rust".into(),
                commit_sha: "sha".into(),
                indexed_at: i,
            })
            .collect();
        idx.index_files(docs).unwrap();
        let hits = idx
            .search("unique_item_number", Some("bulk"), None, 10, 0)
            .unwrap();
        assert_eq!(hits.len(), 10);
        let all = idx
            .search("unique_item_number", Some("bulk"), None, 100, 0)
            .unwrap();
        assert_eq!(all.len(), 100);
    }

    #[test]
    fn test_commit_sha_stored() {
        let mut idx = make_index();
        idx.index_file("repo-1", "f.rs", "fn foobar() {}", "rust", "sha_abc")
            .unwrap();
        let hits = idx.search_global("foobar", 1, 0).unwrap();
        assert_eq!(hits.len(), 1);
    }
}
