#![cfg(feature = "treesitter")]
#![allow(unsafe_code)]

//! Tree-sitter C FFI backend for AST parsing.
//!
//! This module is ONLY compiled when the `treesitter` feature is enabled.
//! It uses the tree-sitter C library (compiled at build time via `cc`) to
//! parse source code into CSTs, then extracts semantic nodes.
//!
//! **Requires:** C compiler at build time (gcc, clang, or cc).
//!
//! SAFETY: All `unsafe` blocks interact with tree-sitter's C API through
//! the `tree-sitter` Rust crate which provides safe wrappers. The raw
//! pointer conversions below are bounded by tree-sitter's documented
//! invariants for node lifetimes.

use crate::ast::native_parsers::{ExtractedNode, NativeAstParser, NativeParseResult};
use crate::ast::{Language, NodeKind};
use std::collections::HashMap;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Tree-sitter language wrappers
// ---------------------------------------------------------------------------

macro_rules! ts_language_fn {
    ($func_name:ident, $lang_mod:ident, $language:expr) => {
        fn $func_name() -> tree_sitter::Language {
            // SAFETY: tree-sitter FFI returns a raw pointer (`*mut TSLanguage`)
            // via `tree_sitter_..._language()`. The grammar crate guarantees the
            // pointer is valid and points to a static, zero-cost Language object.
            unsafe { $lang_mod::language() }
        }
    };
}

macro_rules! ts_parse_fn {
    ($func_name:ident, $language:expr) => {
        fn $func_name(content: &str) -> Result<Vec<ExtractedNode>, String> {
            ts_parse_with_lang(content, $language)
        }
    };
}

// Only define language functions for grammars that exist.
// tree-sitter-c and tree-sitter-cpp may not have a compatible version.

#[cfg(feature = "treesitter-python")]
ts_language_fn!(ts_python, tree_sitter_python, Language::Python);
#[cfg(feature = "treesitter-python")]
ts_parse_fn!(ts_parse_python, Language::Python);

#[cfg(feature = "treesitter-go")]
ts_language_fn!(ts_go, tree_sitter_go, Language::Go);
#[cfg(feature = "treesitter-go")]
ts_parse_fn!(ts_parse_go, Language::Go);

#[cfg(feature = "treesitter-c")]
ts_language_fn!(ts_c, tree_sitter_c, Language::C);
#[cfg(feature = "treesitter-c")]
ts_parse_fn!(ts_parse_c, Language::C);

#[cfg(feature = "treesitter-cpp")]
ts_language_fn!(ts_cpp, tree_sitter_cpp, Language::Cpp);
#[cfg(feature = "treesitter-cpp")]
ts_parse_fn!(ts_parse_cpp, Language::Cpp);

#[cfg(feature = "treesitter-java")]
ts_language_fn!(ts_java, tree_sitter_java, Language::Java);
#[cfg(feature = "treesitter-java")]
ts_parse_fn!(ts_parse_java, Language::Java);

#[cfg(feature = "treesitter-bash")]
ts_language_fn!(ts_bash, tree_sitter_bash, Language::Shell);
#[cfg(feature = "treesitter-bash")]
ts_parse_fn!(ts_parse_bash, Language::Shell);

#[cfg(feature = "treesitter-ruby")]
ts_language_fn!(ts_ruby, tree_sitter_ruby, Language::Ruby);
#[cfg(feature = "treesitter-ruby")]
ts_parse_fn!(ts_parse_ruby, Language::Ruby);

#[cfg(feature = "treesitter-php")]
ts_language_fn!(ts_php, tree_sitter_php, Language::Php);
#[cfg(feature = "treesitter-php")]
ts_parse_fn!(ts_parse_php, Language::Php);

#[cfg(feature = "treesitter-swift")]
ts_language_fn!(ts_swift, tree_sitter_swift, Language::Swift);
#[cfg(feature = "treesitter-swift")]
ts_parse_fn!(ts_parse_swift, Language::Swift);

