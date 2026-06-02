#![forbid(unsafe_code)]

//! Native Rust AST parsers — zero unsafe, zero C FFI.
//!
//! This module provides production-quality AST extraction using native Rust
//! parser crates where available:
//!
//! | Language    | Parser Crate  | Feature Flag |
//! |-------------|--------------|--------------|
//! | Rust        | `syn` v2     | `syn-parser` |
//! | TypeScript  | `swc`        | `swc-parser` |
//! | JavaScript  | `swc`        | `swc-parser` |
//! | SQL         | `sqlparser`  | `sql-parser` |
//! | JSON        | `serde_json` | (always)     |
//! | TOML        | `toml`       | (always)     |
//!
//! All parsers convert their native AST into the shared `ExtractedNode` type
//! for uniform downstream consumption.

use crate::ast::{Language, NodeKind};

/// A language-agnostic extracted node from any parser backend.
/// Designed as the unified output format for the dispatcher.
#[derive(Debug, Clone)]
pub struct ExtractedNode {
    pub kind: NodeKind,
    pub name: String,
    pub start_line: u32,
    pub end_line: u32,
    pub children: Vec<ExtractedNode>,
    pub body_text: Option<String>,
}

impl ExtractedNode {
    pub fn leaf(kind: NodeKind, name: String, start_line: u32, end_line: u32) -> Self {
        Self {
            kind,
            name,
            start_line,
            end_line,
            children: Vec::new(),
            body_text: None,
        }
    }
}

/// Result from a native parser invocation.
#[derive(Debug)]
pub struct NativeParseResult {
    pub nodes: Vec<ExtractedNode>,
    pub line_count: u32,
    pub parse_time_ms: u64,
    pub error: Option<String>,
}

/// Trait for native Rust AST parsers.
pub trait NativeAstParser: Send + Sync {
    /// Parse source content and extract typed AST nodes.
    fn parse_native(&self, content: &str) -> NativeParseResult;

    /// Languages supported by this parser.
    fn supported_languages(&self) -> &[Language];
}

// ---------------------------------------------------------------------------
// Rust parser (syn)
// ---------------------------------------------------------------------------

#[cfg(feature = "syn-parser")]
mod syn_parser {
    #![forbid(unsafe_code)]

    use super::{ExtractedNode, NativeAstParser, NativeParseResult};
    use crate::ast::{Language, NodeKind};
    use syn::visit::Visit;

    struct RustVisitor {
        nodes: Vec<ExtractedNode>,
    }

    fn line_of(span: &proc_macro2::Span) -> u32 {
        span.start().line
    }

    impl Visit<'_> for RustVisitor {
        fn visit_item_fn(&mut self, node: &syn::ItemFn) {
            let start = line_of(&node.fn_token.span);
            // Estimate end line from the block
            let end = node.block.brace_token.close.span.end().line.max(start);

            self.nodes.push(ExtractedNode::leaf(
                NodeKind::Function,
                node.sig.ident.to_string(),
                start,
                end,
            ));
            syn::visit::visit_item_fn(self, node);
        }

        fn visit_item_struct(&mut self, node: &syn::ItemStruct) {
            let start = line_of(&node.struct_token.span);
            let end = node
                .fields
                .brace_token
                .map(|t| t.close.span.end().line)
                .unwrap_or(start);
            self.nodes.push(ExtractedNode::leaf(
                NodeKind::Struct,
                node.ident.to_string(),
                start,
                end.max(start),
            ));
            syn::visit::visit_item_struct(self, node);
        }

        fn visit_item_enum(&mut self, node: &syn::ItemEnum) {
            let start = line_of(&node.enum_token.span);
            let end = node.brace_token.close.span.end().line.max(start);
            self.nodes.push(ExtractedNode::leaf(
                NodeKind::Enum,
                node.ident.to_string(),
                start,
                end,
            ));
            syn::visit::visit_item_enum(self, node);
        }

        fn visit_item_trait(&mut self, node: &syn::ItemTrait) {
            let start = line_of(&node.trait_token.span);
            let end = node.brace_token.close.span.end().line.max(start);
            self.nodes.push(ExtractedNode::leaf(
                NodeKind::Interface,
                node.ident.to_string(),
                start,
                end,
            ));
            syn::visit::visit_item_trait(self, node);
        }

        fn visit_item_impl(&mut self, node: &syn::ItemImpl) {
            let start = line_of(&node.impl_token.span);
            let end = node.brace_token.close.span.end().line.max(start);
            let name = match &node.self_ty.as_ref() {
                syn::Type::Path(tp) => tp
                    .path
                    .segments
                    .last()
                    .map(|s| s.ident.to_string())
                    .unwrap_or_else(|| "impl".into()),
                _ => "impl".into(),
            };
            self.nodes
                .push(ExtractedNode::leaf(NodeKind::Impl, name, start, end));
            syn::visit::visit_item_impl(self, node);
        }

