#![forbid(unsafe_code)]

use crate::ast::Language;
use crate::ast::dispatcher::UnifiedAstParser;
use crate::models::CodeEntity;
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct AstNode {
    pub id: String,
    pub node_type: AstNodeType,
    pub name: String,
    pub line_range: (usize, usize),
    pub children: Vec<AstNode>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AstNodeType {
    File,
    Function,
    Method,
    Struct,
    Enum,
    Trait,
    ImplBlock,
    Module,
    UseStatement,
    Variable,
    Assignment,
    IfExpression,
    LoopExpression,
    MatchExpression,
    CallExpression,
    BinaryExpression,
    Literal,
    Identifier,
    Comment,
    Attribute,
    Unknown,
}

/// Parse engine backed by the tiered `UnifiedAstParser`.
///
/// Supports all 19 registered languages. Routes to the best available
/// backend: native parser (syn/swc/sqlparser/json/toml) > tree-sitter
/// (if feature enabled) > regex fallback.
pub struct ParseEngine {
    /// Track which languages have been explicitly registered (backward compat).
    registered_languages: std::collections::HashSet<String>,
    /// The tiered parser dispatcher.
    unified: UnifiedAstParser,
}

impl ParseEngine {
    pub fn new() -> Self {
        // Register the 3 original languages for backward compatibility.
        // The unified parser handles all 19 regardless; this set controls
        // which languages the `parse()` method accepts.
        let mut registered_languages = std::collections::HashSet::new();
        registered_languages.insert("rust".into());
        registered_languages.insert("python".into());
        registered_languages.insert("go".into());

        Self {
            registered_languages,
            unified: UnifiedAstParser::new(),
        }
    }

    /// Create a ParseEngine that accepts all 19 known languages.
    pub fn new_with_all_languages() -> Self {
        Self {
            registered_languages: std::collections::HashSet::new(),
            unified: UnifiedAstParser::new(),
        }
    }

    /// Register an additional language as accepted by `parse()`.
    pub fn register_language(&mut self, language: &str) {
        self.registered_languages.insert(language.to_string());
    }

    /// Query which backend would be used for a language.
    pub fn backend_for(&self, language: &str) -> &'static str {
        let lang = match Language::grammar_name_to_enum(language) {
            Some(l) => l,
            None => return "none",
        };
        self.unified.backend_for(lang)
    }

    /// List all language names the unified dispatcher can handle.
    pub fn supported_language_names(&self) -> Vec<String> {
        self.unified.supported_language_names()
    }
}

impl Default for ParseEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ParseEngine {
    /// Parse source code into AST nodes using the best available backend.
    ///
    /// Returns an error only if the language is not registered.
    /// Parse errors within the source return an empty node list (the
    /// dispatcher tries all available backends before falling through).
    pub fn parse(&self, source: &str, language: &str) -> anyhow::Result<Vec<AstNode>> {
        // Backward compat: reject unregistered languages
        if !self.registered_languages.is_empty() && !self.registered_languages.contains(language) {
            anyhow::bail!("unsupported language: {language}");
        }

        let lang = Language::grammar_name_to_enum(language)
            .ok_or_else(|| anyhow::anyhow!("unknown language: {language}"))?;

        let result = self.unified.parse(source, lang);

        debug!(
            language = %language,
            backend = %result.backend,
            nodes = result.nodes.len(),
            error = ?result.error,
            "parsed source"
        );

        Ok(result.to_engine_nodes())
    }

