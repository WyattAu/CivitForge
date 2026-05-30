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
        Self { language_map }
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

        let mut nodes = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        let mut id_counter = 0usize;

        for (line_idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("fn ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("async fn ")
            {
                let name =
                    extract_identifier(trimmed, &["fn ", "pub fn ", "async fn ", "pub async fn "]);
                let node = AstNode {
                    id: format!("node-{id_counter}"),
                    node_type: AstNodeType::Function,
                    name,
                    line_range: (line_idx + 1, line_idx + 1),
                    children: Vec::new(),
                    metadata: HashMap::new(),
                };
                id_counter += 1;
                nodes.push(node);
            } else if trimmed.starts_with("struct ") || trimmed.starts_with("pub struct ") {
                let name = extract_identifier(trimmed, &["struct ", "pub struct "]);
                let node = AstNode {
                    id: format!("node-{id_counter}"),
                    node_type: AstNodeType::Struct,
                    name,
                    line_range: (line_idx + 1, line_idx + 1),
                    children: Vec::new(),
                    metadata: HashMap::new(),
                };
                id_counter += 1;
                nodes.push(node);
            } else if trimmed.starts_with("enum ") || trimmed.starts_with("pub enum ") {
                let name = extract_identifier(trimmed, &["enum ", "pub enum "]);
                let node = AstNode {
                    id: format!("node-{id_counter}"),
                    node_type: AstNodeType::Enum,
                    name,
                    line_range: (line_idx + 1, line_idx + 1),
                    children: Vec::new(),
                    metadata: HashMap::new(),
                };
                id_counter += 1;
                nodes.push(node);
            } else if trimmed.starts_with("trait ") || trimmed.starts_with("pub trait ") {
                let name = extract_identifier(trimmed, &["trait ", "pub trait "]);
                let node = AstNode {
                    id: format!("node-{id_counter}"),
                    node_type: AstNodeType::Trait,
                    name,
                    line_range: (line_idx + 1, line_idx + 1),
                    children: Vec::new(),
                    metadata: HashMap::new(),
                };
                id_counter += 1;
                nodes.push(node);
            } else if trimmed.starts_with("impl ") {
                let node = AstNode {
                    id: format!("node-{id_counter}"),
                    node_type: AstNodeType::ImplBlock,
                    name: extract_impl_name(trimmed),
                    line_range: (line_idx + 1, line_idx + 1),
                    children: Vec::new(),
                    metadata: HashMap::new(),
                };
                id_counter += 1;
                nodes.push(node);
            } else if trimmed.starts_with("mod ") || trimmed.starts_with("pub mod ") {
                let name = extract_identifier(trimmed, &["mod ", "pub mod "]);
                let node = AstNode {
                    id: format!("node-{id_counter}"),
                    node_type: AstNodeType::Module,
                    name,
                    line_range: (line_idx + 1, line_idx + 1),
                    children: Vec::new(),
                    metadata: HashMap::new(),
                };
                id_counter += 1;
                nodes.push(node);
            } else if trimmed.starts_with("use ") {
                let node = AstNode {
                    id: format!("node-{id_counter}"),
                    node_type: AstNodeType::UseStatement,
                    name: trimmed
                        .trim_start_matches("use ")
                        .trim_end_matches(';')
                        .trim()
                        .into(),
                    line_range: (line_idx + 1, line_idx + 1),
                    children: Vec::new(),
                    metadata: HashMap::new(),
                };
                id_counter += 1;
                nodes.push(node);
            } else if trimmed.starts_with("let ") {
                let name = extract_identifier(trimmed, &["let ", "let mut "]);
                let node = AstNode {
                    id: format!("node-{id_counter}"),
                    node_type: AstNodeType::Variable,
                    name,
                    line_range: (line_idx + 1, line_idx + 1),
                    children: Vec::new(),
                    metadata: HashMap::new(),
                };
                id_counter += 1;
                nodes.push(node);
            } else if trimmed.starts_with("if ") || trimmed.starts_with("if let") {
                let node = AstNode {
                    id: format!("node-{id_counter}"),
                    node_type: AstNodeType::IfExpression,
                    name: format!("if_expr_{line_idx}"),
                    line_range: (line_idx + 1, line_idx + 1),
                    children: Vec::new(),
                    metadata: HashMap::new(),
                };
                id_counter += 1;
                nodes.push(node);
            } else if trimmed.starts_with("for ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("loop ")
            {
                let node = AstNode {
                    id: format!("node-{id_counter}"),
                    node_type: AstNodeType::LoopExpression,
                    name: format!("loop_{line_idx}"),
                    line_range: (line_idx + 1, line_idx + 1),
                    children: Vec::new(),
                    metadata: HashMap::new(),
                };
                id_counter += 1;
                nodes.push(node);
            } else if trimmed.starts_with("//")
                || trimmed.starts_with("/*")
                || trimmed.starts_with("*")
            {
                let node = AstNode {
                    id: format!("node-{id_counter}"),
                    node_type: AstNodeType::Comment,
                    name: format!("comment_{line_idx}"),
                    line_range: (line_idx + 1, line_idx + 1),
                    children: Vec::new(),
                    metadata: HashMap::new(),
                };
                id_counter += 1;
                nodes.push(node);
            }
        }

        debug!(language = %language, nodes = nodes.len(), "parsed source");
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
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].name, "hello");
        assert_eq!(nodes[0].node_type, AstNodeType::Function);
        assert_eq!(nodes[1].name, "world");
    }

    #[test]
    fn test_parse_struct_and_enum() {
        let engine = ParseEngine::new();
        let code = "struct Foo {}\npub enum Bar { A, B }";
        let nodes = engine.parse(code, "rust").unwrap();
        let types: Vec<&AstNodeType> = nodes.iter().map(|n| &n.node_type).collect();
        assert!(types.contains(&&AstNodeType::Struct));
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
        assert!(types.contains(&AstNodeType::Variable));
        assert!(types.contains(&AstNodeType::LoopExpression));
    }

    #[test]
    fn test_extract_identifier() {
        let name = extract_identifier("fn main() {", &["fn "]);
        assert_eq!(name, "main");
        let name = extract_identifier("pub async fn handle_request(", &["pub async fn "]);
        assert_eq!(name, "handle_request");
    }
}