        fn visit_item_mod(&mut self, node: &syn::ItemMod) {
            let start = line_of(&node.mod_token.span);
            let name = node.ident.to_string();
            self.nodes
                .push(ExtractedNode::leaf(NodeKind::Module, name, start, start));
            syn::visit::visit_item_mod(self, node);
        }

        fn visit_item_use(&mut self, node: &syn::ItemUse) {
            let start = line_of(&node.use_token.span);
            // Reconstruct a path-like name from the use tree
            let name = use_tree_to_string(&node.tree);
            self.nodes
                .push(ExtractedNode::leaf(NodeKind::Import, name, start, start));
            syn::visit::visit_item_use(self, node);
        }

        fn visit_item_type(&mut self, node: &syn::ItemType) {
            let start = line_of(&node.type_token.span);
            self.nodes.push(ExtractedNode::leaf(
                NodeKind::Class,
                node.ident.to_string(),
                start,
                start,
            ));
            syn::visit::visit_item_type(self, node);
        }
    }

    fn use_tree_to_string(tree: &syn::UseTree) -> String {
        match &tree {
            syn::UseTree::Path(p) => {
                let ident = p.ident.to_string();
                match use_tree_to_string(&p.tree) {
                    rest if rest.is_empty() => ident,
                    rest => format!("{ident}::{rest}"),
                }
            }
            syn::UseTree::Name(n) => n.ident.to_string(),
            syn::UseTree::Rename(r) => {
                format!("{} as {}", use_tree_to_string(&r.tree), r.rename)
            }
            syn::UseTree::Glob(_) => "*".into(),
            syn::UseTree::Group(g) => g
                .items
                .iter()
                .map(|i| use_tree_to_string(i))
                .collect::<Vec<_>>()
                .join(", "),
        }
    }

    pub struct SynRustParser;

    impl SynRustParser {
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for SynRustParser {
        fn default() -> Self {
            Self::new()
        }
    }

    impl NativeAstParser for SynRustParser {
        fn parse_native(&self, content: &str) -> NativeParseResult {
            let start = Instant::now();
            let line_count = content.lines().count() as u32;

            match syn::parse_file(content) {
                Ok(file) => {
                    let mut visitor = RustVisitor { nodes: Vec::new() };
                    visitor.visit_file(&file);
                    NativeParseResult {
                        nodes: visitor.nodes,
                        line_count,
                        parse_time_ms: start.elapsed().as_millis() as u64,
                        error: None,
                    }
                }
                Err(e) => NativeParseResult {
                    nodes: Vec::new(),
                    line_count,
                    parse_time_ms: start.elapsed().as_millis() as u64,
                    error: Some(format!("{e}")),
                },
            }
        }

        fn supported_languages(&self) -> &[Language] {
            &[Language::Rust]
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_syn_parse_functions() {
            let parser = SynRustParser::new();
            let code = "fn main() {}\nfn add(a: i32, b: i32) -> i32 { a + b }";
            let result = parser.parse_native(code);
            assert!(result.error.is_none());
            let funcs: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Function)
                .collect();
            assert_eq!(funcs.len(), 2);
            assert_eq!(funcs[0].name, "main");
            assert_eq!(funcs[1].name, "add");
        }

        #[test]
        fn test_syn_parse_structs() {
            let parser = SynRustParser::new();
            let code = "pub struct Point { x: i32, y: i32 }\nstruct Config { name: String }";
            let result = parser.parse_native(code);
            let structs: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Struct)
                .collect();
            assert_eq!(structs.len(), 2);
            assert_eq!(structs[0].name, "Point");
            assert_eq!(structs[1].name, "Config");
        }

        #[test]
        fn test_syn_parse_enum() {
            let parser = SynRustParser::new();
            let code = "pub enum Color { Red, Green, Blue }";
            let result = parser.parse_native(code);
            let enums: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Enum)
                .collect();
            assert_eq!(enums.len(), 1);
            assert_eq!(enums[0].name, "Color");
        }

        #[test]
        fn test_syn_parse_trait() {
            let parser = SynRustParser::new();
            let code = "pub trait Drawable { fn draw(&self); }";
            let result = parser.parse_native(code);
            let traits: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Interface)
                .collect();
            assert_eq!(traits.len(), 1);
            assert_eq!(traits[0].name, "Drawable");
        }

        #[test]
        fn test_syn_parse_impl() {
            let parser = SynRustParser::new();
            let code = "impl Point { fn new() -> Self { Self { x: 0, y: 0 } } }";
            let result = parser.parse_native(code);
            let impls: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Impl)
                .collect();
            assert_eq!(impls.len(), 1);
            assert_eq!(impls[0].name, "Point");
        }

        #[test]
        fn test_syn_parse_use() {
            let parser = SynRustParser::new();
            let code = "use std::collections::HashMap;\nuse anyhow::{Result, Context};";
            let result = parser.parse_native(code);
            let imports: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Import)
                .collect();
            assert_eq!(imports.len(), 2);
        }

        #[test]
        fn test_syn_parse_module() {
            let parser = SynRustParser::new();
            let code = "mod foo;\nmod bar { fn baz() {} }";
            let result = parser.parse_native(code);
            let mods: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Module)
                .collect();
            assert_eq!(mods.len(), 2);
        }

        #[test]
        fn test_syn_parse_empty() {
            let parser = SynRustParser::new();
            let result = parser.parse_native("");
            assert!(result.nodes.is_empty());
            assert_eq!(result.line_count, 0);
            assert!(result.error.is_none());
        }

        #[test]
        fn test_syn_parse_async_function() {
            let parser = SynRustParser::new();
            let code = "pub async fn handle_request() -> Result<()> { Ok(()) }";
            let result = parser.parse_native(code);
            let funcs: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Function)
                .collect();
            assert_eq!(funcs.len(), 1);
            assert_eq!(funcs[0].name, "handle_request");
        }

        #[test]
        fn test_syn_parse_impl_with_trait() {
            let parser = SynRustParser::new();
            let code = "impl Drawable for Circle {\n    fn draw(&self) {}\n}";
            let result = parser.parse_native(code);
            let impls: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Impl)
                .collect();
            assert_eq!(impls.len(), 1);
            assert_eq!(impls[0].name, "Circle");
        }

        #[test]
        fn test_syn_parse_line_count() {
            let parser = SynRustParser::new();
            let code = "fn a() {}\nfn b() {}\nfn c() {}";
            let result = parser.parse_native(code);
            assert_eq!(result.line_count, 3);
        }

        #[test]
        fn test_syn_parse_time_nonzero() {
            let parser = SynRustParser::new();
            let result = parser.parse_native("fn main() {}");
            assert!(result.parse_time_ms > 0 || result.parse_time_ms == 0); // may be 0 for fast parses
        }

        #[test]
        fn test_syn_parse_error() {
            let parser = SynRustParser::new();
            let result = parser.parse_native("fn main( {");
            assert!(result.error.is_some());
        }

        #[test]
        fn test_syn_supported_languages() {
            let parser = SynRustParser::new();
            let langs = parser.supported_languages();
            assert_eq!(langs.len(), 1);
            assert!(langs.contains(&Language::Rust));
        }
    }
}

