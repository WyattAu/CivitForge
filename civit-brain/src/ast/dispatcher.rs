#![forbid(unsafe_code)]

//! Unified AST parser dispatcher.
//!
//! Routes parsing requests through three tiers:
//!
//! 1. **Tier 1 — Native parsers** (syn, swc, sqlparser, serde_json, toml):
//!    Zero unsafe, production quality. Available when their feature flags are enabled.
//!
//! 2. **Tier 2 — Tree-sitter** (C FFI):
//!    Available when `treesitter` feature is enabled. Requires C compiler at build time.
//!
//! 3. **Tier 3 — Regex fallback** (RegexAstParser):
//!    Always available. Best-effort line-pattern matching. No actual parsing.
//!
//! The dispatcher automatically selects the highest-quality available parser
//! for each language.

use crate::ast::engine::AstNode;
use crate::ast::engine::AstNodeType;
use crate::ast::native_parsers::{NativeAstParser, all_native_parsers};
use crate::ast::{
    AstNode as PublicAstNode, AstParser, Language, NodeKind, ParseResult, RegexAstParser,
};
use std::time::Instant;
use tracing::debug;

#[cfg(feature = "treesitter")]
use crate::ast::treesitter_backend::TreeSitterDispatcher;

/// The unified parser that routes to the best available backend.
pub struct UnifiedAstParser {
    /// Language names that have a Tier 1 native parser available
    native_languages: Vec<String>,
    /// Tier 1: Native parser instances (indexed by language name)
    /// We store Option<Box<dyn NativeAstParser>> to allow multi-language parsers
    /// to be referenced by multiple languages but only created once.
    native_parser_instances: Vec<Box<dyn NativeAstParser>>,
    /// Tier 2: Tree-sitter dispatcher (only present with feature)
    #[cfg(feature = "treesitter")]
    ts_dispatcher: TreeSitterDispatcher,
    /// Tier 3: Regex fallback (always available)
    regex_parser: RegexAstParser,
}

impl UnifiedAstParser {
    pub fn new() -> Self {
        let native_parsers = all_native_parsers();
        let mut native_languages = Vec::new();

        for parser in &native_parsers {
            for lang in parser.supported_languages() {
                native_languages.push(lang.grammar_name().to_string());
            }
        }

        // Deduplicate while preserving order
        let mut seen = std::collections::HashSet::new();
        native_languages.retain(|l| seen.insert(l.clone()));

        Self {
            native_languages,
            native_parser_instances: native_parsers,
            #[cfg(feature = "treesitter")]
            ts_dispatcher: TreeSitterDispatcher::new(),
            regex_parser: RegexAstParser::new(),
        }
    }

    /// Parse source content for a given language using the best available backend.
    pub fn parse(&self, content: &str, language: Language) -> UnifiedParseResult {
        let lang_name = language.grammar_name();

        // Tier 1: Try native parser
        if self.native_languages.contains(&lang_name.to_string()) {
            for parser in &self.native_parser_instances {
                if parser
                    .supported_languages()
                    .iter()
                    .any(|l| l.grammar_name() == lang_name)
                {
                    let result = parser.parse_native(content);
                    if result.error.is_none() {
                        debug!(
                            language = lang_name,
                            backend = "native",
                            nodes = result.nodes.len(),
                            "parsed source"
                        );
                        return UnifiedParseResult::from_native(result);
                    }
                    debug!(language = lang_name, backend = "native", error = ?result.error, "native parser failed, trying next tier");
                }
            }
        }

        // Tier 2: Try tree-sitter (if feature enabled)
        #[cfg(feature = "treesitter")]
        if self.ts_dispatcher.can_parse(language) {
            let result = self.ts_dispatcher.parse_language(content, language);
            if result.error.is_none() {
                debug!(
                    language = lang_name,
                    backend = "tree-sitter",
                    nodes = result.nodes.len(),
                    "parsed source"
                );
                return UnifiedParseResult::from_native(result);
            }
            debug!(language = lang_name, backend = "tree-sitter", error = ?result.error, "tree-sitter parser failed, falling back to regex");
        }

        // Tier 3: Regex fallback
        debug!(
            language = lang_name,
            backend = "regex",
            "using regex fallback"
        );
        let start = Instant::now();
        match self.regex_parser.parse(content.as_bytes(), language) {
            Ok(result) => {
                let nodes = result
                    .nodes
                    .into_iter()
                    .map(|n| crate::ast::native_parsers::ExtractedNode {
                        kind: n.kind,
                        name: n.name,
                        start_line: n.start_line,
                        end_line: n.end_line,
                        children: n
                            .children
                            .into_iter()
                            .map(|c| crate::ast::native_parsers::ExtractedNode {
                                kind: c.kind,
                                name: c.name,
                                start_line: c.start_line,
                                end_line: c.end_line,
                                children: Vec::new(),
                                body_text: None,
                            })
                            .collect(),
                        body_text: None,
                    })
                    .collect();
                UnifiedParseResult {
                    nodes,
                    line_count: result.line_count,
                    parse_time_ms: start.elapsed().as_millis() as u64,
                    error: None,
                    backend: "regex".into(),
                }
            }
            Err(e) => UnifiedParseResult {
                nodes: Vec::new(),
                line_count: content.lines().count() as u32,
                parse_time_ms: start.elapsed().as_millis() as u64,
                error: Some(e),
                backend: "regex".into(),
            },
        }
    }