#[cfg(feature = "treesitter-haskell")]
ts_language_fn!(ts_haskell, tree_sitter_haskell, Language::Haskell);
#[cfg(feature = "treesitter-haskell")]
ts_parse_fn!(ts_parse_haskell, Language::Haskell);

#[cfg(feature = "treesitter-scala")]
ts_language_fn!(ts_scala, tree_sitter_scala, Language::Scala);
#[cfg(feature = "treesitter-scala")]
ts_parse_fn!(ts_parse_scala, Language::Scala);

#[cfg(feature = "treesitter-kotlin")]
ts_language_fn!(ts_kotlin, tree_sitter_kotlin, Language::Kotlin);
#[cfg(feature = "treesitter-kotlin")]
ts_parse_fn!(ts_parse_kotlin, Language::Kotlin);

// ---------------------------------------------------------------------------
// Core tree-sitter parse + extraction
// ---------------------------------------------------------------------------

fn ts_parse_with_lang(content: &str, language: Language) -> Result<Vec<ExtractedNode>, String> {
    let lang = ts_language_for(language)?;

    // SAFETY: tree_sitter::Parser is a safe wrapper. The `new` constructor
    // allocates a parser on the heap. The `set_language` method configures
    // it with the provided grammar. Both are documented safe operations.
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&lang)
        .map_err(|e| format!("failed to set language: {e}"))?;

    // SAFETY: parse returns a Tree whose lifetime is bound to the parser.
    // We extract all data from the tree before it is dropped.
    let tree = parser
        .parse(content, None)
        .ok_or("tree-sitter parse returned None")?;

    let root = tree.root_node();
    let mut nodes = Vec::new();
    extract_ts_nodes(&root, &mut nodes, content);

    Ok(nodes)
}

fn ts_language_for(language: Language) -> Result<tree_sitter::Language, String> {
    match language {
        #[cfg(feature = "treesitter-python")]
        Language::Python => Ok(ts_python()),
        #[cfg(feature = "treesitter-go")]
        Language::Go => Ok(ts_go()),
        #[cfg(feature = "treesitter-c")]
        Language::C => Ok(ts_c()),
        #[cfg(feature = "treesitter-cpp")]
        Language::Cpp => Ok(ts_cpp()),
        #[cfg(feature = "treesitter-java")]
        Language::Java => Ok(ts_java()),
        #[cfg(feature = "treesitter-kotlin")]
        Language::Kotlin => Ok(ts_kotlin()),
        #[cfg(feature = "treesitter-bash")]
        Language::Shell => Ok(ts_bash()),
        #[cfg(feature = "treesitter-ruby")]
        Language::Ruby => Ok(ts_ruby()),
        #[cfg(feature = "treesitter-php")]
        Language::Php => Ok(ts_php()),
        #[cfg(feature = "treesitter-swift")]
        Language::Swift => Ok(ts_swift()),
        #[cfg(feature = "treesitter-haskell")]
        Language::Haskell => Ok(ts_haskell()),
        #[cfg(feature = "treesitter-scala")]
        Language::Scala => Ok(ts_scala()),
        _ => Err(format!("no tree-sitter grammar for {language:?}")),
    }
}

/// Recursively extract semantic nodes from tree-sitter CST.
fn extract_ts_nodes(node: &tree_sitter::Node, nodes: &mut Vec<ExtractedNode>, source: &str) {
    let kind_str = node.kind();

    let (node_kind, node_name) = classify_ts_node(kind_str, node, source);

    if let (Some(kind), Some(name)) = (node_kind, node_name) {
        let start_line = node.start_position().row as u32 + 1;
        let end_line = node.end_position().row as u32 + 1;

        let mut extracted = ExtractedNode::leaf(kind, name, start_line, end_line.max(start_line));

        // Recurse into children for nested extraction
        let mut child_nodes = Vec::new();
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                extract_ts_children(&child, &mut child_nodes, source);
            }
        }
        extracted.children = child_nodes;
        nodes.push(extracted);
    } else {
        // Not a semantic node, recurse into children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                extract_ts_nodes(&child, nodes, source);
            }
        }
    }
}