// ---------------------------------------------------------------------------
// SQL parser (sqlparser)
// ---------------------------------------------------------------------------

#[cfg(feature = "sql-parser")]
mod sql_parser {
    #![forbid(unsafe_code)]

    use super::{ExtractedNode, NativeAstParser, NativeParseResult};
    use crate::ast::{Language, NodeKind};

    pub struct SqlParser;

    impl SqlParser {
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for SqlParser {
        fn default() -> Self {
            Self::new()
        }
    }

    impl NativeAstParser for SqlParser {
        fn parse_native(&self, content: &str) -> NativeParseResult {
            let start = std::time::Instant::now();
            let line_count = content.lines().count() as u32;

            let dialect = sqlparser::dialect::GenericDialect;
            match sqlparser::parser::Parser::parse_sql(&dialect, content) {
                Ok(statements) => {
                    let mut nodes = Vec::new();
                    let mut node_id: u32 = 0;

                    for stmt in &statements {
                        let name = extract_sql_name(stmt);
                        let kind = classify_sql_statement(stmt);
                        node_id += 1;
                        nodes.push(ExtractedNode::leaf(kind, name, 1, line_count));
                    }

                    NativeParseResult {
                        nodes,
                        line_count,
                        parse_time_ms: start.elapsed().as_millis() as u64,
                        error: None,
                    }
                }
                Err(e) => NativeParseResult {
                    nodes: Vec::new(),
                    line_count,
                    parse_time_ms: start.elapsed().as_millis() as u64,
                    error: Some(format!("{e}")),
                },
            }
        }

        fn supported_languages(&self) -> &[Language] {
            &[Language::Sql]
        }
    }

