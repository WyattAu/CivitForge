#![forbid(unsafe_code)]

pub mod engine;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Rust,
    Go,
    Python,
    TypeScript,
    C,
    Cpp,
    Java,
    Kotlin,
    Swift,
    JavaScript,
}

impl Language {
    pub fn grammar_name(&self) -> &str {
        match self {
            Language::Rust => "rust",
            Language::Go => "go",
            Language::Python => "python",
            Language::TypeScript => "typescript",
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::Java => "java",
            Language::Kotlin => "kotlin",
            Language::Swift => "swift",
            Language::JavaScript => "javascript",
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Language::Rust => ".rs",
            Language::Go => ".go",
            Language::Python => ".py",
            Language::TypeScript => ".ts",
            Language::C => ".c",
            Language::Cpp => ".cpp",
            Language::Java => ".java",
            Language::Kotlin => ".kt",
            Language::Swift => ".swift",
            Language::JavaScript => ".js",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    Function,
    Class,
    Struct,
    Enum,
    Interface,
    Impl,
    Method,
    Field,
    Import,
    Variable,
    Module,
    Comment,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode {
    pub id: u64,
    pub kind: NodeKind,
    pub name: String,
    pub start_line: u32,
    pub end_line: u32,
    pub children: Vec<AstNode>,
    pub complexity: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    pub language: Language,
    pub file_path: String,
    pub nodes: Vec<AstNode>,
    pub line_count: u32,
    pub parse_time_ms: u64,
    pub error: Option<String>,
}

pub trait AstParser: Send + Sync {
    fn parse(&self, content: &[u8], language: Language) -> Result<ParseResult, String>;
    fn supported_languages(&self) -> Vec<Language>;
}

pub struct RegexAstParser;

impl RegexAstParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RegexAstParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AstParser for RegexAstParser {
    fn parse(&self, content: &[u8], language: Language) -> Result<ParseResult, String> {
        let text = String::from_utf8_lossy(content);
        let lines: Vec<&str> = text.lines().collect();
        let mut nodes = Vec::new();

        let patterns: &[(&str, NodeKind)] = match language {
            Language::Rust => &[
                ("fn ", NodeKind::Function),
                ("pub struct ", NodeKind::Struct),
                ("pub enum ", NodeKind::Enum),
                ("impl ", NodeKind::Impl),
                ("use ", NodeKind::Import),
            ],
            Language::Python => &[
                ("def ", NodeKind::Function),
                ("class ", NodeKind::Class),
                ("import ", NodeKind::Import),
            ],
            Language::Go => &[
                ("func ", NodeKind::Function),
                ("type ", NodeKind::Struct),
                ("package ", NodeKind::Module),
            ],
            _ => &[
                ("fn ", NodeKind::Function),
                ("class ", NodeKind::Class),
                ("import ", NodeKind::Import),
            ],
        };

        let start = std::time::Instant::now();
        let mut node_id: u64 = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            for (pattern, kind) in patterns.iter() {
                if trimmed.starts_with(pattern) {
                    let rest = trimmed.trim_start_matches(pattern);
                    let name: String = rest
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    node_id += 1;
                    nodes.push(AstNode {
                        id: node_id,
                        kind: *kind,
                        name,
                        start_line: (i + 1) as u32,
                        end_line: (i + 1) as u32,
                        children: Vec::new(),
                        complexity: None,
                    });
                    break;
                }
            }
        }

        let parse_time_ms = start.elapsed().as_millis() as u64;

        Ok(ParseResult {
            language,
            file_path: String::new(),
            nodes,
            line_count: lines.len() as u32,
            parse_time_ms,
            error: None,
        })
    }

    fn supported_languages(&self) -> Vec<Language> {
        vec![
            Language::Rust,
            Language::Go,
            Language::Python,
            Language::TypeScript,
            Language::C,
            Language::Cpp,
            Language::Java,
            Language::Kotlin,
            Language::Swift,
            Language::JavaScript,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_grammar_name_all() {
        assert_eq!(Language::Rust.grammar_name(), "rust");
        assert_eq!(Language::Go.grammar_name(), "go");
        assert_eq!(Language::Python.grammar_name(), "python");
        assert_eq!(Language::TypeScript.grammar_name(), "typescript");
        assert_eq!(Language::C.grammar_name(), "c");
        assert_eq!(Language::Cpp.grammar_name(), "cpp");
        assert_eq!(Language::Java.grammar_name(), "java");
        assert_eq!(Language::Kotlin.grammar_name(), "kotlin");
        assert_eq!(Language::Swift.grammar_name(), "swift");
        assert_eq!(Language::JavaScript.grammar_name(), "javascript");
    }

    #[test]
    fn test_language_extension_all() {
        assert_eq!(Language::Rust.extension(), ".rs");
        assert_eq!(Language::Go.extension(), ".go");
        assert_eq!(Language::Python.extension(), ".py");
        assert_eq!(Language::TypeScript.extension(), ".ts");
        assert_eq!(Language::C.extension(), ".c");
        assert_eq!(Language::Cpp.extension(), ".cpp");
        assert_eq!(Language::Java.extension(), ".java");
        assert_eq!(Language::Kotlin.extension(), ".kt");
        assert_eq!(Language::Swift.extension(), ".swift");
        assert_eq!(Language::JavaScript.extension(), ".js");
    }

    #[test]
    fn test_language_hash_equality() {
        let mut set = std::collections::HashSet::new();
        set.insert(Language::Rust);
        assert!(set.contains(&Language::Rust));
        assert!(!set.contains(&Language::Go));
    }

    #[test]
    fn test_node_kind_clone_copy() {
        let kind = NodeKind::Function;
        let copied = kind;
        assert_eq!(kind, copied);
    }

    #[test]
    fn test_node_kind_eq() {
        assert_eq!(NodeKind::Struct, NodeKind::Struct);
        assert_ne!(NodeKind::Function, NodeKind::Struct);
    }

    #[test]
    fn test_node_kind_serialization() {
        let kind = NodeKind::Function;
        let json = serde_json::to_string(&kind).unwrap();
        let de: NodeKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, de);
    }

    #[test]
    fn test_ast_node_creation() {
        let node = AstNode {
            id: 1,
            kind: NodeKind::Function,
            name: "main".into(),
            start_line: 1,
            end_line: 10,
            children: Vec::new(),
            complexity: Some(2.5),
        };
        assert_eq!(node.id, 1);
        assert_eq!(node.name, "main");
        assert_eq!(node.children.len(), 0);
        assert_eq!(node.complexity, Some(2.5));
    }

    #[test]
    fn test_ast_node_with_children() {
        let child = AstNode {
            id: 2,
            kind: NodeKind::Variable,
            name: "x".into(),
            start_line: 2,
            end_line: 2,
            children: Vec::new(),
            complexity: None,
        };
        let node = AstNode {
            id: 1,
            kind: NodeKind::Function,
            name: "main".into(),
            start_line: 1,
            end_line: 10,
            children: vec![child],
            complexity: Some(1.0),
        };
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].name, "x");
    }

    #[test]
    fn test_parse_result_creation() {
        let result = ParseResult {
            language: Language::Rust,
            file_path: "test.rs".into(),
            nodes: Vec::new(),
            line_count: 5,
            parse_time_ms: 1,
            error: None,
        };
        assert_eq!(result.language, Language::Rust);
        assert_eq!(result.line_count, 5);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_parse_result_with_error() {
        let result = ParseResult {
            language: Language::Python,
            file_path: "test.py".into(),
            nodes: Vec::new(),
            line_count: 0,
            parse_time_ms: 0,
            error: Some("syntax error".into()),
        };
        assert!(result.error.is_some());
    }

    #[test]
    fn test_regex_parser_new() {
        let _parser = RegexAstParser::new();
    }

    #[test]
    fn test_regex_parser_default_trait() {
        let _parser: Box<dyn AstParser> = Box::new(RegexAstParser);
        let langs = _parser.supported_languages();
        assert!(!langs.is_empty());
    }

    #[test]
    fn test_regex_parse_rust_function() {
        let parser = RegexAstParser::new();
        let code =
            b"fn main() {\n    println!(\"hello\");\n}\nfn add(a: i32, b: i32) -> i32 { a + b }";
        let result = parser.parse(code, Language::Rust).unwrap();
        let funcs: Vec<&AstNode> = result
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Function)
            .collect();
        assert_eq!(funcs.len(), 2);
        assert_eq!(funcs[0].name, "main");
        assert_eq!(funcs[1].name, "add");
    }

    #[test]
    fn test_regex_parse_rust_struct() {
        let parser = RegexAstParser::new();
        let code = b"pub struct Point {\n    x: i32,\n    y: i32,\n}";
        let result = parser.parse(code, Language::Rust).unwrap();
        let structs: Vec<&AstNode> = result
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Struct)
            .collect();
        assert_eq!(structs.len(), 1);
        assert_eq!(structs[0].name, "Point");
    }