    /// Convert AST nodes to CodeEntity format for downstream consumers.
    pub fn nodes_to_entities(&self, nodes: &[AstNode], file_path: &str) -> Vec<CodeEntity> {
        nodes
            .iter()
            .map(|n| CodeEntity {
                id: n.id.clone(),
                entity_type: format!("{:?}", n.node_type),
                name: n.name.clone(),
                file_path: file_path.into(),
                start_line: n.line_range.0,
                end_line: n.line_range.1,
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rust_functions() {
        let engine = ParseEngine::new();
        let code = "fn hello() {}\nfn world() {}";
        let nodes = engine.parse(code, "rust").unwrap();
        let funcs: Vec<&AstNode> = nodes
            .iter()
            .filter(|n| n.node_type == AstNodeType::Function)
            .collect();
        assert!(!funcs.is_empty());
        assert_eq!(funcs[0].name, "hello");
    }

    #[test]
    fn test_parse_struct() {
        let engine = ParseEngine::new();
        let code = "struct Point {\n    x: i32,\n    y: i32,\n}";
        let nodes = engine.parse(code, "rust").unwrap();
        let types: Vec<&AstNodeType> = nodes.iter().map(|n| &n.node_type).collect();
        assert!(types.contains(&&AstNodeType::Struct));
    }

    #[test]
    fn test_parse_enum() {
        let engine = ParseEngine::new();
        let code = "enum Color {\n    Red,\n    Blue,\n}";
        let nodes = engine.parse(code, "rust").unwrap();
        let types: Vec<&AstNodeType> = nodes.iter().map(|n| &n.node_type).collect();
        assert!(types.contains(&&AstNodeType::Enum));
    }

    #[test]
    fn test_parse_unsupported_language() {
        let engine = ParseEngine::new();
        let result = engine.parse("print('hello')", "brainfuck");
        assert!(result.is_err());
    }

    #[test]
    fn test_nodes_to_entities() {
        let engine = ParseEngine::new();
        let nodes = vec![AstNode {
            id: "n1".into(),
            node_type: AstNodeType::Function,
            name: "main".into(),
            line_range: (1, 5),
            children: vec![],
            metadata: HashMap::new(),
        }];
        let entities = engine.nodes_to_entities(&nodes, "src/main.rs");
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].name, "main");
        assert_eq!(entities[0].file_path, "src/main.rs");
    }

    #[test]
    fn test_parse_variables_and_loops() {
        let engine = ParseEngine::new();
        let code = "let x = 42;\nfor i in 0..10 {}\nwhile true {}";
        let nodes = engine.parse(code, "rust").unwrap();
        let types: Vec<AstNodeType> = nodes.iter().map(|n| n.node_type).collect();
        assert!(
            types
                .iter()
                .any(|t| matches!(t, AstNodeType::LoopExpression | AstNodeType::Variable))
        );
    }

    // --- New tests for the unified backend ---

    #[test]
    fn test_parse_json_via_native_backend() {
        let mut engine = ParseEngine::new();
        engine.register_language("json");
        let code = r#"{"name": "Alice", "age": 30}"#;
        let nodes = engine.parse(code, "json").unwrap();
        // JSON fields are extracted as Identifier/Variable nodes
        assert!(nodes.iter().any(|n| n.name == "name"));
        assert!(nodes.iter().any(|n| n.name == "age"));
    }

    #[test]
    fn test_parse_toml_via_native_backend() {
        let mut engine = ParseEngine::new();
        engine.register_language("toml");
        let code = "[package]\nname = \"civitforge\"\nversion = \"0.1.0\"";
        let nodes = engine.parse(code, "toml").unwrap();
        assert!(nodes.iter().any(|n| n.name == "package"));
        assert!(nodes.iter().any(|n| n.name == "name"));
        assert!(nodes.iter().any(|n| n.name == "version"));
    }

    #[test]
    fn test_new_with_all_languages_accepts_python() {
        let engine = ParseEngine::new_with_all_languages();
        let code = "def hello():\n    print('world')";
        let nodes = engine.parse(code, "python").unwrap();
        let funcs: Vec<&AstNode> = nodes
            .iter()
            .filter(|n| n.node_type == AstNodeType::Function)
            .collect();
        assert!(funcs.iter().any(|f| f.name == "hello"));
    }

    #[test]
    fn test_new_with_all_languages_accepts_go() {
        let engine = ParseEngine::new_with_all_languages();
        let code = "package main\n\nfunc Add(a int, b int) int { return a + b }";
        let nodes = engine.parse(code, "go").unwrap();
        let funcs: Vec<&AstNode> = nodes
            .iter()
            .filter(|n| n.node_type == AstNodeType::Function)
            .collect();
        assert!(funcs.iter().any(|f| f.name == "Add"));
    }

    #[test]
    fn test_backend_for_rust() {
        let engine = ParseEngine::new();
        // Without syn-parser feature, Rust uses regex
        assert!(engine.backend_for("rust") == "regex" || engine.backend_for("rust") == "native");
    }

    #[test]
    fn test_backend_for_json() {
        let engine = ParseEngine::new();
        assert_eq!(engine.backend_for("json"), "native");
    }

    #[test]
    fn test_supported_language_names() {
        let engine = ParseEngine::new();
        let langs = engine.supported_language_names();
        assert!(langs.len() >= 19);
        assert!(langs.contains(&"json".into()));
        assert!(langs.contains(&"toml".into()));
        assert!(langs.contains(&"rust".into()));
    }

    #[test]
    fn test_register_language_expansion() {
        let mut engine = ParseEngine::new();
        assert!(engine.parse("SELECT 1", "sql").is_err());
        engine.register_language("sql");
        // Should succeed now
        let result = engine.parse("SELECT 1", "sql");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_empty_source() {
        let engine = ParseEngine::new();
        let result = engine.parse("", "rust").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_impl_block() {
        let engine = ParseEngine::new();
        let code = "impl Point {\n    fn new() -> Self { Self { x: 0, y: 0 } }\n}";
        let nodes = engine.parse(code, "rust").unwrap();
        let impls: Vec<&AstNode> = nodes
            .iter()
            .filter(|n| n.node_type == AstNodeType::ImplBlock)
            .collect();
        assert!(impls.iter().any(|i| i.name == "Point"));
    }

    #[test]
    fn test_parse_use_statement() {
        let engine = ParseEngine::new();
        let code = "use std::collections::HashMap;\nuse anyhow::Result;";
        let nodes = engine.parse(code, "rust").unwrap();
        let imports: Vec<&AstNode> = nodes
            .iter()
            .filter(|n| n.node_type == AstNodeType::UseStatement)
            .collect();
        assert!(!imports.is_empty());
    }
}