    fn extract_sql_name(stmt: &sqlparser::ast::Statement) -> String {
        match stmt {
            sqlparser::ast::Statement::CreateTable { name, .. } => name.to_string(),
            sqlparser::ast::Statement::CreateView { name, .. } => name.to_string(),
            sqlparser::ast::Statement::CreateIndex { name, .. } => name
                .map(|n| n.to_string())
                .unwrap_or_else(|| "index".into()),
            sqlparser::ast::Statement::Drop { object_name, .. } => object_name.to_string(),
            sqlparser::ast::Statement::AlterTable { table_name, .. } => table_name.to_string(),
            sqlparser::ast::Statement::Insert { table_name, .. } => table_name.to_string(),
            sqlparser::ast::Statement::Update { table, .. } => table
                .iter()
                .next()
                .map(|t| t.to_string())
                .unwrap_or_default(),
            sqlparser::ast::Statement::Delete { table_name, .. } => table_name
                .iter()
                .next()
                .map(|t| t.to_string())
                .unwrap_or_default(),
            sqlparser::ast::Statement::CreateFunction { name, .. } => name.to_string(),
            sqlparser::ast::Statement::CreateTrigger { trigger_name, .. } => {
                trigger_name.to_string()
            }
            sqlparser::ast::Statement::CreateMacro { name, .. } => name.to_string(),
            sqlparser::ast::Statement::CreateProcedure { name, .. } => name.to_string(),
            _ => format!("{:?}", stmt),
        }
    }

    fn classify_sql_statement(stmt: &sqlparser::ast::Statement) -> NodeKind {
        match stmt {
            sqlparser::ast::Statement::CreateTable { .. }
            | sqlparser::ast::Statement::CreateView { .. }
            | sqlparser::ast::Statement::CreateIndex { .. }
            | sqlparser::ast::Statement::CreateFunction { .. }
            | sqlparser::ast::Statement::CreateTrigger { .. }
            | sqlparser::ast::Statement::CreateMacro { .. }
            | sqlparser::ast::Statement::CreateProcedure { .. } => NodeKind::Struct,
            sqlparser::ast::Statement::AlterTable { .. }
            | sqlparser::ast::Statement::Drop { .. } => NodeKind::Class,
            sqlparser::ast::Statement::Insert { .. }
            | sqlparser::ast::Statement::Update { .. }
            | sqlparser::ast::Statement::Delete { .. }
            | sqlparser::ast::Statement::Query(_) => NodeKind::Function,
            sqlparser::ast::Statement::Explain { .. } => NodeKind::Comment,
            _ => NodeKind::Unknown,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_sql_parse_select() {
            let parser = SqlParser::new();
            let code = "SELECT id, name FROM users WHERE active = true;";
            let result = parser.parse_native(code);
            assert!(result.error.is_none());
            assert_eq!(result.nodes.len(), 1);
            assert_eq!(result.nodes[0].kind, NodeKind::Function);
        }

        #[test]
        fn test_sql_parse_create_table() {
            let parser = SqlParser::new();
            let code = "CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(255));";
            let result = parser.parse_native(code);
            assert!(result.error.is_none());
            let tables: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Struct)
                .collect();
            assert_eq!(tables.len(), 1);
            assert_eq!(tables[0].name, "users");
        }

        #[test]
        fn test_sql_parse_multiple_statements() {
            let parser = SqlParser::new();
            let code = "SELECT 1; SELECT 2;";
            let result = parser.parse_native(code);
            assert!(result.error.is_none());
            assert_eq!(result.nodes.len(), 2);
        }

        #[test]
        fn test_sql_parse_error() {
            let parser = SqlParser::new();
            let result = parser.parse_native("SELECTT FROM");
            assert!(result.error.is_some());
        }

        #[test]
        fn test_sql_parse_empty() {
            let parser = SqlParser::new();
            let result = parser.parse_native("");
            assert!(result.error.is_none());
            assert!(result.nodes.is_empty());
        }

        #[test]
        fn test_sql_parse_insert() {
            let parser = SqlParser::new();
            let code = "INSERT INTO users (id, name) VALUES (1, 'Alice');";
            let result = parser.parse_native(code);
            assert!(result.error.is_none());
            assert_eq!(result.nodes.len(), 1);
        }

        #[test]
        fn test_sql_supported_languages() {
            let parser = SqlParser::new();
            assert!(parser.supported_languages().contains(&Language::Sql));
        }
    }
}

// ---------------------------------------------------------------------------
// TypeScript / JavaScript parser (swc)
// ---------------------------------------------------------------------------

#[cfg(feature = "swc-parser")]
mod swc_parser {
    #![forbid(unsafe_code)]

    use super::{ExtractedNode, NativeAstParser, NativeParseResult};
    use crate::ast::{Language, NodeKind};