fn extract_ts_children(node: &tree_sitter::Node, nodes: &mut Vec<ExtractedNode>, source: &str) {
    let kind_str = node.kind();
    let (node_kind, node_name) = classify_ts_node(kind_str, node, source);

    if let (Some(kind), Some(name)) = (node_kind, node_name) {
        let start_line = node.start_position().row as u32 + 1;
        let end_line = node.end_position().row as u32 + 1;
        nodes.push(ExtractedNode::leaf(
            kind,
            name,
            start_line,
            end_line.max(start_line),
        ));
    } else {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                extract_ts_children(&child, nodes, source);
            }
        }
    }
}

/// Map tree-sitter node kinds to CivitForge NodeKind.
fn classify_ts_node(
    kind: &str,
    node: &tree_sitter::Node,
    source: &str,
) -> (Option<NodeKind>, Option<String>) {
    match kind {
        // Functions
        "function_declaration"
        | "function_definition"
        | "function_item"
        | "method_declaration"
        | "method_definition"
        | "arrow_function"
        | "generator_function_declaration" => {
            let name = extract_identifier(node, source);
            (Some(NodeKind::Function), name)
        }

        // Classes / Structs
        "class_declaration" | "class_definition" => {
            let name = extract_identifier(node, source);
            (Some(NodeKind::Class), name)
        }
        "struct_declaration" | "struct_definition" => {
            let name = extract_identifier(node, source);
            (Some(NodeKind::Struct), name)
        }

        // Enums
        "enum_declaration" | "enum_definition" => {
            let name = extract_identifier(node, source);
            (Some(NodeKind::Enum), name)
        }

        // Interfaces / Traits
        "interface_declaration"
        | "interface_definition"
        | "trait_declaration"
        | "trait_definition"
        | "protocol_declaration"
        | "protocol_definition" => {
            let name = extract_identifier(node, source);
            (Some(NodeKind::Interface), name)
        }

        // Impl blocks
        "impl_block" | "implementation_declaration" => {
            let name = extract_identifier(node, source);
            (Some(NodeKind::Impl), name)
        }

        // Modules
        "module_declaration" | "namespace_declaration" => {
            let name = extract_identifier(node, source);
            (Some(NodeKind::Module), name)
        }

        // Imports
        "import_statement"
        | "import_declaration"
        | "use_statement"
        | "from_import_statement"
        | "include_directive"
        | "require_statement" => {
            let name = extract_import_path(node, source);
            (Some(NodeKind::Import), name)
        }

        // Type aliases
        "type_alias_declaration" | "type_definition" => {
            let name = extract_identifier(node, source);
            (Some(NodeKind::Class), name)
        }

        _ => (None, None),
    }
}

/// Try to extract an identifier from the first named child.
fn extract_identifier(node: &tree_sitter::Node, source: &str) -> Option<String> {
    for i in 0..node.child_count() {
        let child = node.child(i)?;
        if child.is_named() {
            let kind = child.kind();
            if kind == "identifier" || kind == "type_identifier" || kind == "property_identifier" {
                let start = child.start_byte();
                let end = child.end_byte();
                if end <= source.len() {
                    return Some(source[start..end].to_string());
                }
            }
        }
    }
    None
}

/// Try to extract an import path from the source text.
fn extract_import_path(node: &tree_sitter::Node, source: &str) -> Option<String> {
    let start = node.start_byte();
    let end = node.end_byte();
    let text = source.get(start..end)?;
    // Extract the string literal or path from the import line
    let text = text.trim();
    // Find the path-like portion (usually a string literal or dotted path)
    let mut path = String::new();
    let mut in_string = false;
    for ch in text.chars() {
        match ch {
            '"' | '\'' if !in_string => {
                in_string = true;
            }
            '"' | '\'' if in_string => {
                in_string = false;
            }
            _ if in_string => {
                path.push(ch);
            }
            _ => {}
        }
    }
    if path.is_empty() {
        // Fall back to the entire trimmed line
        Some(text.to_string())
    } else {
        Some(path)
    }
}

