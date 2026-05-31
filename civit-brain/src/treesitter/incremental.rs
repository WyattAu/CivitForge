#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use super::parser::{ParseOptions, ParseResult, TreeSitterParser};

#[derive(Debug, Clone)]
pub struct CachedTree {
    pub source_hash: String,
    pub root: Vec<super::parser::TsNode>,
    pub line_offsets: Vec<usize>,
}

#[derive(Debug)]
pub struct DiffResult {
    pub added_ranges: Vec<(usize, usize)>,
    pub removed_ranges: Vec<(usize, usize)>,
    pub modified_ranges: Vec<(usize, usize)>,
}

pub struct IncrementalParser {
    parser: TreeSitterParser,
    caches: HashMap<String, CachedTree>,
}

impl IncrementalParser {
    pub fn new() -> Self {
        Self {
            parser: TreeSitterParser::new(),
            caches: HashMap::new(),
        }
    }

    pub fn parse_incremental(
        &mut self,
        file_path: &str,
        source: &str,
        language: &str,
    ) -> ParseResult {
        let hash = hash_content(source);

        if let Some(cached) = self.caches.get(file_path) {
            if cached.source_hash == hash {
                return ParseResult {
                    root: cached.root.clone(),
                    error_count: 0,
                    parse_time: std::time::Duration::ZERO,
                };
            }
        }

        let result = self.parser.parse(source, language);

        let line_offsets = compute_line_offsets(source);
        let cached_tree = CachedTree {
            source_hash: hash,
            root: result.root.clone(),
            line_offsets,
        };

        self.caches.insert(file_path.to_string(), cached_tree);
        result
    }

    pub fn parse_incremental_with_options(
        &mut self,
        file_path: &str,
        source: &str,
        language: &str,
        opts: ParseOptions,
    ) -> ParseResult {
        let hash = hash_content(source);

        if let Some(cached) = self.caches.get(file_path) {
            if cached.source_hash == hash {
                return ParseResult {
                    root: cached.root.clone(),
                    error_count: 0,
                    parse_time: std::time::Duration::ZERO,
                };
            }
        }

        let result = self.parser.parse_with_options(source, language, opts);

        let line_offsets = compute_line_offsets(source);
        let cached_tree = CachedTree {
            source_hash: hash,
            root: result.root.clone(),
            line_offsets,
        };

        self.caches.insert(file_path.to_string(), cached_tree);
        result
    }

    pub fn invalidate(&mut self, file_path: &str) -> bool {
        self.caches.remove(file_path).is_some()
    }

    pub fn clear_cache(&mut self) {
        self.caches.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.caches.len()
    }

    pub fn is_cached(&self, file_path: &str) -> bool {
        self.caches.contains_key(file_path)
    }

    pub fn diff(&self, old: &str, new: &str) -> DiffResult {
        compute_diff(old, new)
    }

    pub fn cached_tree(&self, file_path: &str) -> Option<&CachedTree> {
        self.caches.get(file_path)
    }
}

impl Default for IncrementalParser {
    fn default() -> Self {
        Self::new()
    }
}