    /// List all languages that have a native (Tier 1) parser.
    pub fn native_language_names(&self) -> &[String] {
        &self.native_languages
    }

    /// List all languages that have a tree-sitter (Tier 2) parser.
    #[cfg(feature = "treesitter")]
    pub fn treesitter_language_names(&self) -> Vec<String> {
        self.ts_dispatcher
            .supported_languages()
            .iter()
            .map(|l| l.grammar_name().to_string())
            .collect()
    }

    #[cfg(not(feature = "treesitter"))]
    pub fn treesitter_language_names(&self) -> Vec<String> {
        Vec::new()
    }

    /// List all languages that have at least one parser (any tier).
    pub fn supported_language_names(&self) -> Vec<String> {
        let mut langs = std::collections::HashSet::new();
        for lang in &self.native_languages {
            langs.insert(lang.clone());
        }
        for lang in self.treesitter_language_names() {
            langs.insert(lang);
        }
        // Regex parser supports all 19 languages
        for lang in self.regex_parser.supported_languages() {
            langs.insert(lang.grammar_name().to_string());
        }
        let mut result: Vec<String> = langs.into_iter().collect();
        result.sort();
        result
    }

    /// Report which backend would be used for a given language.
    pub fn backend_for(&self, language: Language) -> &'static str {
        let lang_name = language.grammar_name();

        if self.native_languages.iter().any(|l| l == lang_name) {
            return "native";
        }

        #[cfg(feature = "treesitter")]
        if self.ts_dispatcher.can_parse(language) {
            return "tree-sitter";
        }

        "regex"
    }
}

impl Default for UnifiedAstParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Unified parse result from any backend.
#[derive(Debug)]
pub struct UnifiedParseResult {
    pub nodes: Vec<crate::ast::native_parsers::ExtractedNode>,
    pub line_count: u32,
    pub parse_time_ms: u64,
    pub error: Option<String>,
    pub backend: String,
}

impl UnifiedParseResult {
    fn from_native(result: crate::ast::native_parsers::NativeParseResult) -> Self {
        Self {
            nodes: result.nodes,
            line_count: result.line_count,
            parse_time_ms: result.parse_time_ms,
            error: result.error,
            backend: "native".into(),
        }
    }

    /// Convert to the legacy ParseResult format for backward compatibility.
    pub fn to_legacy_parse_result(&self, language: Language, file_path: &str) -> ParseResult {
        ParseResult {
            language,
            file_path: file_path.into(),
            nodes: self
                .nodes
                .iter()
                .map(|n| PublicAstNode {
                    id: 0,
                    kind: n.kind,
                    name: n.name.clone(),
                    start_line: n.start_line,
                    end_line: n.end_line,
                    children: Vec::new(),
                    complexity: None,
                })
                .collect(),
            line_count: self.line_count,
            parse_time_ms: self.parse_time_ms,
            error: self.error.clone(),
        }
    }

