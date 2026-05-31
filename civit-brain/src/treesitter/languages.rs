#![forbid(unsafe_code)]

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LanguageDefinition {
    pub name: String,
    pub extensions: Vec<String>,
    pub keywords: Vec<String>,
    pub type_keywords: Vec<String>,
    pub string_delimiters: Vec<(char, char)>,
    pub comment_styles: Vec<CommentStyle>,
    pub brackets: Vec<(char, char)>,
    pub indentation: IndentationStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentationStyle {
    Spaces(usize),
    Tabs,
}

#[derive(Debug, Clone)]
pub struct CommentStyle {
    pub line: Option<String>,
    pub block_start: Option<String>,
    pub block_end: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LanguageRegistry {
    languages: HashMap<String, LanguageDefinition>,
}

impl LanguageRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            languages: HashMap::new(),
        };
        registry.register_builtin_languages();
        registry
    }

    pub fn register(&mut self, lang: LanguageDefinition) {
        let name = lang.name.clone();
        self.languages.insert(name, lang);
    }

    pub fn get(&self, name: &str) -> Option<&LanguageDefinition> {
        self.languages.get(name)
    }

    pub fn detect_by_extension(&self, ext: &str) -> Option<&LanguageDefinition> {
        let ext = ext.trim_start_matches('.');
        self.languages
            .values()
            .find(|lang| lang.extensions.iter().any(|e| e == ext))
    }

    pub fn supported_languages(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.languages.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    fn register_builtin_languages(&mut self) {
        self.register(rust_lang());
        self.register(go_lang());
        self.register(python_lang());
        self.register(typescript_lang());
        self.register(javascript_lang());
        self.register(java_lang());
        self.register(kotlin_lang());
        self.register(swift_lang());
        self.register(c_lang());
        self.register(cpp_lang());
        self.register(ruby_lang());
        self.register(php_lang());
        self.register(zig_lang());
        self.register(haskell_lang());
        self.register(scala_lang());
        self.register(shell_lang());
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! svec { ($($x:expr),* $(,)?) => { vec![$(String::from($x)),*] }; }

fn rust_lang() -> LanguageDefinition {
    LanguageDefinition {
        name: "rust".into(),
        extensions: svec!["rs"],
        keywords: svec![
            "fn", "let", "mut", "if", "else", "for", "while", "loop", "match", "return", "break",
            "continue", "in", "as", "ref", "move", "async", "await", "yield", "static", "const",
            "unsafe", "extern", "crate", "super", "self", "Self",
        ],
        type_keywords: svec![
            "struct", "enum", "trait", "impl", "type", "use", "mod", "where", "dyn", "box",
        ],
        string_delimiters: vec![('"', '"')],
        comment_styles: vec![CommentStyle {
            line: Some("//".into()),
            block_start: Some("/*".into()),
            block_end: Some("*/".into()),
        }],
        brackets: vec![('(', ')'), ('{', '}'), ('[', ']'), ('<', '>')],
        indentation: IndentationStyle::Spaces(4),
    }
}

fn go_lang() -> LanguageDefinition {
    LanguageDefinition {
        name: "go".into(),
        extensions: svec!["go"],
        keywords: svec![
            "break",
            "case",
            "chan",
            "const",
            "continue",
            "default",
            "defer",
            "else",
            "fallthrough",
            "for",
            "func",
            "go",
            "goto",
            "if",
            "import",
            "interface",
            "map",
            "package",
            "range",
            "return",
            "select",
            "struct",
            "switch",
            "type",
            "var",
        ],
        type_keywords: svec!["struct", "interface", "map", "chan", "func", "type"],
        string_delimiters: vec![('"', '"'), ('`', '`')],
        comment_styles: vec![CommentStyle {
            line: Some("//".into()),
            block_start: Some("/*".into()),
            block_end: Some("*/".into()),
        }],
        brackets: vec![('(', ')'), ('{', '}'), ('[', ']')],
        indentation: IndentationStyle::Tabs,
    }
}

fn python_lang() -> LanguageDefinition {
    LanguageDefinition {
        name: "python".into(),
        extensions: svec!["py", "pyi", "pyx"],
        keywords: svec![
            "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
            "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
            "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise",
            "return", "try", "while", "with", "yield",
        ],
        type_keywords: svec!["class", "def", "async", "type"],
        string_delimiters: vec![('"', '"'), ('\'', '\''), ('"', '"'), ('\'', '\'')],
        comment_styles: vec![CommentStyle {
            line: Some("#".into()),
            block_start: None,
            block_end: None,
        }],
        brackets: vec![('(', ')'), ('{', '}'), ('[', ']')],
        indentation: IndentationStyle::Spaces(4),
    }
}

fn typescript_lang() -> LanguageDefinition {
    LanguageDefinition {
        name: "typescript".into(),
        extensions: svec!["ts", "tsx"],
        keywords: svec![
            "break",
            "case",
            "catch",
            "class",
            "const",
            "continue",
            "debugger",
            "default",
            "delete",
            "do",
            "else",
            "enum",
            "export",
            "extends",
            "false",
            "finally",
            "for",
            "function",
            "if",
            "import",
            "in",
            "instanceof",
            "let",
            "new",
            "null",
            "return",
            "super",
            "switch",
            "this",
            "throw",
            "true",
            "try",
            "typeof",
            "var",
            "void",
            "while",
            "with",
            "yield",
            "async",
            "await",
            "of",
            "from",
            "as",
            "implements",
        ],
        type_keywords: svec![
            "interface",
            "type",
            "enum",
            "class",
            "namespace",
            "module",
            "declare",
            "abstract",
            "readonly",
            "keyof",
            "infer",
            "extends",
            "implements",
        ],
        string_delimiters: vec![('"', '"'), ('\'', '\''), ('`', '`')],
        comment_styles: vec![CommentStyle {
            line: Some("//".into()),
            block_start: Some("/*".into()),
            block_end: Some("*/".into()),
        }],
        brackets: vec![('(', ')'), ('{', '}'), ('[', ']'), ('<', '>')],
        indentation: IndentationStyle::Spaces(4),
    }
}

fn javascript_lang() -> LanguageDefinition {
    LanguageDefinition {
        name: "javascript".into(),
        extensions: svec!["js", "jsx", "mjs", "cjs"],
        keywords: svec![
            "break",
            "case",
            "catch",
            "class",
            "const",
            "continue",
            "debugger",
            "default",
            "delete",
            "do",
            "else",
            "export",
            "extends",
            "false",
            "finally",
            "for",
            "function",
            "if",
            "import",
            "in",
            "instanceof",
            "let",
            "new",
            "null",
            "return",
            "super",
            "switch",
            "this",
            "throw",
            "true",
            "try",
            "typeof",
            "var",
            "void",
            "while",
            "with",
            "yield",
            "async",
            "await",
            "of",
            "from",
        ],
        type_keywords: svec!["class", "function"],
        string_delimiters: vec![('"', '"'), ('\'', '\''), ('`', '`')],
        comment_styles: vec![CommentStyle {
            line: Some("//".into()),
            block_start: Some("/*".into()),
            block_end: Some("*/".into()),
        }],
        brackets: vec![('(', ')'), ('{', '}'), ('[', ']')],
        indentation: IndentationStyle::Spaces(4),
    }
}

fn java_lang() -> LanguageDefinition {
    LanguageDefinition {
        name: "java".into(),
        extensions: svec!["java"],
        keywords: svec![
            "abstract",
            "assert",
            "break",
            "case",
            "catch",
            "class",
            "const",
            "continue",
            "default",
            "do",
            "else",
            "enum",
            "extends",
            "final",
            "finally",
            "for",
            "if",
            "implements",
            "import",
            "instanceof",
            "interface",
            "native",
            "new",
            "package",
            "private",
            "protected",
            "public",
            "return",
            "static",
            "strictfp",
            "super",
            "switch",
            "synchronized",
            "this",
            "throw",
            "throws",
            "transient",
            "try",
            "void",
            "volatile",
            "while",
            "record",
            "sealed",
            "permits",
            "yield",
        ],
        type_keywords: svec![
            "class",
            "interface",
            "enum",
            "record",
            "sealed",
            "abstract",
            "implements",
            "extends",
        ],
        string_delimiters: vec![('"', '"')],
        comment_styles: vec![
            CommentStyle {
                line: Some("//".into()),
                block_start: Some("/*".into()),
                block_end: Some("*/".into()),
            },
            CommentStyle {
                line: None,
                block_start: Some("/**".into()),
                block_end: Some("*/".into()),
            },
        ],
        brackets: vec![('(', ')'), ('{', '}'), ('[', ']'), ('<', '>')],
        indentation: IndentationStyle::Spaces(4),
    }
}

fn kotlin_lang() -> LanguageDefinition {
    LanguageDefinition {
        name: "kotlin".into(),
        extensions: svec!["kt", "kts"],
        keywords: svec![
            "as",
            "break",
            "class",
            "continue",
            "do",
            "else",
            "false",
            "for",
            "fun",
            "if",
            "in",
            "interface",
            "is",
            "null",
            "object",
            "package",
            "return",
            "super",
            "this",
            "throw",
            "true",
            "try",
            "typealias",
            "val",
            "var",
            "when",
            "while",
            "by",
            "catch",
            "constructor",
            "delegate",
            "dynamic",
            "field",
            "get",
            "init",
            "it",
            "override",
            "private",
            "protected",
            "public",
            "internal",
            "operator",
            "property",
            "receiver",
            "set",
            "suspend",
            "yield",
            "data",
            "sealed",
            "enum",
            "annotation",
        ],
        type_keywords: svec![
            "class",
            "interface",
            "object",
            "enum",
            "typealias",
            "sealed",
            "data",
            "annotation",
        ],
        string_delimiters: vec![('"', '"')],
        comment_styles: vec![CommentStyle {
            line: Some("//".into()),
            block_start: Some("/*".into()),
            block_end: Some("*/".into()),
        }],
        brackets: vec![('(', ')'), ('{', '}'), ('[', ']'), ('<', '>')],
        indentation: IndentationStyle::Spaces(4),
    }
}

fn swift_lang() -> LanguageDefinition {
    LanguageDefinition {
        name: "swift".into(),
        extensions: svec!["swift"],
        keywords: svec![
            "break",
            "case",
            "catch",
            "class",
            "continue",
            "default",
            "defer",
            "do",
            "else",
            "enum",
            "extension",
            "fallthrough",
            "false",
            "fileprivate",
            "for",
            "func",
            "guard",
            "if",
            "import",
            "in",
            "init",
            "inout",
            "internal",
            "is",
            "let",
            "nil",
            "operator",
            "private",
            "protocol",
            "public",
            "repeat",
            "return",
            "self",
            "static",
            "struct",
            "subscript",
            "super",
            "switch",
            "throw",
            "throws",
            "true",
            "try",
            "typealias",
            "var",
            "where",
            "while",
            "async",
            "await",
            "some",
            "any",
        ],
        type_keywords: svec![
            "class",
            "struct",
            "enum",
            "protocol",
            "extension",
            "typealias",
        ],
        string_delimiters: vec![('"', '"')],
        comment_styles: vec![CommentStyle {
            line: Some("//".into()),
            block_start: Some("/*".into()),
            block_end: Some("*/".into()),
        }],
        brackets: vec![('(', ')'), ('{', '}'), ('[', ']'), ('<', '>')],
        indentation: IndentationStyle::Spaces(4),
    }
}

fn c_lang() -> LanguageDefinition {
    LanguageDefinition {
        name: "c".into(),
        extensions: svec!["c", "h"],
        keywords: svec![
            "auto",
            "break",
            "case",
            "char",
            "const",
            "continue",
            "default",
            "do",
            "double",
            "else",
            "enum",
            "extern",
            "float",
            "for",
            "goto",
            "if",
            "inline",
            "int",
            "long",
            "register",
            "restrict",
            "return",
            "short",
            "signed",
            "sizeof",
            "static",
            "struct",
            "switch",
            "typedef",
            "union",
            "unsigned",
            "void",
            "volatile",
            "while",
            "_Alignas",
            "_Alignof",
            "_Atomic",
            "_Bool",
            "_Complex",
            "_Generic",
            "_Imaginary",
            "_Noreturn",
            "_Static_assert",
            "_Thread_local",
        ],
        type_keywords: svec!["struct", "enum", "union", "typedef"],
        string_delimiters: vec![('"', '"'), ('\'', '\'')],
        comment_styles: vec![CommentStyle {
            line: Some("//".into()),
            block_start: Some("/*".into()),
            block_end: Some("*/".into()),
        }],
        brackets: vec![('(', ')'), ('{', '}'), ('[', ']')],
        indentation: IndentationStyle::Spaces(4),
    }
}

fn cpp_lang() -> LanguageDefinition {
    LanguageDefinition {
        name: "cpp".into(),
        extensions: svec!["cpp", "cc", "cxx", "hpp", "hxx", "hh"],
        keywords: svec![
            "alignas",
            "alignof",
            "and",
            "and_eq",
            "asm",
            "auto",
            "bitand",
            "bitor",
            "bool",
            "break",
            "case",
            "catch",
            "char",
            "char8_t",
            "char16_t",
            "char32_t",
            "class",
            "compl",
            "concept",
            "const",
            "consteval",
            "constexpr",
            "constinit",
            "const_cast",
            "continue",
            "co_await",
            "co_return",
            "co_yield",
            "decltype",
            "default",
            "delete",
            "do",
            "double",
            "dynamic_cast",
            "else",
            "enum",
            "explicit",
            "export",
            "extern",
            "false",
            "final",
            "float",
            "for",
            "friend",
            "goto",
            "if",
            "inline",
            "int",
            "long",
            "mutable",
            "namespace",
            "new",
            "noexcept",
            "not",
            "not_eq",
            "nullptr",
            "operator",
            "or",
            "or_eq",
            "override",
            "private",
            "protected",
            "public",
            "register",
            "reinterpret_cast",
            "requires",
            "return",
            "short",
            "signed",
            "sizeof",
            "static",
            "static_assert",
            "static_cast",
            "struct",
            "switch",
            "template",
            "this",
            "thread_local",
            "throw",
            "true",
            "try",
            "typedef",
            "typeid",
            "typename",
            "union",
            "unsigned",
            "using",
            "virtual",
            "void",
            "volatile",
            "while",
            "xor",
            "xor_eq",
        ],
        type_keywords: svec![
            "class",
            "struct",
            "enum",
            "union",
            "typedef",
            "template",
            "namespace",
            "concept",
            "requires",
        ],
        string_delimiters: vec![('"', '"'), ('\'', '\'')],
        comment_styles: vec![CommentStyle {
            line: Some("//".into()),
            block_start: Some("/*".into()),
            block_end: Some("*/".into()),
        }],
        brackets: vec![('(', ')'), ('{', '}'), ('[', ']'), ('<', '>')],
        indentation: IndentationStyle::Spaces(4),
    }
}

fn ruby_lang() -> LanguageDefinition {
    LanguageDefinition {
        name: "ruby".into(),
        extensions: svec!["rb"],
        keywords: svec![
            "alias", "and", "begin", "break", "case", "class", "def", "defined", "do", "else",
            "elsif", "end", "ensure", "false", "for", "if", "in", "module", "next", "nil", "not",
            "or", "redo", "rescue", "retry", "return", "self", "super", "then", "true", "undef",
            "unless", "until", "when", "while", "yield",
        ],
        type_keywords: svec!["class", "module", "def", "begin", "rescue", "ensure"],
        string_delimiters: vec![('"', '"'), ('\'', '\'')],
        comment_styles: vec![CommentStyle {
            line: Some("#".into()),
            block_start: Some("=begin".into()),
            block_end: Some("=end".into()),
        }],
        brackets: vec![('(', ')'), ('{', '}'), ('[', ']')],
        indentation: IndentationStyle::Spaces(2),
    }
}

fn php_lang() -> LanguageDefinition {
    LanguageDefinition {
        name: "php".into(),
        extensions: svec!["php", "phtml"],
        keywords: svec![
            "abstract",
            "and",
            "array",
            "as",
            "break",
            "callable",
            "case",
            "catch",
            "class",
            "clone",
            "const",
            "continue",
            "declare",
            "default",
            "die",
            "do",
            "echo",
            "else",
            "elseif",
            "empty",
            "enddeclare",
            "endfor",
            "endforeach",
            "endif",
            "endswitch",
            "endwhile",
            "eval",
            "exit",
            "extends",
            "final",
            "finally",
            "fn",
            "for",
            "foreach",
            "function",
            "global",
            "goto",
            "if",
            "implements",
            "include",
            "include_once",
            "instanceof",
            "insteadof",
            "interface",
            "isset",
            "list",
            "match",
            "namespace",
            "new",
            "or",
            "print",
            "private",
            "protected",
            "public",
            "readonly",
            "require",
            "require_once",
            "return",
            "static",
            "switch",
            "throw",
            "trait",
            "try",
            "unset",
            "use",
            "var",
            "while",
            "xor",
            "yield",
            "yield from",
        ],
        type_keywords: svec![
            "class",
            "interface",
            "trait",
            "enum",
            "function",
            "namespace",
            "readonly",
        ],
        string_delimiters: vec![('"', '"'), ('\'', '\'')],
        comment_styles: vec![
            CommentStyle {
                line: Some("//".into()),
                block_start: Some("/*".into()),
                block_end: Some("*/".into()),
            },
            CommentStyle {
                line: Some("#".into()),
                block_start: None,
                block_end: None,
            },
        ],
        brackets: vec![('(', ')'), ('{', '}'), ('[', ']')],
        indentation: IndentationStyle::Spaces(4),
    }
}

fn zig_lang() -> LanguageDefinition {
    LanguageDefinition {
        name: "zig".into(),
        extensions: svec!["zig"],
        keywords: svec![
            "const",
            "var",
            "fn",
            "if",
            "else",
            "switch",
            "for",
            "while",
            "break",
            "continue",
            "return",
            "defer",
            "errdefer",
            "try",
            "catch",
            "unreachable",
            "and",
            "or",
            "orelse",
            "resume",
            "await",
            "suspend",
            "cancel",
            "async",
        ],
        type_keywords: svec![
            "const",
            "var",
            "fn",
            "struct",
            "enum",
            "union",
            "error",
            "pub",
            "extern",
            "packed",
            "opaque",
            "threadlocal",
            "comptime",
        ],
        string_delimiters: vec![('"', '"')],
        comment_styles: vec![CommentStyle {
            line: Some("//".into()),
            block_start: None,
            block_end: None,
        }],
        brackets: vec![('(', ')'), ('{', '}'), ('[', ']')],
        indentation: IndentationStyle::Spaces(4),
    }
}

fn haskell_lang() -> LanguageDefinition {
    LanguageDefinition {
        name: "haskell".into(),
        extensions: svec!["hs"],
        keywords: svec![
            "case",
            "class",
            "data",
            "default",
            "deriving",
            "do",
            "else",
            "if",
            "import",
            "in",
            "infixl",
            "infixr",
            "instance",
            "let",
            "module",
            "newtype",
            "of",
            "then",
            "type",
            "where",
            "forall",
            "qualified",
        ],
        type_keywords: svec![
            "data", "type", "newtype", "class", "instance", "module", "import",
        ],
        string_delimiters: vec![('"', '"')],
        comment_styles: vec![CommentStyle {
            line: Some("--".into()),
            block_start: Some("{-".into()),
            block_end: Some("-}".into()),
        }],
        brackets: vec![('(', ')'), ('{', '}'), ('[', ']')],
        indentation: IndentationStyle::Spaces(2),
    }
}

fn scala_lang() -> LanguageDefinition {
    LanguageDefinition {
        name: "scala".into(),
        extensions: svec!["scala", "sc"],
        keywords: svec![
            "abstract",
            "case",
            "catch",
            "class",
            "def",
            "do",
            "else",
            "extends",
            "false",
            "final",
            "finally",
            "for",
            "forSome",
            "if",
            "implicit",
            "import",
            "lazy",
            "match",
            "new",
            "null",
            "object",
            "override",
            "package",
            "private",
            "protected",
            "return",
            "sealed",
            "super",
            "this",
            "throw",
            "trait",
            "true",
            "try",
            "type",
            "val",
            "var",
            "while",
            "with",
            "yield",
            "given",
            "using",
            "enum",
            "then",
            "end",
            "export",
        ],
        type_keywords: svec![
            "class",
            "object",
            "trait",
            "enum",
            "sealed",
            "type",
            "given",
            "extension",
            "using",
        ],
        string_delimiters: vec![('"', '"'), ('\'', '\''), ('"', '"')],
        comment_styles: vec![CommentStyle {
            line: Some("//".into()),
            block_start: Some("/*".into()),
            block_end: Some("*/".into()),
        }],
        brackets: vec![('(', ')'), ('{', '}'), ('[', ']'), ('<', '>')],
        indentation: IndentationStyle::Spaces(2),
    }
}

fn shell_lang() -> LanguageDefinition {
    LanguageDefinition {
        name: "shell".into(),
        extensions: svec!["sh", "bash", "zsh"],
        keywords: svec![
            "if", "then", "else", "elif", "fi", "for", "while", "until", "do", "done", "case",
            "esac", "in", "function", "select", "time", "coproc", "|", "&&", "||", "return",
            "exit", "break", "continue", "declare", "export", "local", "readonly", "typeset",
            "unset", "source", "alias", "echo", "eval", "exec", "set", "shift", "trap", "true",
            "false",
        ],
        type_keywords: svec!["function"],
        string_delimiters: vec![('"', '"'), ('\'', '\'')],
        comment_styles: vec![CommentStyle {
            line: Some("#".into()),
            block_start: None,
            block_end: None,
        }],
        brackets: vec![('(', ')'), ('{', '}'), ('[', ']'), ('`', '`')],
        indentation: IndentationStyle::Spaces(4),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_has_builtin_languages() {
        let registry = LanguageRegistry::new();
        let langs = registry.supported_languages();
        assert!(langs.len() >= 15);
        assert!(langs.contains(&"rust"));
        assert!(langs.contains(&"go"));
        assert!(langs.contains(&"python"));
        assert!(langs.contains(&"typescript"));
        assert!(langs.contains(&"javascript"));
        assert!(langs.contains(&"java"));
        assert!(langs.contains(&"kotlin"));
        assert!(langs.contains(&"swift"));
        assert!(langs.contains(&"c"));
        assert!(langs.contains(&"cpp"));
        assert!(langs.contains(&"ruby"));
        assert!(langs.contains(&"php"));
        assert!(langs.contains(&"zig"));
        assert!(langs.contains(&"haskell"));
        assert!(langs.contains(&"scala"));
        assert!(langs.contains(&"shell"));
    }

    #[test]
    fn test_get_language() {
        let registry = LanguageRegistry::new();
        let rust = registry.get("rust").unwrap();
        assert_eq!(rust.name, "rust");
        assert!(rust.keywords.contains(&"fn".to_string()));
        assert!(rust.extensions.contains(&"rs".to_string()));
    }

    #[test]
    fn test_get_nonexistent() {
        let registry = LanguageRegistry::new();
        assert!(registry.get("brainfuck").is_none());
    }

    #[test]
    fn test_detect_by_extension() {
        let registry = LanguageRegistry::new();
        assert_eq!(registry.detect_by_extension("rs").unwrap().name, "rust");
        assert_eq!(registry.detect_by_extension("go").unwrap().name, "go");
        assert_eq!(registry.detect_by_extension("py").unwrap().name, "python");
        assert_eq!(
            registry.detect_by_extension("ts").unwrap().name,
            "typescript"
        );
        assert_eq!(
            registry.detect_by_extension("js").unwrap().name,
            "javascript"
        );
        assert_eq!(registry.detect_by_extension("java").unwrap().name, "java");
        assert_eq!(registry.detect_by_extension("kt").unwrap().name, "kotlin");
        assert_eq!(registry.detect_by_extension("swift").unwrap().name, "swift");
        assert_eq!(registry.detect_by_extension("c").unwrap().name, "c");
        assert_eq!(registry.detect_by_extension("cpp").unwrap().name, "cpp");
        assert_eq!(registry.detect_by_extension("rb").unwrap().name, "ruby");
        assert_eq!(registry.detect_by_extension("php").unwrap().name, "php");
        assert_eq!(registry.detect_by_extension("zig").unwrap().name, "zig");
        assert_eq!(registry.detect_by_extension("hs").unwrap().name, "haskell");
        assert_eq!(registry.detect_by_extension("scala").unwrap().name, "scala");
        assert_eq!(registry.detect_by_extension("sh").unwrap().name, "shell");
    }

    #[test]
    fn test_detect_by_extension_leading_dot() {
        let registry = LanguageRegistry::new();
        assert_eq!(registry.detect_by_extension(".rs").unwrap().name, "rust");
    }

    #[test]
    fn test_detect_unknown_extension() {
        let registry = LanguageRegistry::new();
        assert!(registry.detect_by_extension("xyz").is_none());
    }

    #[test]
    fn test_rust_definition() {
        let registry = LanguageRegistry::new();
        let rust = registry.get("rust").unwrap();
        assert!(rust.keywords.contains(&"fn".to_string()));
        assert!(rust.keywords.contains(&"let".to_string()));
        assert!(rust.keywords.contains(&"mut".to_string()));
        assert!(rust.type_keywords.contains(&"struct".to_string()));
        assert!(rust.type_keywords.contains(&"enum".to_string()));
        assert!(rust.type_keywords.contains(&"trait".to_string()));
        assert_eq!(rust.indentation, IndentationStyle::Spaces(4));
        assert_eq!(rust.string_delimiters.len(), 1);
    }

    #[test]
    fn test_go_definition() {
        let registry = LanguageRegistry::new();
        let go = registry.get("go").unwrap();
        assert!(go.keywords.contains(&"func".to_string()));
        assert!(go.keywords.contains(&"go".to_string()));
        assert!(go.keywords.contains(&"chan".to_string()));
        assert_eq!(go.indentation, IndentationStyle::Tabs);
        assert!(go.comment_styles[0].line.as_deref() == Some("//"));
    }

    #[test]
    fn test_python_definition() {
        let registry = LanguageRegistry::new();
        let py = registry.get("python").unwrap();
        assert!(py.keywords.contains(&"def".to_string()));
        assert!(py.keywords.contains(&"class".to_string()));
        assert!(py.keywords.contains(&"async".to_string()));
        assert_eq!(py.indentation, IndentationStyle::Spaces(4));
        assert!(py.extensions.contains(&"py".to_string()));
        assert!(py.extensions.contains(&"pyi".to_string()));
    }

    #[test]
    fn test_cpp_multiple_extensions() {
        let registry = LanguageRegistry::new();
        let cpp = registry.get("cpp").unwrap();
        assert!(cpp.extensions.contains(&"cpp".to_string()));
        assert!(cpp.extensions.contains(&"cc".to_string()));
        assert!(cpp.extensions.contains(&"cxx".to_string()));
        assert!(cpp.extensions.contains(&"hpp".to_string()));
    }

    #[test]
    fn test_register_custom_language() {
        let mut registry = LanguageRegistry::new();
        let custom = LanguageDefinition {
            name: "custom".into(),
            extensions: vec!["custom".into()],
            keywords: vec!["foo".into(), "bar".into()],
            type_keywords: vec![],
            string_delimiters: vec![],
            comment_styles: vec![],
            brackets: vec![],
            indentation: IndentationStyle::Spaces(2),
        };
        registry.register(custom);
        assert!(registry.get("custom").is_some());
        assert_eq!(
            registry.detect_by_extension("custom").unwrap().name,
            "custom"
        );
    }

    #[test]
    fn test_supported_languages_sorted() {
        let registry = LanguageRegistry::new();
        let langs = registry.supported_languages();
        let mut sorted = langs.clone();
        sorted.sort();
        assert_eq!(langs, sorted);
    }
}