fn hash_content(content: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn compute_line_offsets(source: &str) -> Vec<usize> {
    let mut offsets = vec![0usize];
    for (i, ch) in source.char_indices() {
        if ch == '\n' {
            offsets.push(i + 1);
        }
    }
    offsets
}

pub fn compute_diff(old: &str, new: &str) -> DiffResult {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    let old_hashed: Vec<(usize, String)> = old_lines
        .iter()
        .enumerate()
        .map(|(i, l)| (i, hash_line(l)))
        .collect();

    let new_hashed: Vec<(usize, String)> = new_lines
        .iter()
        .enumerate()
        .map(|(i, l)| (i, hash_line(l)))
        .collect();

    let mut added_ranges = Vec::new();
    let mut removed_ranges = Vec::new();
    let mut modified_ranges = Vec::new();

    let mut oi = 0usize;
    let mut ni = 0usize;

    while oi < old_hashed.len() && ni < new_hashed.len() {
        if old_hashed[oi].1 == new_hashed[ni].1 {
            oi += 1;
            ni += 1;
            continue;
        }

        let fwd = find_forward_match(&old_hashed, oi, &new_hashed, ni);
        let bwd = find_backward_match(&old_hashed, oi, &new_hashed, ni);

        let use_bwd = bwd.0 != 0 && (fwd.0 == 0 || bwd.0 < fwd.0);

        if use_bwd {
            let old_removed_start = oi;
            let new_added_start = ni;
            let new_removed_end = bwd.1;
            let old_removed_end = bwd.2;

            if oi < old_removed_end {
                removed_ranges.push((old_removed_start, old_removed_end));
            }
            if ni < new_removed_end {
                added_ranges.push((new_added_start, new_removed_end));
            }

            for row in oi..old_removed_end {
                modified_ranges.push((row, row));
            }

            oi = old_removed_end;
            ni = new_removed_end;
        } else if fwd.0 != 0 {
            removed_ranges.push((oi, oi + fwd.0));
            added_ranges.push((ni, ni + fwd.0));

            for row in oi..oi + fwd.0 {
                modified_ranges.push((row, row));
            }

            oi += fwd.0;
            ni += fwd.0;
        } else {
            removed_ranges.push((oi, old_hashed.len()));
            added_ranges.push((ni, new_hashed.len()));
            break;
        }
    }

    while oi < old_hashed.len() {
        removed_ranges.push((oi, oi + 1));
        oi += 1;
    }

    while ni < new_hashed.len() {
        added_ranges.push((ni, ni + 1));
        ni += 1;
    }

    DiffResult {
        added_ranges,
        removed_ranges,
        modified_ranges,
    }
}

fn hash_line(line: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    line.hash(&mut hasher);
    format!("{:08x}", hasher.finish())
}

fn find_forward_match(
    old: &[(usize, String)],
    oi: usize,
    new: &[(usize, String)],
    ni: usize,
) -> (usize, usize, usize) {
    let max_look = 8.min(
        old.len()
            .saturating_sub(oi)
            .min(new.len().saturating_sub(ni)),
    );
    for dist in 1..=max_look {
        if old.get(oi + dist).map(|o| &o.1) == new.get(ni + dist).map(|n| &n.1) {
            return (dist, ni + dist, oi + dist);
        }
    }
    (0, 0, 0)
}

fn find_backward_match(
    old: &[(usize, String)],
    oi: usize,
    new: &[(usize, String)],
    ni: usize,
) -> (usize, usize, usize) {
    let remaining_old = old.len().saturating_sub(oi + 1);
    let remaining_new = new.len().saturating_sub(ni + 1);
    let max_look = 8.min(remaining_old.min(remaining_new));
    for dist in 1..=max_look {
        let old_idx = old.len().saturating_sub(dist);
        let new_idx = new.len().saturating_sub(dist);
        if old[old_idx].1 == new[new_idx].1 {
            return (dist, new_idx, old_idx);
        }
    }
    (0, 0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incremental_parse_caches_result() {
        let mut parser = IncrementalParser::new();
        let source = "fn main() {}";
        let result1 = parser.parse_incremental("test.rs", source, "rust");
        assert!(result1.parse_time.as_nanos() > 0);
        assert_eq!(parser.cache_size(), 1);

        let result2 = parser.parse_incremental("test.rs", source, "rust");
        assert_eq!(result2.parse_time, std::time::Duration::ZERO);

        let funcs1: Vec<&str> = result1
            .root
            .iter()
            .filter(|n| n.kind == super::super::parser::TsNodeKind::Function)
            .map(|n| n.name.as_str())
            .collect();
        let funcs2: Vec<&str> = result2
            .root
            .iter()
            .filter(|n| n.kind == super::super::parser::TsNodeKind::Function)
            .map(|n| n.name.as_str())
            .collect();
        assert_eq!(funcs1, funcs2);
    }

    #[test]
    fn test_incremental_parse_reparse_on_change() {
        let mut parser = IncrementalParser::new();
        let old = "fn foo() {}";
        let new = "fn bar() {}";

        parser.parse_incremental("test.rs", old, "rust");
        let result = parser.parse_incremental("test.rs", new, "rust");
        assert!(result.parse_time.as_nanos() > 0);
    }

    #[test]
    fn test_invalidate_cache() {
        let mut parser = IncrementalParser::new();
        parser.parse_incremental("test.rs", "fn x() {}", "rust");
        assert!(parser.is_cached("test.rs"));
        assert!(parser.invalidate("test.rs"));
        assert!(!parser.is_cached("test.rs"));
        assert!(!parser.invalidate("nonexistent"));
    }

    #[test]
    fn test_clear_cache() {
        let mut parser = IncrementalParser::new();
        parser.parse_incremental("a.rs", "fn a() {}", "rust");
        parser.parse_incremental("b.rs", "fn b() {}", "rust");
        assert_eq!(parser.cache_size(), 2);
        parser.clear_cache();
        assert_eq!(parser.cache_size(), 0);
    }

    #[test]
    fn test_separate_files_cached_independently() {
        let mut parser = IncrementalParser::new();
        parser.parse_incremental("a.rs", "fn a() {}", "rust");
        parser.parse_incremental("b.rs", "fn b() {}", "rust");
        assert_eq!(parser.cache_size(), 2);
        assert!(parser.is_cached("a.rs"));
        assert!(parser.is_cached("b.rs"));
    }

    #[test]
    fn test_cached_tree_access() {
        let mut parser = IncrementalParser::new();
        parser.parse_incremental("test.rs", "fn main() {}", "rust");
        let cached = parser.cached_tree("test.rs").unwrap();
        assert!(!cached.source_hash.is_empty());
        assert!(!cached.root.is_empty());
        assert!(!cached.line_offsets.is_empty());
    }

    #[test]
    fn test_diff_identical() {
        let result = compute_diff("fn main() {}", "fn main() {}");
        assert!(result.added_ranges.is_empty());
        assert!(result.removed_ranges.is_empty());
        assert!(result.modified_ranges.is_empty());
    }

    #[test]
    fn test_diff_addition() {
        let result = compute_diff("fn main() {}", "fn main() {}\nfn foo() {}");
        assert!(!result.added_ranges.is_empty());
        assert!(result.removed_ranges.is_empty());
    }

    #[test]
    fn test_diff_removal() {
        let result = compute_diff("fn main() {}\nfn foo() {}", "fn main() {}");
        assert!(result.added_ranges.is_empty());
        assert!(!result.removed_ranges.is_empty());
    }

    #[test]
    fn test_diff_modification() {
        let result = compute_diff("fn foo() {}", "fn bar() {}");
        assert!(!result.modified_ranges.is_empty());
    }

    #[test]
    fn test_diff_empty_old() {
        let result = compute_diff("", "fn main() {}");
        assert!(!result.added_ranges.is_empty());
        assert!(result.removed_ranges.is_empty());
    }

    #[test]
    fn test_diff_empty_new() {
        let result = compute_diff("fn main() {}", "");
        assert!(result.added_ranges.is_empty());
        assert!(!result.removed_ranges.is_empty());
    }

    #[test]
    fn test_hash_content_deterministic() {
        let h1 = hash_content("fn main() {}");
        let h2 = hash_content("fn main() {}");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_content_different() {
        let h1 = hash_content("fn main() {}");
        let h2 = hash_content("fn foo() {}");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_incremental_with_options() {
        let mut parser = IncrementalParser::new();
        let opts = ParseOptions {
            track_comments: true,
            track_whitespace: false,
            max_depth: 32,
        };
        let result1 =
            parser.parse_incremental_with_options("test.rs", "fn main() {}", "rust", opts.clone());
        assert!(result1.parse_time.as_nanos() > 0);

        let result2 =
            parser.parse_incremental_with_options("test.rs", "fn main() {}", "rust", opts);
        assert_eq!(result2.parse_time, std::time::Duration::ZERO);
    }
}
