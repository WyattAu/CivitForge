#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::languages::LanguageRegistry;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TsNodeKind {
    Function,
    Method,
    Struct,
    Enum,
    Trait,
    Interface,
    Class,
    Module,
    Import,
    Variable,
    Constant,
    TypeAlias,
    IfStatement,
    LoopStatement,
    MatchStatement,
    CallExpression,
    BinaryExpression,
    UnaryExpression,
    FieldAccess,
    IndexExpression,
    Comment,
    Attribute,
    Annotation,
    Macro,
    GenericParam,
    ErrorNode,
}

impl TsNodeKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Method => "method",
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Trait => "trait",
            Self::Interface => "interface",
            Self::Class => "class",
            Self::Module => "module",
            Self::Import => "import",
            Self::Variable => "variable",
            Self::Constant => "constant",
            Self::TypeAlias => "type_alias",
            Self::IfStatement => "if_statement",
            Self::LoopStatement => "loop_statement",
            Self::MatchStatement => "match_statement",
            Self::CallExpression => "call_expression",
            Self::BinaryExpression => "binary_expression",
            Self::UnaryExpression => "unary_expression",
            Self::FieldAccess => "field_access",
            Self::IndexExpression => "index_expression",
            Self::Comment => "comment",
            Self::Attribute => "attribute",
            Self::Annotation => "annotation",
            Self::Macro => "macro",
            Self::GenericParam => "generic_param",
            Self::ErrorNode => "error",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TsNode {
    pub id: String,
    pub kind: TsNodeKind,
    pub name: String,
    pub text: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: usize,
    pub end_line: usize,
    pub children: Vec<TsNode>,
    pub metadata: HashMap<String, String>,
}