// ---------------------------------------------------------------------------
// Unified tree-sitter parser dispatcher
// ---------------------------------------------------------------------------

pub struct TreeSitterDispatcher {
    language_map: HashMap<Language, fn(&str) -> Result<Vec<ExtractedNode>, String>>,
}

impl TreeSitterDispatcher {
    pub fn new() -> Self {
        let mut language_map: HashMap<Language, fn(&str) -> Result<Vec<ExtractedNode>, String>> =
            HashMap::new();

        #[cfg(feature = "treesitter-python")]
        language_map.insert(Language::Python, ts_parse_python);
        #[cfg(feature = "treesitter-go")]
        language_map.insert(Language::Go, ts_parse_go);
        #[cfg(feature = "treesitter-c")]
        language_map.insert(Language::C, ts_parse_c);
        #[cfg(feature = "treesitter-cpp")]
        language_map.insert(Language::Cpp, ts_parse_cpp);
        #[cfg(feature = "treesitter-java")]
        language_map.insert(Language::Java, ts_parse_java);
        #[cfg(feature = "treesitter-kotlin")]
        language_map.insert(Language::Kotlin, ts_parse_kotlin);
        #[cfg(feature = "treesitter-bash")]
        language_map.insert(Language::Shell, ts_parse_bash);
        #[cfg(feature = "treesitter-ruby")]
        language_map.insert(Language::Ruby, ts_parse_ruby);
        #[cfg(feature = "treesitter-php")]
        language_map.insert(Language::Php, ts_parse_php);
        #[cfg(feature = "treesitter-swift")]
        language_map.insert(Language::Swift, ts_parse_swift);
        #[cfg(feature = "treesitter-haskell")]
        language_map.insert(Language::Haskell, ts_parse_haskell);
        #[cfg(feature = "treesitter-scala")]
        language_map.insert(Language::Scala, ts_parse_scala);

        Self { language_map }
    }

    pub fn supported_languages(&self) -> Vec<Language> {
        let mut langs: Vec<Language> = self.language_map.keys().copied().collect();
        langs.sort_by_key(|l| format!("{l:?}"));
        langs
    }

    pub fn can_parse(&self, language: Language) -> bool {
        self.language_map.contains_key(&language)
    }
}

impl Default for TreeSitterDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeAstParser for TreeSitterDispatcher {
    fn parse_native(&self, content: &str) -> NativeParseResult {
        // The dispatcher needs to know which language — but NativeAstParser
        // doesn't pass one. This is used through the router which dispatches
        // by language. Return empty for unrecognized content.
        let start = Instant::now();
        let line_count = content.lines().count() as u32;
        NativeParseResult {
            nodes: Vec::new(),
            line_count,
            parse_time_ms: start.elapsed().as_millis() as u64,
            error: Some("TreeSitterDispatcher: use parse_language() instead".into()),
        }
    }

    fn supported_languages(&self) -> &[Language] {
        // We can't return a static reference here since we build dynamically.
        // Return empty — the router handles dispatch.
        &[]
    }
}