    /// Convert to engine::AstNode format for ParseEngine compatibility.
    pub fn to_engine_nodes(&self) -> Vec<AstNode> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(i, n)| AstNode {
                id: format!("node-{i}"),
                node_type: node_kind_to_engine_type(n.kind),
                name: n.name.clone(),
                line_range: (n.start_line as usize, n.end_line as usize),
                children: Vec::new(),
                metadata: std::collections::HashMap::new(),
            })
            .collect()
    }
}

/// Map our NodeKind to engine::AstNodeType.
fn node_kind_to_engine_type(kind: NodeKind) -> AstNodeType {
    match kind {
        NodeKind::Function | NodeKind::Method => AstNodeType::Function,
        NodeKind::Struct => AstNodeType::Struct,
        NodeKind::Enum => AstNodeType::Enum,
        NodeKind::Interface => AstNodeType::Trait,
        NodeKind::Class => AstNodeType::Struct,
        NodeKind::Impl => AstNodeType::ImplBlock,
        NodeKind::Module => AstNodeType::Module,
        NodeKind::Import => AstNodeType::UseStatement,
        NodeKind::Variable => AstNodeType::Variable,
        NodeKind::Comment => AstNodeType::Comment,
        NodeKind::Loop => AstNodeType::LoopExpression,
        NodeKind::Condition => AstNodeType::IfExpression,
        _ => AstNodeType::Unknown,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_parser_new() {
        let _parser = UnifiedAstParser::new();
    }

    #[test]
    fn test_unified_parser_default() {
        let _parser = UnifiedAstParser::default();
    }

    #[test]
    fn test_unified_supported_languages_includes_json_toml() {
        let parser = UnifiedAstParser::new();
        let langs = parser.supported_language_names();
        assert!(langs.contains(&"json".into()));
        assert!(langs.contains(&"toml".into()));
    }

    #[test]
    fn test_unified_supported_languages_includes_regex_all() {
        let parser = UnifiedAstParser::new();
        let langs = parser.supported_language_names();
        // Regex fallback supports 19 languages, so we should have at least that many
        assert!(
            langs.len() >= 19,
            "expected at least 19 languages, got {}",
            langs.len()
        );
    }

    #[test]
    fn test_unified_parse_json_native() {
        let parser = UnifiedAstParser::new();
        let code = r#"{"name": "Alice"}"#;
        let result = parser.parse(code, Language::Json);
        assert!(result.error.is_none(), "error: {:?}", result.error);
        assert_eq!(result.backend, "native");
        assert!(result.nodes.iter().any(|n| n.name == "name"));
    }

    #[test]
    fn test_unified_parse_toml_native() {
        let parser = UnifiedAstParser::new();
        let code = "[package]\nname = \"foo\"";
        let result = parser.parse(code, Language::Toml);
        assert!(result.error.is_none(), "error: {:?}", result.error);
        assert_eq!(result.backend, "native");
        assert!(result.nodes.iter().any(|n| n.name == "package"));
    }

    #[test]
    fn test_unified_parse_rust_falls_to_regex() {
        let parser = UnifiedAstParser::new();
        // Without syn-parser feature, Rust should fall to regex
        let code = "fn main() {}";
        let result = parser.parse(code, Language::Rust);
        assert!(result.error.is_none(), "error: {:?}", result.error);
        assert_eq!(result.backend, "regex");
    }

    #[test]
    fn test_unified_parse_python_falls_to_regex() {
        let parser = UnifiedAstParser::new();
        // Without treesitter feature, Python should fall to regex
        let code = "def hello(): pass";
        let result = parser.parse(code, Language::Python);
        assert!(result.error.is_none(), "error: {:?}", result.error);
        assert_eq!(result.backend, "regex");
    }

    #[test]
    fn test_unified_parse_empty() {
        let parser = UnifiedAstParser::new();
        let result = parser.parse("", Language::Rust);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_unified_backend_for_json() {
        let parser = UnifiedAstParser::new();
        assert_eq!(parser.backend_for(Language::Json), "native");
        assert_eq!(parser.backend_for(Language::Toml), "native");
    }

    #[test]
    fn test_unified_backend_for_rust_no_features() {
        let parser = UnifiedAstParser::new();
        // Without syn-parser feature, Rust should use regex
        assert_eq!(parser.backend_for(Language::Rust), "regex");
    }

    #[test]
    fn test_to_legacy_parse_result() {
        let result = UnifiedParseResult {
            nodes: vec![crate::ast::native_parsers::ExtractedNode::leaf(
                NodeKind::Function,
                "main".into(),
                1,
                5,
            )],
            line_count: 10,
            parse_time_ms: 1,
            error: None,
            backend: "test".into(),
        };
        let legacy = result.to_legacy_parse_result(Language::Rust, "test.rs");
        assert_eq!(legacy.language, Language::Rust);
        assert_eq!(legacy.file_path, "test.rs");
        assert_eq!(legacy.nodes.len(), 1);
        assert_eq!(legacy.nodes[0].name, "main");
        assert_eq!(legacy.line_count, 10);
    }

    #[test]
    fn test_to_engine_nodes() {
        let result = UnifiedParseResult {
            nodes: vec![crate::ast::native_parsers::ExtractedNode::leaf(
                NodeKind::Struct,
                "Config".into(),
                3,
                10,
            )],
            line_count: 15,
            parse_time_ms: 2,
            error: None,
            backend: "test".into(),
        };
        let engine_nodes = result.to_engine_nodes();
        assert_eq!(engine_nodes.len(), 1);
        assert_eq!(engine_nodes[0].name, "Config");
        assert_eq!(engine_nodes[0].node_type, AstNodeType::Struct);
        assert_eq!(engine_nodes[0].line_range, (3, 10));
    }

    #[test]
    fn test_node_kind_to_engine_type_mapping() {
        assert_eq!(
            node_kind_to_engine_type(NodeKind::Function),
            AstNodeType::Function
        );
        assert_eq!(
            node_kind_to_engine_type(NodeKind::Method),
            AstNodeType::Function
        );
        assert_eq!(
            node_kind_to_engine_type(NodeKind::Struct),
            AstNodeType::Struct
        );
        assert_eq!(node_kind_to_engine_type(NodeKind::Enum), AstNodeType::Enum);
        assert_eq!(
            node_kind_to_engine_type(NodeKind::Interface),
            AstNodeType::Trait
        );
        assert_eq!(
            node_kind_to_engine_type(NodeKind::Class),
            AstNodeType::Struct
        );
        assert_eq!(
            node_kind_to_engine_type(NodeKind::Import),
            AstNodeType::UseStatement
        );
        assert_eq!(
            node_kind_to_engine_type(NodeKind::Module),
            AstNodeType::Module
        );
        assert_eq!(
            node_kind_to_engine_type(NodeKind::Unknown),
            AstNodeType::Unknown
        );
    }

    #[test]
    fn test_unified_native_language_names() {
        let parser = UnifiedAstParser::new();
        let native = parser.native_language_names();
        assert!(native.contains(&"json".into()));
        assert!(native.contains(&"toml".into()));
    }

    #[test]
    fn test_unified_toml_parse_table_with_fields() {
        let parser = UnifiedAstParser::new();
        let code = r#"[workspace.dependencies]
tokio = "1"
serde = { version = "1", features = ["derive"] }
"#;
        let result = parser.parse(code, Language::Toml);
        assert!(result.error.is_none(), "error: {:?}", result.error);
        // TOML dotted keys create nested tables: workspace -> dependencies -> tokio
        assert!(result.nodes.iter().any(|n| n.name == "workspace"));
        assert!(result.nodes.iter().any(|n| n.name == "dependencies"));
        assert!(result.nodes.iter().any(|n| n.name == "tokio"));
        assert!(result.nodes.iter().any(|n| n.name == "serde"));
    }

    #[test]
    fn test_unified_json_parse_array() {
        let parser = UnifiedAstParser::new();
        let code = r#"[{"id": 1}, {"id": 2}]"#;
        let result = parser.parse(code, Language::Json);
        assert!(result.error.is_none(), "error: {:?}", result.error);
        let ids: Vec<_> = result.nodes.iter().filter(|n| n.name == "id").collect();
        assert_eq!(ids.len(), 2);
    }
}