impl TsNode {
    pub fn new(
        kind: TsNodeKind,
        name: String,
        text: String,
        start_byte: usize,
        start_line: usize,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            kind,
            name,
            text,
            start_byte,
            end_byte: start_byte,
            start_line,
            end_line: start_line,
            children: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseResult {
    pub root: Vec<TsNode>,
    pub error_count: usize,
    pub parse_time: Duration,
}

#[derive(Debug, Clone)]
pub struct ParseOptions {
    pub track_comments: bool,
    pub track_whitespace: bool,
    pub max_depth: usize,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            track_comments: true,
            track_whitespace: false,
            max_depth: 64,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Keyword,
    Identifier,
    String,
    Number,
    Operator,
    DelimiterOpen(char),
    DelimiterClose(char),
    Punctuation,
    Comment,
    Whitespace,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    text: String,
    offset: usize,
    line: usize,
}

pub struct TreeSitterParser {
    language_registry: LanguageRegistry,
}

impl TreeSitterParser {
    pub fn new() -> Self {
        Self {
            language_registry: LanguageRegistry::new(),
        }
    }

    pub fn parse(&self, source: &str, language: &str) -> ParseResult {
        self.parse_with_options(source, language, ParseOptions::default())
    }

    pub fn parse_with_options(
        &self,
        source: &str,
        language: &str,
        opts: ParseOptions,
    ) -> ParseResult {
        let start = Instant::now();
        let mut error_count = 0usize;

        let lang_def = match self.language_registry.get(language) {
            Some(l) => l,
            None => {
                return ParseResult {
                    root: Vec::new(),
                    error_count: 1,
                    parse_time: start.elapsed(),
                };
            }
        };

        let line_offsets = compute_line_offsets(source);
        let tokens = tokenize(source, lang_def, &opts);
        let structures = build_structures(&tokens, lang_def, &mut error_count);
        let root = annotate_semantics(structures, &tokens, source, lang_def, &line_offsets);

        ParseResult {
            root,
            error_count,
            parse_time: start.elapsed(),
        }
    }

    pub fn language_registry(&self) -> &LanguageRegistry {
        &self.language_registry
    }
}

impl Default for TreeSitterParser {
    fn default() -> Self {
        Self::new()
    }
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

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn tokenize(
    source: &str,
    lang: &super::languages::LanguageDefinition,
    opts: &ParseOptions,
) -> Vec<Token> {
    let mut tokens = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0usize;
    let mut current_line = 0usize;
    let len = bytes.len();

    let keyword_set: std::collections::HashSet<&str> =
        lang.keywords.iter().map(|s| s.as_str()).collect();
    let type_keyword_set: std::collections::HashSet<&str> =
        lang.type_keywords.iter().map(|s| s.as_str()).collect();

    let open_set: std::collections::HashSet<char> = lang.brackets.iter().map(|(o, _)| *o).collect();
    let close_set: std::collections::HashSet<char> =
        lang.brackets.iter().map(|(_, c)| *c).collect();
    let bracket_map: std::collections::HashMap<char, char> =
        lang.brackets.iter().cloned().collect();

    let mut block_comment_start: Option<usize> = None;
    let mut block_end_marker: Option<String> = None;

    while i < len {
        let ch = source[i..].chars().next().expect("non-empty string");
        let ch_len = ch.len_utf8();

        if ch == '\n' {
            current_line += 1;
            i += ch_len;
            continue;
        }

        if ch.is_whitespace() {
            if opts.track_whitespace {
                let start = i;
                while i < len
                    && source[i..]
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_whitespace())
                {
                    if source[i..].starts_with('\n') {
                        current_line += 1;
                    }
                    i += source[i..].chars().next().expect("non-empty string").len_utf8();
                }
                tokens.push(Token {
                    kind: TokenKind::Whitespace,
                    text: source[start..i].to_string(),
                    offset: start,
                    line: current_line,
                });
            } else {
                i += ch_len;
            }
            continue;
        }

        if block_comment_start.is_some() {
            if let Some(ref be) = block_end_marker {
                let rest = &source[i..];
                if rest.starts_with(be.as_str()) {
                    if opts.track_comments {
                        tokens.push(Token {
                            kind: TokenKind::Comment,
                            text: be.clone(),
                            offset: i,
                            line: current_line,
                        });
                    }
                    i += be.len();
                    block_comment_start = None;
                    block_end_marker = None;
                    continue;
                }
            }
            i += ch_len;
            continue;
        }

        if ch == '"' || ch == '\'' {
            let quote = ch;
            let start = i;
            i += ch_len;

            let mut escaped = false;
            while i < len {
                let c = source[i..].chars().next().expect("non-empty string");
                let c_len = c.len_utf8();
                if c == '\n' {
                    current_line += 1;
                }
                if c == '\\' && !escaped {
                    escaped = true;
                    i += c_len;
                    continue;
                }
                if c == quote && !escaped {
                    i += c_len;
                    break;
                }
                escaped = false;
                i += c_len;
            }

            tokens.push(Token {
                kind: TokenKind::String,
                text: source[start..i].to_string(),
                offset: start,
                line: current_line,
            });
            continue;
        }

        let rest = &source[i..];
        let mut found_comment = false;
        for style in &lang.comment_styles {
            if let Some(ref lp) = style.line
                && rest.starts_with(lp.as_str())
            {
                if opts.track_comments {
                    let end = rest.find('\n').unwrap_or(rest.len());
                    let comment_text = rest[..end].trim_end().to_string();
                    tokens.push(Token {
                        kind: TokenKind::Comment,
                        text: comment_text,
                        offset: i,
                        line: current_line,
                    });
                }
                i += rest.find('\n').map_or(rest.len(), |n| n + 1);
                found_comment = true;
                break;
            }
            if let (Some(bs), Some(be)) = (&style.block_start, &style.block_end)
                && rest.starts_with(bs.as_str())
            {
                block_comment_start = Some(i);
                block_end_marker = Some(be.clone());
                if opts.track_comments {
                    tokens.push(Token {
                        kind: TokenKind::Comment,
                        text: bs.clone(),
                        offset: i,
                        line: current_line,
                    });
                }
                i += bs.len();
                found_comment = true;
                break;
            }
        }
        if found_comment {
            continue;
        }

        if open_set.contains(&ch) {
            tokens.push(Token {
                kind: TokenKind::DelimiterOpen(ch),
                text: ch.to_string(),
                offset: i,
                line: current_line,
            });
            i += ch_len;
            continue;
        }

        if close_set.contains(&ch) {
            tokens.push(Token {
                kind: TokenKind::DelimiterClose(ch),
                text: ch.to_string(),
                offset: i,
                line: current_line,
            });
            i += ch_len;
            continue;
        }

        if ch == ',' || ch == ';' || ch == ':' {
            tokens.push(Token {
                kind: TokenKind::Punctuation,
                text: ch.to_string(),
                offset: i,
                line: current_line,
            });
            i += ch_len;
            continue;
        }

        if is_word_char(ch) {
            let start = i;
            while i < len && source[i..].chars().next().is_some_and(is_word_char) {
                i += source[i..].chars().next().expect("non-empty string").len_utf8();
            }
            let word = &source[start..i];

            let kind = if keyword_set.contains(word) || type_keyword_set.contains(word) {
                TokenKind::Keyword
            } else if word.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                TokenKind::Number
            } else {
                TokenKind::Identifier
            };

            tokens.push(Token {
                kind,
                text: word.to_string(),
                offset: start,
                line: current_line,
            });
            continue;
        }

        tokens.push(Token {
            kind: TokenKind::Operator,
            text: ch.to_string(),
            offset: i,
            line: current_line,
        });
        i += ch_len;
    }

    let _ = bracket_map;
    tokens
}

#[derive(Clone)]
struct Structure {
    prefix_tokens: Vec<Token>,
    open_token_idx: usize,
    close_token_idx: usize,
    open_offset: usize,
    close_offset: usize,
    open_line: usize,
    close_line: usize,
}

fn build_structures(
    tokens: &[Token],
    lang: &super::languages::LanguageDefinition,
    error_count: &mut usize,
) -> Vec<Structure> {
    let mut structures = Vec::new();
    let bracket_map: HashMap<char, char> = lang.brackets.iter().cloned().collect();
    let mut stack: Vec<(usize, char)> = Vec::new();

    for (i, token) in tokens.iter().enumerate() {
        match &token.kind {
            TokenKind::DelimiterOpen(ch) => {
                stack.push((i, *ch));
            }
            TokenKind::DelimiterClose(ch) => {
                let expected_open = bracket_map
                    .iter()
                    .find(|(_, c)| **c == *ch)
                    .map(|(o, _)| *o);

                let match_idx = stack.iter().rposition(|(_, o)| Some(*o) == expected_open);

                match match_idx {
                    Some(idx) => {
                        let (open_i, _) = stack.remove(idx);
                        let open_t = &tokens[open_i];

                        let mut prefix_start = open_i;
                        let mut bracket_depth = 0i32;
                        while prefix_start > 0 {
                            let prev = &tokens[prefix_start - 1];
                            match &prev.kind {
                                TokenKind::DelimiterClose(_) => {
                                    bracket_depth += 1;
                                    prefix_start -= 1;
                                    continue;
                                }
                                TokenKind::DelimiterOpen(_) if bracket_depth > 0 => {
                                    bracket_depth -= 1;
                                    prefix_start -= 1;
                                    continue;
                                }
                                TokenKind::DelimiterOpen(_) => {
                                    break;
                                }
                                _ => {
                                    if bracket_depth > 0 {
                                        prefix_start -= 1;
                                        continue;
                                    }
                                    let is_part_of_prefix = matches!(
                                        prev.kind,
                                        TokenKind::Keyword
                                            | TokenKind::Identifier
                                            | TokenKind::Operator
                                            | TokenKind::Punctuation
                                            | TokenKind::Comment
                                            | TokenKind::Number
                                    ) && (prev.line == open_t.line
                                        || prev.line + 1 == open_t.line);
                                    if is_part_of_prefix {
                                        prefix_start -= 1;
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }

                        let prefix_tokens: Vec<Token> = tokens[prefix_start..open_i].to_vec();

                        structures.push(Structure {
                            prefix_tokens,
                            open_token_idx: open_i,
                            close_token_idx: i,
                            open_offset: tokens[prefix_start].offset,
                            close_offset: token.offset,
                            open_line: tokens[prefix_start].line,
                            close_line: token.line,
                        });
                    }
                    None => {
                        *error_count += 1;
                    }
                }
            }
            _ => {}
        }
    }

    *error_count += stack.len();

    structures.sort_by_key(|s| s.open_token_idx);
    structures
}

fn annotate_semantics(
    structures: Vec<Structure>,
    tokens: &[Token],
    source: &str,
    lang: &super::languages::LanguageDefinition,
    _line_offsets: &[usize],
) -> Vec<TsNode> {
    let mut nodes = Vec::new();
    let type_keyword_set: std::collections::HashSet<&str> =
        lang.type_keywords.iter().map(|s| s.as_str()).collect();

    for (si, structure) in structures.iter().enumerate() {
        let inner: Vec<&Token> = tokens[(structure.open_token_idx + 1)..structure.close_token_idx]
            .iter()
            .collect();
        let mut head: Vec<&Token> = structure.prefix_tokens.iter().collect();
        head.extend(
            inner
                .iter()
                .take_while(|t| !matches!(t.kind, TokenKind::DelimiterOpen(_))),
        );

        if let Some(annotation) = try_parse_annotation(&head) {
            let end = structure.close_offset
                + source[structure.close_offset..]
                    .chars()
                    .next()
                    .map_or(1, |c| c.len_utf8());
            let text = source[structure.open_offset..end].to_string();
            let mut node = TsNode::new(
                TsNodeKind::Annotation,
                annotation,
                text,
                structure.open_offset,
                structure.open_line + 1,
            );
            node.end_byte = end;
            node.end_line = structure.close_line + 1;
            nodes.push(node);
            continue;
        }

        if let Some((kind, name)) = try_parse_type_def(&head, &type_keyword_set, lang) {
            let end = structure.close_offset
                + source[structure.close_offset..]
                    .chars()
                    .next()
                    .map_or(1, |c| c.len_utf8());
            let text = source[structure.open_offset..end].to_string();
            let mut node = TsNode::new(
                kind,
                name,
                text,
                structure.open_offset,
                structure.open_line + 1,
            );
            node.end_byte = end;
            node.end_line = structure.close_line + 1;
            let children = build_child_structures(si, &structures);
            let child_nodes = annotate_semantics(children, tokens, source, lang, _line_offsets);
            node.children = child_nodes;
            nodes.push(node);
            continue;
        }

        if let Some((name, is_method)) = try_parse_function(&head, lang) {
            let end = structure.close_offset
                + source[structure.close_offset..]
                    .chars()
                    .next()
                    .map_or(1, |c| c.len_utf8());
            let text = source[structure.open_offset..end].to_string();
            let kind = if is_method {
                TsNodeKind::Method
            } else {
                TsNodeKind::Function
            };
            let mut node = TsNode::new(
                kind,
                name,
                text,
                structure.open_offset,
                structure.open_line + 1,
            );
            node.end_byte = end;
            node.end_line = structure.close_line + 1;
            let children = build_child_structures(si, &structures);
            let child_nodes = annotate_semantics(children, tokens, source, lang, _line_offsets);
            node.children = child_nodes;
            nodes.push(node);
            continue;
        }

        if let Some(ctrl_name) = try_parse_control_flow(&head, lang) {
            let end = structure.close_offset
                + source[structure.close_offset..]
                    .chars()
                    .next()
                    .map_or(1, |c| c.len_utf8());
            let text = source[structure.open_offset..end].to_string();
            let kind = match ctrl_name.as_str() {
                "if" | "elif" | "else" => TsNodeKind::IfStatement,
                "for" | "while" | "loop" | "do" => TsNodeKind::LoopStatement,
                "match" | "switch" | "case" | "select" => TsNodeKind::MatchStatement,
                _ => TsNodeKind::ErrorNode,
            };
            let mut node = TsNode::new(
                kind,
                ctrl_name,
                text,
                structure.open_offset,
                structure.open_line + 1,
            );
            node.end_byte = end;
            node.end_line = structure.close_line + 1;
            let children = build_child_structures(si, &structures);
            let child_nodes = annotate_semantics(children, tokens, source, lang, _line_offsets);
            node.children = child_nodes;
            nodes.push(node);
            continue;
        }
    }

    for (i, token) in tokens.iter().enumerate() {
        if let TokenKind::Keyword = &token.kind
            && let Some((kind, name)) = try_parse_import(token.text.as_str(), lang)
        {
            let end_offset = token.offset + token.text.len();
            let mut node =
                TsNode::new(kind, name, token.text.clone(), token.offset, token.line + 1);
            node.end_byte = end_offset;
            node.end_line = token.line + 1;
            let rest_tokens: Vec<&Token> = tokens[i + 1..]
                .iter()
                .take_while(|t| !matches!(t.kind, TokenKind::DelimiterOpen(_)))
                .filter(|t| {
                    matches!(
                        t.kind,
                        TokenKind::Identifier
                            | TokenKind::Keyword
                            | TokenKind::Punctuation
                            | TokenKind::Operator
                    )
                })
                .collect();
            if let Some(last) = rest_tokens.last() {
                node.text = source[node.start_byte..last.offset + last.text.len()].to_string();
                node.end_byte = last.offset + last.text.len();
            }
            nodes.push(node);
        }
    }

    nodes.sort_by_key(|n| n.start_byte);
    nodes
}

fn try_parse_import(
    keyword: &str,
    _lang: &super::languages::LanguageDefinition,
) -> Option<(TsNodeKind, String)> {
    match keyword {
        "use" | "import" | "from" | "require" | "include" => {
            Some((TsNodeKind::Import, keyword.to_string()))
        }
        _ => None,
    }
}

fn build_child_structures(parent_idx: usize, all_structures: &[Structure]) -> Vec<Structure> {
    let parent = &all_structures[parent_idx];
    let parent_start = parent.open_token_idx;
    let parent_end = parent.close_token_idx;

    let mut children = Vec::new();
    for (ci, s) in all_structures.iter().enumerate() {
        if s.open_token_idx > parent_start && s.close_token_idx < parent_end {
            let mut has_parent = false;
            for (oi, other) in all_structures.iter().enumerate() {
                if oi != ci
                    && other.open_token_idx < s.open_token_idx
                    && other.close_token_idx > s.close_token_idx
                    && other.open_token_idx > parent_start
                    && other.close_token_idx < parent_end
                {
                    has_parent = true;
                    break;
                }
            }
            if !has_parent {
                children.push(s.clone());
            }
        }
    }

    children
}

fn try_parse_annotation(tokens: &[&Token]) -> Option<String> {
    if tokens.is_empty() {
        return None;
    }
    let first = &tokens[0];
    match &first.kind {
        TokenKind::Punctuation => {
            if first.text == "@" || first.text == "#" {
                let name: String = tokens
                    .iter()
                    .skip(1)
                    .take_while(|t| {
                        !matches!(t.kind, TokenKind::DelimiterOpen(_))
                            && !matches!(t.kind, TokenKind::Punctuation)
                    })
                    .map(|t| t.text.as_str())
                    .collect::<Vec<_>>()
                    .join("");
                if !name.is_empty() {
                    return Some(name);
                }
            }
            None
        }
        _ => None,
    }
}

fn try_parse_type_def(
    tokens: &[&Token],
    type_keywords: &std::collections::HashSet<&str>,
    lang: &super::languages::LanguageDefinition,
) -> Option<(TsNodeKind, String)> {
    if tokens.is_empty() {
        return None;
    }

    let modifiers: std::collections::HashSet<&str> = [
        "pub",
        "public",
        "private",
        "protected",
        "internal",
        "static",
        "final",
        "abstract",
        "sealed",
        "export",
        "open",
        "data",
        "readonly",
        "override",
        "opaque",
        "packed",
        "extern",
        "comptime",
        "threadlocal",
    ]
    .iter()
    .copied()
    .collect();

    let mut type_kw_idx = None;
    for (i, tok) in tokens.iter().enumerate() {
        if let TokenKind::Keyword = &tok.kind {
            if type_keywords.contains(tok.text.as_str()) {
                type_kw_idx = Some(i);
                break;
            }
            if modifiers.contains(tok.text.as_str()) {
                continue;
            }
            return None;
        }
        if let TokenKind::Identifier = &tok.kind {
            if modifiers.contains(tok.text.as_str()) {
                continue;
            }
            return None;
        }
        if matches!(tok.kind, TokenKind::Punctuation | TokenKind::Operator) {
            continue;
        }
    }

    let kw_idx = type_kw_idx?;
    let type_kw = &tokens[kw_idx];

    let mut resolved_kind = match type_kw.text.as_str() {
        "struct" => TsNodeKind::Struct,
        "enum" => TsNodeKind::Enum,
        "trait" => TsNodeKind::Trait,
        "interface" => TsNodeKind::Interface,
        "class" => TsNodeKind::Class,
        "type" | "typealias" => TsNodeKind::TypeAlias,
        "mod" | "module" | "package" | "namespace" => TsNodeKind::Module,
        "impl" => TsNodeKind::Trait,
        "record" => TsNodeKind::Struct,
        _ => return None,
    };

    if type_kw.text.as_str() == "type" || type_kw.text.as_str() == "typealias" {
        for t in tokens.iter().skip(kw_idx + 1) {
            if let TokenKind::Keyword = t.kind {
                if t.text == "struct" {
                    resolved_kind = TsNodeKind::Struct;
                } else if t.text == "enum" {
                    resolved_kind = TsNodeKind::Enum;
                } else if t.text == "interface" {
                    resolved_kind = TsNodeKind::Interface;
                }
                break;
            }
            if matches!(t.kind, TokenKind::DelimiterOpen(_)) {
                break;
            }
        }
    }

    if let Some(name) = tokens
        .iter()
        .skip(kw_idx + 1)
        .find(|t| matches!(t.kind, TokenKind::Identifier))
        .map(|t| t.text.clone())
        && !name.is_empty()
    {
        let _ = lang;
        return Some((resolved_kind, name));
    }

    None
}

fn try_parse_function(
    tokens: &[&Token],
    lang: &super::languages::LanguageDefinition,
) -> Option<(String, bool)> {
    if tokens.is_empty() {
        return None;
    }

    let func_keywords: std::collections::HashSet<&str> =
        ["fn", "func", "def", "fun", "function", "sub", "proc"]
            .iter()
            .copied()
            .collect();

    let modifiers: std::collections::HashSet<&str> = [
        "pub",
        "private",
        "protected",
        "internal",
        "static",
        "final",
        "abstract",
        "sealed",
        "export",
        "open",
        "override",
        "async",
        "inline",
        "suspend",
        "virtual",
        "synchronized",
        "native",
        "mut",
        "extern",
        "const",
        "where",
        "with",
        "operator",
        "infix",
        "prefix",
        "postfix",
        "let",
    ]
    .iter()
    .copied()
    .collect();

    let mut func_kw_idx = None;
    let mut is_method = false;

    for (i, tok) in tokens.iter().enumerate() {
        if let TokenKind::Operator = &tok.kind
            && tok.text == "."
        {
            is_method = true;
            continue;
        }
        if let TokenKind::Identifier = &tok.kind {
            if func_keywords.contains(tok.text.as_str()) {
                func_kw_idx = Some(i);
                break;
            }
            continue;
        }
        if let TokenKind::Keyword = &tok.kind {
            if func_keywords.contains(tok.text.as_str()) {
                func_kw_idx = Some(i);
                break;
            }
            if !modifiers.contains(tok.text.as_str()) {
                return None;
            }
        }
    }

    let kw_idx = func_kw_idx?;

    if let Some(name) = tokens
        .iter()
        .skip(kw_idx + 1)
        .find(|t| matches!(t.kind, TokenKind::Identifier))
        .map(|t| t.text.clone())
    {
        if !is_method {
            let has_self = lang.keywords.iter().any(|k| k == "self" || k == "this");
            if has_self
                && tokens
                    .iter()
                    .take(kw_idx)
                    .any(|t| t.text == "self" || t.text == "Self" || t.text == "this")
            {
                is_method = true;
            }
        }
        return Some((name, is_method));
    }

    None
}

fn try_parse_control_flow(
    tokens: &[&Token],
    lang: &super::languages::LanguageDefinition,
) -> Option<String> {
    if tokens.is_empty() {
        return None;
    }

    let control_keywords: std::collections::HashSet<&str> = [
        "if", "else", "elif", "for", "while", "loop", "do", "match", "switch", "case", "select",
        "try", "catch", "finally", "guard", "when",
    ]
    .iter()
    .copied()
    .collect();

    let modifiers: std::collections::HashSet<&str> = ["async", "await", "unsafe", "checked"]
        .iter()
        .copied()
        .collect();

    for tok in tokens.iter() {
        if let TokenKind::Keyword = &tok.kind {
            if modifiers.contains(tok.text.as_str()) {
                continue;
            }
            if (control_keywords.contains(tok.text.as_str())
                || lang.keywords.iter().any(|k| k == &tok.text))
                && let "if" | "else" | "elif" | "for" | "while" | "loop" | "do" | "match" | "switch"
                | "case" | "select" | "try" | "catch" | "finally" | "guard" | "when" =
                    tok.text.as_str()
            {
                return Some(tok.text.clone());
            }
        }
        if matches!(
            tok.kind,
            TokenKind::Identifier
                | TokenKind::Operator
                | TokenKind::Number
                | TokenKind::Punctuation
        ) {
            continue;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rust_function() {
        let parser = TreeSitterParser::new();
        let source = "fn hello() {\n    println!(\"world\");\n}\nfn world() {}";
        let result = parser.parse(source, "rust");
        assert!(result.error_count == 0, "errors: {}", result.error_count);
        let funcs: Vec<&TsNode> = result
            .root
            .iter()
            .filter(|n| n.kind == TsNodeKind::Function)
            .collect();
        assert!(!funcs.is_empty(), "found {} functions", funcs.len());
        assert_eq!(funcs[0].name, "hello");
    }

    #[test]
    fn test_parse_rust_struct() {
        let parser = TreeSitterParser::new();
        let source = "struct Point {\n    x: i32,\n    y: i32,\n}";
        let result = parser.parse(source, "rust");
        let structs: Vec<&TsNode> = result
            .root
            .iter()
            .filter(|n| n.kind == TsNodeKind::Struct)
            .collect();
        assert_eq!(structs.len(), 1, "found {} structs", structs.len());
        assert_eq!(structs[0].name, "Point");
    }

    #[test]
    fn test_parse_rust_enum() {
        let parser = TreeSitterParser::new();
        let source = "enum Color {\n    Red,\n    Green,\n    Blue,\n}";
        let result = parser.parse(source, "rust");
        let enums: Vec<&TsNode> = result
            .root
            .iter()
            .filter(|n| n.kind == TsNodeKind::Enum)
            .collect();
        assert_eq!(enums.len(), 1);
        assert_eq!(enums[0].name, "Color");
    }

    #[test]
    fn test_parse_rust_trait() {
        let parser = TreeSitterParser::new();
        let source = "trait Drawable {\n    fn draw(&self);\n}";
        let result = parser.parse(source, "rust");
        let traits: Vec<&TsNode> = result
            .root
            .iter()
            .filter(|n| n.kind == TsNodeKind::Trait)
            .collect();
        assert_eq!(traits.len(), 1);
        assert_eq!(traits[0].name, "Drawable");
    }

    #[test]
    fn test_parse_rust_if_statement() {
        let parser = TreeSitterParser::new();
        let source = "if x > 0 {\n    println!(\"positive\");\n}";
        let result = parser.parse(source, "rust");
        let ifs: Vec<&TsNode> = result
            .root
            .iter()
            .filter(|n| n.kind == TsNodeKind::IfStatement)
            .collect();
        assert_eq!(ifs.len(), 1);
    }

    #[test]
    fn test_parse_rust_loop() {
        let parser = TreeSitterParser::new();
        let source = "for i in 0..10 {\n    println!(\"{}\");\n}\nwhile x > 0 {\n    x -= 1;\n}\nloop {\n    break;\n}";
        let result = parser.parse(source, "rust");
        let loops: Vec<&TsNode> = result
            .root
            .iter()
            .filter(|n| n.kind == TsNodeKind::LoopStatement)
            .collect();
        assert!(loops.len() >= 2, "found {} loops", loops.len());
    }

    #[test]
    fn test_parse_rust_match() {
        let parser = TreeSitterParser::new();
        let source = "match x {\n    Some(v) => v,\n    None => 0,\n}";
        let result = parser.parse(source, "rust");
        let matches: Vec<&TsNode> = result
            .root
            .iter()
            .filter(|n| n.kind == TsNodeKind::MatchStatement)
            .collect();
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_parse_nested_expressions() {
        let parser = TreeSitterParser::new();
        let source = "fn outer() {\n    fn inner() {\n        let x = 1;\n    }\n}";
        let result = parser.parse(source, "rust");
        let funcs: Vec<&TsNode> = result
            .root
            .iter()
            .filter(|n| n.kind == TsNodeKind::Function)
            .collect();
        assert!(!funcs.is_empty());
        assert_eq!(funcs[0].name, "outer");
    }

    #[test]
    fn test_parse_use_statement() {
        let parser = TreeSitterParser::new();
        let source = "use std::collections::HashMap;\nuse anyhow::{Result, Context};";
        let result = parser.parse(source, "rust");
        let imports: Vec<&TsNode> = result
            .root
            .iter()
            .filter(|n| n.kind == TsNodeKind::Import)
            .collect();
        assert!(!imports.is_empty(), "found {} imports", imports.len());
    }

    #[test]
    fn test_parse_pub_struct() {
        let parser = TreeSitterParser::new();
        let source = "pub struct Config {\n    name: String,\n}";
        let result = parser.parse(source, "rust");
        let structs: Vec<&TsNode> = result
            .root
            .iter()
            .filter(|n| n.kind == TsNodeKind::Struct)
            .collect();
        assert_eq!(structs.len(), 1);
        assert_eq!(structs[0].name, "Config");
    }

    #[test]
    fn test_parse_async_function() {
        let parser = TreeSitterParser::new();
        let source = "pub async fn handle_request() {\n    Ok(())\n}";
        let result = parser.parse(source, "rust");
        let funcs: Vec<&TsNode> = result
            .root
            .iter()
            .filter(|n| n.kind == TsNodeKind::Function)
            .collect();
        assert!(!funcs.is_empty(), "found {} funcs", funcs.len());
        assert_eq!(funcs[0].name, "handle_request");
    }

    #[test]
    fn test_error_recovery_unmatched_brace() {
        let parser = TreeSitterParser::new();
        let source = "fn foo() {\n    if true {\n        bar()\n";
        let result = parser.parse(source, "rust");
        assert!(result.error_count > 0);
    }

    #[test]
    fn test_error_recovery_extra_close() {
        let parser = TreeSitterParser::new();
        let source = "fn foo() {\n    bar()\n}\n}";
        let result = parser.parse(source, "rust");
        assert!(result.error_count > 0);
    }

    #[test]
    fn test_parse_unknown_language() {
        let parser = TreeSitterParser::new();
        let result = parser.parse("print('hello')", "brainfuck");
        assert_eq!(result.error_count, 1);
        assert!(result.root.is_empty());
    }

    #[test]
    fn test_parse_go_function() {
        let parser = TreeSitterParser::new();
        let source = "func Add(a int, b int) int {\n    return a + b\n}";
        let result = parser.parse(source, "go");
        let funcs: Vec<&TsNode> = result
            .root
            .iter()
            .filter(|n| n.kind == TsNodeKind::Function)
            .collect();
        assert!(!funcs.is_empty(), "found {} funcs", funcs.len());
        assert_eq!(funcs[0].name, "Add");
    }

    #[test]
    fn test_parse_go_struct() {
        let parser = TreeSitterParser::new();
        let source = "type Point struct {\n    X int\n    Y int\n}";
        let result = parser.parse(source, "go");
        let structs: Vec<&TsNode> = result
            .root
            .iter()
            .filter(|n| n.kind == TsNodeKind::Struct)
            .collect();
        assert_eq!(structs.len(), 1, "found {} structs", structs.len());
        assert_eq!(structs[0].name, "Point");
    }

    #[test]
    fn test_parse_python_def() {
        let parser = TreeSitterParser::new();
        let source = "def hello():\n    print('world')";
        let result = parser.parse(source, "python");
        assert!(result.error_count == 0, "errors: {}", result.error_count);
    }

    #[test]
    fn test_parse_python_class() {
        let parser = TreeSitterParser::new();
        let source = "class Animal(object): pass";
        let result = parser.parse(source, "python");
        let classes: Vec<&TsNode> = result
            .root
            .iter()
            .filter(|n| matches!(n.kind, TsNodeKind::Class | TsNodeKind::Function))
            .collect();
        assert!(!classes.is_empty(), "found {} classes", classes.len());
    }

    #[test]
    fn test_parse_typescript_interface() {
        let parser = TreeSitterParser::new();
        let source = "interface User {\n    name: string;\n    age: number;\n}";
        let result = parser.parse(source, "typescript");
        let ifaces: Vec<&TsNode> = result
            .root
            .iter()
            .filter(|n| n.kind == TsNodeKind::Interface)
            .collect();
        assert_eq!(ifaces.len(), 1, "found {} interfaces", ifaces.len());
        assert_eq!(ifaces[0].name, "User");
    }

    #[test]
    fn test_parse_java_class() {
        let parser = TreeSitterParser::new();
        let source = "public class Main {\n    public static void main(String[] args) {}\n}";
        let result = parser.parse(source, "java");
        let classes: Vec<&TsNode> = result
            .root
            .iter()
            .filter(|n| n.kind == TsNodeKind::Class)
            .collect();
        assert_eq!(classes.len(), 1, "found {} classes", classes.len());
        assert_eq!(classes[0].name, "Main");
    }

    #[test]
    fn test_parse_time_measured() {
        let parser = TreeSitterParser::new();
        let source = "fn main() {}";
        let result = parser.parse(source, "rust");
        assert!(result.parse_time.as_nanos() > 0);
    }

    #[test]
    fn test_parse_options_track_comments() {
        let parser = TreeSitterParser::new();
        let source = "// this is a comment\nfn main() {}";
        let opts = ParseOptions {
            track_comments: true,
            track_whitespace: false,
            max_depth: 64,
        };
        let result = parser.parse_with_options(source, "rust", opts);
        assert!(result.root.iter().any(|n| n.kind == TsNodeKind::Function));
    }

    #[test]
    fn test_tokenization_basic() {
        let registry = LanguageRegistry::new();
        let rust = registry.get("rust").expect("key present");
        let opts = ParseOptions::default();
        let tokens = tokenize("fn main() { let x = 42; }", rust, &opts);
        assert!(
            tokens
                .iter()
                .any(|t| t.kind == TokenKind::Keyword && t.text == "fn")
        );
        assert!(
            tokens
                .iter()
                .any(|t| t.kind == TokenKind::Identifier && t.text == "main")
        );
        assert!(
            tokens
                .iter()
                .any(|t| t.kind == TokenKind::Number && t.text == "42")
        );
    }

    #[test]
    fn test_node_ids_unique() {
        let parser = TreeSitterParser::new();
        let source = "fn foo() {}\nfn bar() {}";
        let result = parser.parse(source, "rust");
        let ids: Vec<&str> = result.root.iter().map(|n| n.id.as_str()).collect();
        let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len());
    }

    #[test]
    fn test_node_byte_and_line_ranges() {
        let parser = TreeSitterParser::new();
        let source = "struct Foo {\n    x: i32,\n}\n";
        let result = parser.parse(source, "rust");
        let structs: Vec<&TsNode> = result
            .root
            .iter()
            .filter(|n| n.kind == TsNodeKind::Struct)
            .collect();
        assert_eq!(structs.len(), 1);
        let s = &structs[0];
        assert!(s.end_byte > s.start_byte);
        assert!(s.end_line >= s.start_line);
    }
}