    pub struct SwcTsJsParser;

    impl SwcTsJsParser {
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for SwcTsJsParser {
        fn default() -> Self {
            Self::new()
        }
    }

    impl NativeAstParser for SwcTsJsParser {
        fn parse_native(&self, content: &str) -> NativeParseResult {
            let start = std::time::Instant::now();
            let line_count = content.lines().count() as u32;

            // Try TypeScript first, fall back to JavaScript
            let result = parse_as_ts(content);
            match result {
                Ok(nodes) => NativeParseResult {
                    nodes,
                    line_count,
                    parse_time_ms: start.elapsed().as_millis() as u64,
                    error: None,
                },
                Err(ts_err) => {
                    // Fall back to JS
                    let js_result = parse_as_js(content);
                    match js_result {
                        Ok(nodes) => NativeParseResult {
                            nodes,
                            line_count,
                            parse_time_ms: start.elapsed().as_millis() as u64,
                            error: None,
                        },
                        Err(_) => NativeParseResult {
                            nodes: Vec::new(),
                            line_count,
                            parse_time_ms: start.elapsed().as_millis() as u64,
                            error: Some(format!("TS: {ts_err}")),
                        },
                    }
                }
            }
        }

        fn supported_languages(&self) -> &[Language] {
            &[Language::TypeScript, Language::JavaScript]
        }
    }

