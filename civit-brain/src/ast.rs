#![forbid(unsafe_code)]

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

pub struct ParseEngine {
    language_map: HashMap<String, Vec<AstNodeType>>,
    ts_parser: crate::treesitter::parser::TreeSitterParser,
}

impl ParseEngine {
    pub fn new() -> Self {
        let mut language_map = HashMap::new();
        language_map.insert(
            "rust".into(),
            vec![
                AstNodeType::Function,
                AstNodeType::Struct,
                AstNodeType::Enum,
                AstNodeType::Trait,
                AstNodeType::ImplBlock,
                AstNodeType::Module,
                AstNodeType::UseStatement,
                AstNodeType::Variable,
                AstNodeType::IfExpression,
                AstNodeType::LoopExpression,
                AstNodeType::MatchExpression,
                AstNodeType::CallExpression,
                AstNodeType::Attribute,
                AstNodeType::Comment,
            ],
        );
        language_map.insert(
            "python".into(),
            vec![
                AstNodeType::Function,
                AstNodeType::Variable,
                AstNodeType::IfExpression,
                AstNodeType::LoopExpression,
                AstNodeType::Comment,
            ],
        );
        language_map.insert(
            "go".into(),
            vec![
                AstNodeType::Function,
                AstNodeType::Struct,
                AstNodeType::Variable,
                AstNodeType::IfExpression,
                AstNodeType::LoopExpression,
                AstNodeType::Comment,
            ],
        );
        Self {
            language_map,
            ts_parser: crate::treesitter::parser::TreeSitterParser::new(),
        }
    }
}

impl Default for ParseEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ParseEngine {
    pub fn parse(&self, source: &str, language: &str) -> anyhow::Result<Vec<AstNode>> {
        let supported = self.language_map.contains_key(language);
        if !supported {
            anyhow::bail!("unsupported language: {language}");
        }

        let ts_result = self.ts_parser.parse(source, language);
        let mut nodes = Vec::new();
        let mut id_counter = 0usize;

        for ts_node in &ts_result.root {
            let node_type = map_ts_kind_to_ast(&ts_node.kind);
            let node = AstNode {
                id: format!("node-{id_counter}"),
                node_type,
                name: ts_node.name.clone(),
                line_range: (ts_node.start_line, ts_node.end_line.max(ts_node.start_line)),
                children: convert_ts_children(&ts_node.children, &mut id_counter),
                metadata: ts_node.metadata.clone(),
            };
            id_counter += 1;
            nodes.push(node);
        }

        debug!(language = %language, nodes = nodes.len(), errors = ts_result.error_count, "parsed source via tree-sitter");
        Ok(nodes)
    }

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

fn map_ts_kind_to_ast(kind: &crate::treesitter::parser::TsNodeKind) -> AstNodeType {
    match kind {
        crate::treesitter::parser::TsNodeKind::Function => AstNodeType::Function,
        crate::treesitter::parser::TsNodeKind::Method => AstNodeType::Method,
        crate::treesitter::parser::TsNodeKind::Struct => AstNodeType::Struct,
        crate::treesitter::parser::TsNodeKind::Enum => AstNodeType::Enum,
        crate::treesitter::parser::TsNodeKind::Trait
        | crate::treesitter::parser::TsNodeKind::Interface => AstNodeType::Trait,
        crate::treesitter::parser::TsNodeKind::Class => AstNodeType::Struct,
        crate::treesitter::parser::TsNodeKind::Module => AstNodeType::Module,
        crate::treesitter::parser::TsNodeKind::Import => AstNodeType::UseStatement,
        crate::treesitter::parser::TsNodeKind::Variable
        | crate::treesitter::parser::TsNodeKind::Constant => AstNodeType::Variable,
        crate::treesitter::parser::TsNodeKind::IfStatement => AstNodeType::IfExpression,
        crate::treesitter::parser::TsNodeKind::LoopStatement => AstNodeType::LoopExpression,
        crate::treesitter::parser::TsNodeKind::MatchStatement => AstNodeType::MatchExpression,
        crate::treesitter::parser::TsNodeKind::CallExpression => AstNodeType::CallExpression,
        crate::treesitter::parser::TsNodeKind::Comment => AstNodeType::Comment,
        crate::treesitter::parser::TsNodeKind::Attribute
        | crate::treesitter::parser::TsNodeKind::Annotation => AstNodeType::Attribute,
        crate::treesitter::parser::TsNodeKind::Macro => AstNodeType::Attribute,
        _ => AstNodeType::Unknown,
    }
}

fn convert_ts_children(
    children: &[crate::treesitter::parser::TsNode],
    id_counter: &mut usize,
) -> Vec<AstNode> {
    children
        .iter()
        .map(|c| {
            let node = AstNode {
                id: format!("node-{id_counter}"),
                node_type: map_ts_kind_to_ast(&c.kind),
                name: c.name.clone(),
                line_range: (c.start_line, c.end_line.max(c.start_line)),
                children: convert_ts_children(&c.children, id_counter),
                metadata: c.metadata.clone(),
            };
            *id_counter += 1;
            node
        })
        .collect()
}

#[allow(dead_code)]
fn extract_identifier(line: &str, prefixes: &[&str]) -> String {
    for prefix in prefixes {
        if let Some(rest) = line.strip_prefix(prefix) {
            let rest = rest.trim();
            let end = rest.find(['(', '<', ':', ' ', '{']).unwrap_or(rest.len());
            return rest[..end].to_string();
        }
    }
    "unknown".into()
}

#[allow(dead_code)]
fn extract_impl_name(line: &str) -> String {
    let rest = line.strip_prefix("impl").unwrap_or(line).trim();
    let end = rest.find([' ', '{', '<']).unwrap_or(rest.len());
    let name = rest[..end].trim();
    if name.is_empty() || name == "for" {
        "impl_block".into()
    } else {
        name.to_string()
    }
}

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
        // Tree-sitter may not detect simple let bindings as Variable nodes,
        // but should detect loop structures
        assert!(types.iter().any(|t| matches!(
            t,
            AstNodeType::LoopExpression | AstNodeType::Variable
        )));
    }

    #[test]
    fn test_extract_identifier() {
        let name = extract_identifier("fn main() {", &["fn "]);
        assert_eq!(name, "main");
        let name = extract_identifier("pub async fn handle_request(", &["pub async fn "]);
        assert_eq!(name, "handle_request");
    }
}