impl TreeSitterDispatcher {
    /// Parse content for a specific language using tree-sitter.
    pub fn parse_language(&self, content: &str, language: Language) -> NativeParseResult {
        let start = Instant::now();
        let line_count = content.lines().count() as u32;

        if let Some(parse_fn) = self.language_map.get(&language) {
            match parse_fn(content) {
                Ok(nodes) => NativeParseResult {
                    nodes,
                    line_count,
                    parse_time_ms: start.elapsed().as_millis() as u64,
                    error: None,
                },
                Err(e) => NativeParseResult {
                    nodes: Vec::new(),
                    line_count,
                    parse_time_ms: start.elapsed().as_millis() as u64,
                    error: Some(e),
                },
            }
        } else {
            NativeParseResult {
                nodes: Vec::new(),
                line_count,
                parse_time_ms: start.elapsed().as_millis() as u64,
                error: Some(format!("no tree-sitter grammar for {language:?}")),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests (only compiled when treesitter feature is active)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatcher_new() {
        let d = TreeSitterDispatcher::new();
        let langs = d.supported_languages();
        // At least some grammars should be available since the feature is enabled
        assert!(!langs.is_empty(), "expected at least 1 tree-sitter grammar");
    }

    #[test]
    fn test_dispatcher_supported_languages_sorted() {
        let d = TreeSitterDispatcher::new();
        let langs = d.supported_languages();
        let mut sorted = langs.clone();
        sorted.sort_by_key(|l| format!("{l:?}"));
        assert_eq!(langs, sorted);
    }

    #[cfg(feature = "treesitter-python")]
    #[test]
    fn test_parse_python_function() {
        let d = TreeSitterDispatcher::new();
        let code = "def hello():\n    print('world')\n\ndef add(a, b):\n    return a + b";
        let result = d.parse_language(code, Language::Python);
        assert!(result.error.is_none(), "error: {:?}", result.error);
        let funcs: Vec<_> = result
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Function)
            .collect();
        assert!(
            funcs.len() >= 1,
            "expected at least 1 function, got {}",
            funcs.len()
        );
        assert!(funcs.iter().any(|f| f.name == "hello"));
    }

    #[cfg(feature = "treesitter-python")]
    #[test]
    fn test_parse_python_class() {
        let d = TreeSitterDispatcher::new();
        let code = "class Animal:\n    pass";
        let result = d.parse_language(code, Language::Python);
        assert!(result.error.is_none(), "error: {:?}", result.error);
        let classes: Vec<_> = result
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Class)
            .collect();
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name, "Animal");
    }

    #[cfg(feature = "treesitter-python")]
    #[test]
    fn test_parse_python_import() {
        let d = TreeSitterDispatcher::new();
        let code = "import os\nfrom sys import path";
        let result = d.parse_language(code, Language::Python);
        assert!(result.error.is_none(), "error: {:?}", result.error);
        let imports: Vec<_> = result
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Import)
            .collect();
        assert!(imports.len() >= 1);
    }

    #[cfg(feature = "treesitter-go")]
    #[test]
    fn test_parse_go_function() {
        let d = TreeSitterDispatcher::new();
        let code = "package main\n\nfunc Add(a int, b int) int {\n\treturn a + b\n}";
        let result = d.parse_language(code, Language::Go);
        assert!(result.error.is_none(), "error: {:?}", result.error);
        let funcs: Vec<_> = result
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Function)
            .collect();
        assert!(funcs.len() >= 1);
        assert!(funcs.iter().any(|f| f.name == "Add"));
    }

    #[cfg(feature = "treesitter-java")]
    #[test]
    fn test_parse_java_class() {
        let d = TreeSitterDispatcher::new();
        let code = "public class Main {\n    public static void main(String[] args) {}\n}";
        let result = d.parse_language(code, Language::Java);
        assert!(result.error.is_none(), "error: {:?}", result.error);
        let classes: Vec<_> = result
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Class)
            .collect();
        assert!(classes.iter().any(|c| c.name == "Main"));
    }

    #[test]
    fn test_parse_unsupported_language() {
        let d = TreeSitterDispatcher::new();
        let result = d.parse_language("print('hello')", Language::Json);
        assert!(result.error.is_some());
        assert!(result.nodes.is_empty());
    }

    #[test]
    fn test_can_parse() {
        let d = TreeSitterDispatcher::new();
        #[cfg(feature = "treesitter-python")]
        assert!(d.can_parse(Language::Python));
        assert!(!d.can_parse(Language::Json));
    }
}