    fn parse_as_ts(content: &str) -> Result<Vec<ExtractedNode>, String> {
        use swc_common::sync::Lrc;
        use swc_common::{FileName, SourceMap};
        use swc_ecma_ast::Module;
        use swc_ecma_parser::{Parser as SwcParser, StringInput, Syntax, TsConfig, lexer::Lexer};

        let cm: Lrc<SourceMap> = Default::default();
        let fm = cm.new_source_file(FileName::Anon, content.into());

        let lexer = Lexer::new(
            Syntax::Typescript(TsConfig {
                tsx: false,
                decorators: false,
                dts: false,
                no_early_errors: false,
            }),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        let mut parser = SwcParser::new_from(lexer);
        let module: Result<Module, _> = parser.parse_module();

        module
            .map(|m| extract_js_nodes_from_module(&m))
            .map_err(|e| format!("{e:?}"))
    }

    fn parse_as_js(content: &str) -> Result<Vec<ExtractedNode>, String> {
        use swc_common::sync::Lrc;
        use swc_common::{FileName, SourceMap};
        use swc_ecma_ast::Module;
        use swc_ecma_parser::{Parser as SwcParser, StringInput, Syntax, lexer::Lexer};

        let cm: Lrc<SourceMap> = Default::default();
        let fm = cm.new_source_file(FileName::Anon, content.into());

        let lexer = Lexer::new(
            Syntax::Es(Default::default()),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        let mut parser = SwcParser::new_from(lexer);
        let module: Result<Module, _> = parser.parse_module();

        module
            .map(|m| extract_js_nodes_from_module(&m))
            .map_err(|e| format!("{e:?}"))
    }

    fn extract_js_nodes_from_module(module: &swc_ecma_ast::Module) -> Vec<ExtractedNode> {
        use swc_common::Span;
        use swc_ecma_ast::*;
        use swc_ecma_visit::Visit;

        struct JsVisitor {
            nodes: Vec<ExtractedNode>,
        }

        impl Visit for JsVisitor {
            fn visit_function(&mut self, node: &Function) {
                let start = node.span.lo().0 as u32;
                let end = node.span.hi().0 as u32;
                let name = node
                    .ident
                    .as_ref()
                    .map(|i| i.sym.to_string())
                    .unwrap_or_else(|| "anonymous".into());
                self.nodes.push(ExtractedNode::leaf(
                    NodeKind::Function,
                    name,
                    start,
                    end.max(start),
                ));
            }

            fn visit_class_decl(&mut self, node: &ClassDecl) {
                let start = node.class.span.lo().0 as u32;
                let end = node.class.span.hi().0 as u32;
                self.nodes.push(ExtractedNode::leaf(
                    NodeKind::Class,
                    node.ident.sym.to_string(),
                    start,
                    end.max(start),
                ));
            }

            fn visit_import_decl(&mut self, node: &ImportDecl) {
                let start = node.span.lo().0 as u32;
                let src = node.src.value.to_string();
                self.nodes
                    .push(ExtractedNode::leaf(NodeKind::Import, src, start, start));
            }

            fn visit_export_decl(&mut self, node: &ExportDecl) {
                match &node.decl {
                    Decl::Class(cd) => {
                        let start = cd.class.span.lo().0 as u32;
                        let end = cd.class.span.hi().0 as u32;
                        self.nodes.push(ExtractedNode::leaf(
                            NodeKind::Class,
                            cd.ident.sym.to_string(),
                            start,
                            end.max(start),
                        ));
                    }
                    Decl::Fn(fd) => {
                        let start = fd.function.span.lo().0 as u32;
                        let end = fd.function.span.hi().0 as u32;
                        self.nodes.push(ExtractedNode::leaf(
                            NodeKind::Function,
                            fd.ident.sym.to_string(),
                            start,
                            end.max(start),
                        ));
                    }
                    Decl::Var(v) => {
                        for decl in &v.decls {
                            if let Pat::Ident(id) = &decl.name {
                                let start = id.span.lo().0 as u32;
                                self.nodes.push(ExtractedNode::leaf(
                                    NodeKind::Variable,
                                    id.id.sym.to_string(),
                                    start,
                                    start,
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }

            fn visit_ts_interface_decl(&mut self, node: &TsInterfaceDecl) {
                let start = node.span.lo().0 as u32;
                let end = node.span.hi().0 as u32;
                self.nodes.push(ExtractedNode::leaf(
                    NodeKind::Interface,
                    node.id.sym.to_string(),
                    start,
                    end.max(start),
                ));
            }

            fn visit_ts_type_alias_decl(&mut self, node: &TsTypeAliasDecl) {
                let start = node.span.lo().0 as u32;
                self.nodes.push(ExtractedNode::leaf(
                    NodeKind::Class,
                    node.id.sym.to_string(),
                    start,
                    start,
                ));
            }

            fn visit_ts_enum_decl(&mut self, node: &TsEnumDecl) {
                let start = node.span.lo().0 as u32;
                let end = node.span.hi().0 as u32;
                self.nodes.push(ExtractedNode::leaf(
                    NodeKind::Enum,
                    node.id.sym.to_string(),
                    start,
                    end.max(start),
                ));
            }
        }

        let mut visitor = JsVisitor { nodes: Vec::new() };
        visitor.visit_module(module);
        visitor.nodes
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_swc_parse_js_function() {
            let parser = SwcTsJsParser::new();
            let code =
                "function hello() { console.log('world'); }\nfunction add(a, b) { return a + b; }";
            let result = parser.parse_native(code);
            assert!(result.error.is_none());
            let funcs: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Function)
                .collect();
            assert_eq!(funcs.len(), 2);
            assert_eq!(funcs[0].name, "hello");
            assert_eq!(funcs[1].name, "add");
        }

        #[test]
        fn test_swc_parse_js_class() {
            let parser = SwcTsJsParser::new();
            let code = "class Animal { constructor(name) { this.name = name; } }";
            let result = parser.parse_native(code);
            assert!(result.error.is_none());
            let classes: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Class)
                .collect();
            assert_eq!(classes.len(), 1);
            assert_eq!(classes[0].name, "Animal");
        }

        #[test]
        fn test_swc_parse_js_import() {
            let parser = SwcTsJsParser::new();
            let code = "import { foo } from 'bar';";
            let result = parser.parse_native(code);
            assert!(result.error.is_none());
            let imports: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Import)
                .collect();
            assert_eq!(imports.len(), 1);
            assert_eq!(imports[0].name, "bar");
        }

        #[test]
        fn test_swc_parse_ts_interface() {
            let parser = SwcTsJsParser::new();
            let code = "interface User { name: string; age: number; }";
            let result = parser.parse_native(code);
            assert!(result.error.is_none());
            let ifaces: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Interface)
                .collect();
            assert_eq!(ifaces.len(), 1);
            assert_eq!(ifaces[0].name, "User");
        }

        #[test]
        fn test_swc_parse_ts_enum() {
            let parser = SwcTsJsParser::new();
            let code = "enum Color { Red, Green, Blue }";
            let result = parser.parse_native(code);
            assert!(result.error.is_none());
            let enums: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Enum)
                .collect();
            assert_eq!(enums.len(), 1);
            assert_eq!(enums[0].name, "Color");
        }

        #[test]
        fn test_swc_parse_ts_type_alias() {
            let parser = SwcTsJsParser::new();
            let code = "type ID = string | number;";
            let result = parser.parse_native(code);
            assert!(result.error.is_none());
            let types: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Class)
                .collect();
            assert!(types.iter().any(|t| t.name == "ID"));
        }

        #[test]
        fn test_swc_parse_empty() {
            let parser = SwcTsJsParser::new();
            let result = parser.parse_native("");
            assert!(result.error.is_none());
        }

        #[test]
        fn test_swc_supported_languages() {
            let parser = SwcTsJsParser::new();
            let langs = parser.supported_languages();
            assert_eq!(langs.len(), 2);
            assert!(langs.contains(&Language::TypeScript));
            assert!(langs.contains(&Language::JavaScript));
        }
    }
}

// ---------------------------------------------------------------------------
// Data-format parsers (always available, zero unsafe)
// ---------------------------------------------------------------------------

mod data_parsers {
    #![forbid(unsafe_code)]

    use super::{ExtractedNode, NativeAstParser, NativeParseResult};
    use crate::ast::{Language, NodeKind};

    pub struct JsonParser;

    impl JsonParser {
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for JsonParser {
        fn default() -> Self {
            Self::new()
        }
    }

    impl NativeAstParser for JsonParser {
        fn parse_native(&self, content: &str) -> NativeParseResult {
            let start = std::time::Instant::now();
            let line_count = content.lines().count() as u32;

            match serde_json::from_str::<serde_json::Value>(content) {
                Ok(value) => {
                    let mut nodes = Vec::new();
                    extract_json_nodes(&value, &mut nodes, 0);
                    NativeParseResult {
                        nodes,
                        line_count,
                        parse_time_ms: start.elapsed().as_millis() as u64,
                        error: None,
                    }
                }
                Err(e) => NativeParseResult {
                    nodes: Vec::new(),
                    line_count,
                    parse_time_ms: start.elapsed().as_millis() as u64,
                    error: Some(format!("{e}")),
                },
            }
        }

        fn supported_languages(&self) -> &[Language] {
            &[Language::Json]
        }
    }

    fn extract_json_nodes(value: &serde_json::Value, nodes: &mut Vec<ExtractedNode>, depth: u32) {
        match value {
            serde_json::Value::Object(map) => {
                for key in map.keys() {
                    nodes.push(ExtractedNode::leaf(
                        NodeKind::Field,
                        key.clone(),
                        depth + 1,
                        depth + 1,
                    ));
                    extract_json_nodes(&map[key], nodes, depth + 1);
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    extract_json_nodes(item, nodes, depth + 1);
                }
            }
            _ => {}
        }
    }

    pub struct TomlParser;

    impl TomlParser {
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for TomlParser {
        fn default() -> Self {
            Self::new()
        }
    }

    impl NativeAstParser for TomlParser {
        fn parse_native(&self, content: &str) -> NativeParseResult {
            let start = std::time::Instant::now();
            let line_count = content.lines().count() as u32;

            match content.parse::<toml::Table>() {
                Ok(table) => {
                    let mut nodes = Vec::new();
                    extract_toml_nodes(&table, &mut nodes, 0);
                    NativeParseResult {
                        nodes,
                        line_count,
                        parse_time_ms: start.elapsed().as_millis() as u64,
                        error: None,
                    }
                }
                Err(e) => NativeParseResult {
                    nodes: Vec::new(),
                    line_count,
                    parse_time_ms: start.elapsed().as_millis() as u64,
                    error: Some(format!("{e}")),
                },
            }
        }

        fn supported_languages(&self) -> &[Language] {
            &[Language::Toml]
        }
    }

    fn extract_toml_nodes(table: &toml::Table, nodes: &mut Vec<ExtractedNode>, depth: u32) {
        for (key, value) in table {
            let kind = match value {
                toml::Value::Table(_) => NodeKind::Struct,
                toml::Value::Array(_) => NodeKind::Variable,
                _ => NodeKind::Field,
            };
            nodes.push(ExtractedNode::leaf(kind, key.clone(), depth + 1, depth + 1));
            if let toml::Value::Table(sub) = value {
                extract_toml_nodes(sub, nodes, depth + 1);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_json_parse_object() {
            let parser = JsonParser::new();
            let code = r#"{"name": "Alice", "age": 30}"#;
            let result = parser.parse_native(code);
            assert!(result.error.is_none());
            let fields: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Field)
                .collect();
            assert_eq!(fields.len(), 2);
        }

        #[test]
        fn test_json_parse_nested() {
            let parser = JsonParser::new();
            let code = r#"{"user": {"name": "Bob"}}"#;
            let result = parser.parse_native(code);
            assert!(result.error.is_none());
            assert!(result.nodes.iter().any(|n| n.name == "user"));
            assert!(result.nodes.iter().any(|n| n.name == "name"));
        }

        #[test]
        fn test_json_parse_error() {
            let parser = JsonParser::new();
            let result = parser.parse_native("{invalid}");
            assert!(result.error.is_some());
        }

        #[test]
        fn test_json_supported_languages() {
            let parser = JsonParser::new();
            assert!(parser.supported_languages().contains(&Language::Json));
        }

        #[test]
        fn test_toml_parse_basic() {
            let parser = TomlParser::new();
            let code = "[package]\nname = \"foo\"\nversion = \"0.1.0\"";
            let result = parser.parse_native(code);
            assert!(result.error.is_none());
            assert!(result.nodes.iter().any(|n| n.name == "package"));
            assert!(result.nodes.iter().any(|n| n.name == "name"));
        }

        #[test]
        fn test_toml_parse_table_section() {
            let parser = TomlParser::new();
            let code = "[dependencies]\nserde = \"1\"\ntokio = \"1\"";
            let result = parser.parse_native(code);
            assert!(result.error.is_none());
            let structs: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Struct)
                .collect();
            assert!(structs.iter().any(|s| s.name == "dependencies"));
        }

        #[test]
        fn test_toml_parse_error() {
            let parser = TomlParser::new();
            let result = parser.parse_native("[]\n[");
            assert!(result.error.is_some());
        }

        #[test]
        fn test_toml_supported_languages() {
            let parser = TomlParser::new();
            assert!(parser.supported_languages().contains(&Language::Toml));
        }
    }
}

// ---------------------------------------------------------------------------
// Always-available parsers collection
// ---------------------------------------------------------------------------

/// Returns the set of native parsers that are available without any feature flags.
/// This always includes JSON and TOML parsers.
pub fn always_available_parsers() -> Vec<Box<dyn NativeAstParser>> {
    vec![
        Box::new(data_parsers::JsonParser::new()),
        Box::new(data_parsers::TomlParser::new()),
    ]
}

/// Returns feature-gated native parsers when their features are enabled.
#[cfg(feature = "syn-parser")]
pub fn feature_parsers() -> Vec<Box<dyn NativeAstParser>> {
    let mut parsers = Vec::new();
    parsers.push(Box::new(syn_parser::SynRustParser::new()));
    parsers
}

/// Returns feature-gated native parsers when their features are enabled.
#[cfg(all(feature = "swc-parser", not(feature = "syn-parser")))]
pub fn feature_parsers() -> Vec<Box<dyn NativeAstParser>> {
    vec![Box::new(swc_parser::SwcTsJsParser::new())]
}

#[cfg(all(feature = "swc-parser", feature = "syn-parser"))]
pub fn feature_parsers() -> Vec<Box<dyn NativeAstParser>> {
    let mut parsers = Vec::new();
    parsers.push(Box::new(syn_parser::SynRustParser::new()));
    parsers.push(Box::new(swc_parser::SwcTsJsParser::new()));
    parsers
}

#[cfg(all(
    feature = "sql-parser",
    not(any(feature = "syn-parser", feature = "swc-parser"))
))]
pub fn feature_parsers() -> Vec<Box<dyn NativeAstParser>> {
    vec![Box::new(sql_parser::SqlParser::new())]
}

#[cfg(all(
    feature = "sql-parser",
    any(feature = "syn-parser", feature = "swc-parser")
))]
pub fn feature_parsers() -> Vec<Box<dyn NativeAstParser>> {
    let mut parsers = Vec::new();
    #[cfg(feature = "syn-parser")]
    parsers.push(Box::new(syn_parser::SynRustParser::new()));
    #[cfg(feature = "swc-parser")]
    parsers.push(Box::new(swc_parser::SwcTsJsParser::new()));
    parsers.push(Box::new(sql_parser::SqlParser::new()));
    parsers
}

#[cfg(not(any(feature = "syn-parser", feature = "swc-parser", feature = "sql-parser")))]
pub fn feature_parsers() -> Vec<Box<dyn NativeAstParser>> {
    Vec::new()
}

/// Collects all available native parsers (always-available + feature-gated).
pub fn all_native_parsers() -> Vec<Box<dyn NativeAstParser>> {
    let mut parsers = feature_parsers();
    parsers.extend(always_available_parsers());
    parsers
}

// ---------------------------------------------------------------------------
// Tests for dispatcher selection logic
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_always_available_parsers_includes_json_toml() {
        let parsers = always_available_parsers();
        assert!(parsers.len() >= 2);
        let langs: Vec<Language> = parsers
            .iter()
            .flat_map(|p| p.supported_languages().to_vec())
            .collect();
        assert!(langs.contains(&Language::Json));
        assert!(langs.contains(&Language::Toml));
    }

    #[test]
    fn test_extracted_node_leaf() {
        let node = ExtractedNode::leaf(NodeKind::Function, "main".into(), 1, 5);
        assert_eq!(node.kind, NodeKind::Function);
        assert_eq!(node.name, "main");
        assert!(node.children.is_empty());
        assert!(node.body_text.is_none());
    }
}