    #[test]
    fn test_regex_parse_rust_enum() {
        let parser = RegexAstParser::new();
        let code = b"pub enum Color { Red, Green, Blue }";
        let result = parser.parse(code, Language::Rust).unwrap();
        let enums: Vec<&AstNode> = result
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Enum)
            .collect();
        assert_eq!(enums.len(), 1);
        assert_eq!(enums[0].name, "Color");
    }

    #[test]
    fn test_regex_parse_rust_impl() {
        let parser = RegexAstParser::new();
        let code = b"impl Point {\n    fn new() -> Self { Self { x: 0, y: 0 } }\n}";
        let result = parser.parse(code, Language::Rust).unwrap();
        let impls: Vec<&AstNode> = result
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Impl)
            .collect();
        assert_eq!(impls.len(), 1);
    }

    #[test]
    fn test_regex_parse_rust_import() {
        let parser = RegexAstParser::new();
        let code = b"use std::collections::HashMap;\nuse anyhow::Result;";
        let result = parser.parse(code, Language::Rust).unwrap();
        let imports: Vec<&AstNode> = result
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Import)
            .collect();
        assert_eq!(imports.len(), 2);
    }

    #[test]
    fn test_regex_parse_python_def() {
        let parser = RegexAstParser::new();
        let code = b"def hello():\n    print('world')\n\ndef add(a, b):\n    return a + b";
        let result = parser.parse(code, Language::Python).unwrap();
        let funcs: Vec<&AstNode> = result
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Function)
            .collect();
        assert_eq!(funcs.len(), 2);
        assert_eq!(funcs[0].name, "hello");
    }

    #[test]
    fn test_regex_parse_python_class() {
        let parser = RegexAstParser::new();
        let code = b"class Animal:\n    pass";
        let result = parser.parse(code, Language::Python).unwrap();
        let classes: Vec<&AstNode> = result
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Class)
            .collect();
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name, "Animal");
    }

    #[test]
    fn test_regex_parse_python_import() {
        let parser = RegexAstParser::new();
        let code = b"import os\nimport sys";
        let result = parser.parse(code, Language::Python).unwrap();
        let imports: Vec<&AstNode> = result
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Import)
            .collect();
        assert_eq!(imports.len(), 2);
    }

    #[test]
    fn test_regex_parse_go_func() {
        let parser = RegexAstParser::new();
        let code = b"func Add(a int, b int) int {\n    return a + b\n}";
        let result = parser.parse(code, Language::Go).unwrap();
        let funcs: Vec<&AstNode> = result
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Function)
            .collect();
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].name, "Add");
    }

    #[test]
    fn test_regex_parse_go_package() {
        let parser = RegexAstParser::new();
        let code = b"package main\n\nfunc main() {}";
        let result = parser.parse(code, Language::Go).unwrap();
        let modules: Vec<&AstNode> = result
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Module)
            .collect();
        assert_eq!(modules.len(), 1);
    }

    #[test]
    fn test_regex_parse_empty() {
        let parser = RegexAstParser::new();
        let result = parser.parse(b"", Language::Rust).unwrap();
        assert!(result.nodes.is_empty());
        assert_eq!(result.line_count, 0);
    }

    #[test]
    fn test_regex_parse_line_count() {
        let parser = RegexAstParser::new();
        let code = b"fn a() {}\nfn b() {}\nfn c() {}\nfn d() {}";
        let result = parser.parse(code, Language::Rust).unwrap();
        assert_eq!(result.line_count, 4);
    }

    #[test]
    fn test_regex_parser_supported_languages() {
        let parser = RegexAstParser::new();
        let langs = parser.supported_languages();
        assert_eq!(langs.len(), 10);
        assert!(langs.contains(&Language::Rust));
        assert!(langs.contains(&Language::Python));
        assert!(langs.contains(&Language::Go));
        assert!(langs.contains(&Language::TypeScript));
        assert!(langs.contains(&Language::JavaScript));
    }

    #[test]
    fn test_language_serialization_roundtrip() {
        let lang = Language::Rust;
        let json = serde_json::to_string(&lang).unwrap();
        let de: Language = serde_json::from_str(&json).unwrap();
        assert_eq!(lang, de);
    }

    #[test]
    fn test_ast_node_serialization() {
        let node = AstNode {
            id: 42,
            kind: NodeKind::Struct,
            name: "Config".into(),
            start_line: 5,
            end_line: 15,
            children: Vec::new(),
            complexity: Some(3.0),
        };
        let json = serde_json::to_string(&node).unwrap();
        let de: AstNode = serde_json::from_str(&json).unwrap();
        assert_eq!(de.id, 42);
        assert_eq!(de.name, "Config");
        assert_eq!(de.kind, NodeKind::Struct);
    }

    #[test]
    fn test_parse_result_serialization() {
        let result = ParseResult {
            language: Language::Rust,
            file_path: "main.rs".into(),
            nodes: vec![],
            line_count: 10,
            parse_time_ms: 5,
            error: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        let de: ParseResult = serde_json::from_str(&json).unwrap();
        assert_eq!(de.file_path, "main.rs");
        assert_eq!(de.line_count, 10);
    }
}
