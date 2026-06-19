use crate::components::BadgeColor;

#[cfg(feature = "csr")]
use wasm_bindgen::JsCast;

pub fn relative_time(timestamp: &str) -> String {
    #[cfg(feature = "csr")]
    {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
            let now = chrono::Utc::now();
            let diff = now.signed_duration_since(dt);
            if diff.num_seconds() < 60 {
                return "just now".to_string();
            } else if diff.num_minutes() < 60 {
                return format!("{}m ago", diff.num_minutes());
            } else if diff.num_hours() < 24 {
                return format!("{}h ago", diff.num_hours());
            } else if diff.num_days() < 30 {
                return format!("{}d ago", diff.num_days());
            } else {
                return dt.format("%b %d, %Y").to_string();
            }
        }
    }
    timestamp.to_string()
}

pub fn truncate_uuid(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len])
    } else {
        s.to_string()
    }
}

pub fn get_input_value(name: &str) -> String {
    #[cfg(feature = "csr")]
    {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return String::new(),
        };
        let doc = match window.document() {
            Some(d) => d,
            None => return String::new(),
        };
        let el = match doc.get_element_by_id(name) {
            Some(el) => el,
            None => return String::new(),
        };
        let tag = el.tag_name().to_lowercase();
        if tag == "textarea" {
            match el.dyn_into::<web_sys::HtmlTextAreaElement>() {
                Ok(ta) => return ta.value(),
                Err(_) => return String::new(),
            }
        }
        match el.dyn_into::<web_sys::HtmlInputElement>() {
            Ok(input) => input.value(),
            Err(_) => String::new(),
        }
    }
    #[cfg(not(feature = "csr"))]
    {
        let _ = name;
        String::new()
    }
}

pub fn status_badge_color(state: &str) -> BadgeColor {
    match state {
        "open" => BadgeColor::Success,
        "in_progress" => BadgeColor::Info,
        "closed" => BadgeColor::Neutral,
        _ => BadgeColor::Neutral,
    }
}

pub fn status_label(state: &str) -> String {
    match state {
        "in_progress" => "In Progress".to_string(),
        s => {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        }
    }
}

pub fn sanitize_error(raw: &str) -> String {
    raw.chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '.' || *c == '-')
        .collect::<String>()
        .trim()
        .to_string()
}

/// Format byte count into human-readable string (KB, MB, GB, TB).
pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Truncate a string to max_len characters, appending "..." if truncated.
pub fn truncate_title(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len])
    } else {
        s.to_string()
    }
}

/// Return a CSS hex color for a programming language name.
pub fn language_color(name: &str) -> &'static str {
    match name {
        "rust" => "#dea584",
        "python" => "#3572A5",
        "javascript" => "#f1e05a",
        "typescript" => "#3178c6",
        "go" => "#00ADD8",
        "java" => "#b07219",
        "c" => "#555555",
        "cpp" => "#f34b7d",
        "csharp" => "#178600",
        "ruby" => "#701516",
        "php" => "#4F5D95",
        "swift" => "#F05138",
        "kotlin" => "#A97BFF",
        "scala" => "#c22d40",
        "html" => "#e34c26",
        "css" => "#563d7c",
        "shell" => "#89e051",
        "bash" => "#89e051",
        "sql" => "#e38c00",
        "dockerfile" => "#384d54",
        "markdown" => "#083fa1",
        _ => "#6e7681",
    }
}

pub fn detect_language(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "rs" => "rust",
        "py" => "python",
        "js" | "mjs" | "cjs" => "javascript",
        "ts" | "mts" | "cts" => "typescript",
        "jsx" => "jsx",
        "tsx" => "tsx",
        "go" => "go",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "hh" => "cpp",
        "cs" => "csharp",
        "rb" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "sql" => "sql",
        "html" | "htm" | "xhtml" => "html",
        "css" => "css",
        "scss" | "sass" | "less" => "scss",
        "xml" => "xml",
        "json" | "geojson" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "ini" => "ini",
        "cfg" => "ini",
        "sh" | "bash" | "zsh" | "fish" => "bash",
        "ps1" => "powershell",
        "bat" | "cmd" => "dos",
        "md" | "mdx" | "markdown" => "markdown",
        "tex" | "latex" => "latex",
        "r" | "rmd" => "r",
        "scala" | "sc" | "sbt" => "scala",
        "ex" | "exs" => "elixir",
        "erl" | "hrl" => "erlang",
        "hs" => "haskell",
        "dart" => "dart",
        "lua" => "lua",
        "vim" => "vim",
        "dockerfile" => "dockerfile",
        "makefile" => "makefile",
        "cmake" => "cmake",
        "proto" => "protobuf",
        "graphql" | "gql" => "graphql",
        "tf" | "tfvars" => "terraform",
        "zig" => "zig",
        "nim" => "nim",
        "v" => "v",
        "clj" | "cljs" | "cljc" => "clojure",
        "ml" | "mli" => "ocaml",
        "fs" | "fsx" | "fsi" => "fsharp",
        "pas" | "pp" | "inc" => "pascal",
        "adb" | "ads" => "ada",
        "groovy" | "gradle" => "groovy",
        "vue" => "vue",
        "svelte" => "svelte",
        _ => "",
    }
}

#[cfg(feature = "csr")]
pub fn inject_highlight_js() {
    let doc = web_sys::window().and_then(|w| w.document());
    if let Some(doc) = doc {
        let already_loaded = doc.get_element_by_id("hljs-css");
        if already_loaded.is_some() {
            return;
        }
        let _ = js_sys::eval(
            r#"
            (function() {
                if (window.hljs) return;
                var link = document.createElement('link');
                link.rel = 'stylesheet';
                link.href = 'https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github.min.css';
                link.id = 'hljs-css';
                document.head.appendChild(link);
                var script = document.createElement('script');
                script.src = 'https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js';
                script.id = 'hljs-js';
                document.head.appendChild(script);
            })();
            "#,
        );
    }
}

#[cfg(feature = "csr")]
pub fn highlight_code_block(selector: &str) {
    let script = format!(
        r#"
        (function() {{
            if (!window.hljs) return;
            document.querySelectorAll('{selector}').forEach(function(el) {{
                if (!el.dataset.highlighted) {{
                    hljs.highlightElement(el);
                }}
            }});
        }})();
        "#
    );
    let _ = js_sys::eval(&script);
}

#[cfg(feature = "csr")]
pub fn inject_marked_js() {
    let doc = web_sys::window().and_then(|w| w.document());
    if let Some(doc) = doc {
        let already_loaded = doc.get_element_by_id("marked-js");
        if already_loaded.is_some() {
            return;
        }
        let _ = js_sys::eval(
            r#"
            (function() {
                if (window.marked) return;
                var script = document.createElement('script');
                script.src = 'https://cdnjs.cloudflare.com/ajax/libs/marked/12.0.1/marked.min.js';
                script.id = 'marked-js';
                script.onload = function() {
                    if (window.marked) {
                        marked.setOptions({
                            breaks: true,
                            gfm: true,
                            headerIds: false,
                            mangle: false
                        });
                    }
                };
                document.head.appendChild(script);
            })();
            "#,
        );
    }
}

#[cfg(feature = "csr")]
pub fn inject_katex_js() {
    let doc = web_sys::window().and_then(|w| w.document());
    if let Some(doc) = doc {
        if doc.get_element_by_id("katex-css").is_some() {
            return;
        }
        let _ = js_sys::eval(
            r#"
            (function() {
                if (window.katex) return;
                var link = document.createElement('link');
                link.rel = 'stylesheet';
                link.href = 'https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css';
                link.id = 'katex-css';
                document.head.appendChild(link);
                var script = document.createElement('script');
                script.src = 'https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js';
                script.id = 'katex-js';
                document.head.appendChild(script);
            })();
            "#,
        );
    }
}

#[cfg(feature = "csr")]
pub fn inject_mermaid_js() {
    let doc = web_sys::window().and_then(|w| w.document());
    if let Some(doc) = doc {
        if doc.get_element_by_id("mermaid-js").is_some() {
            return;
        }
        let _ = js_sys::eval(
            r#"
            (function() {
                if (window.mermaid) return;
                var script = document.createElement('script');
                script.src = 'https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js';
                script.id = 'mermaid-js';
                script.onload = function() {
                    if (window.mermaid) {
                        window.mermaid.initialize({ startOnLoad: false, theme: 'default' });
                    }
                };
                document.head.appendChild(script);
            })();
            "#,
        );
    }
}

#[cfg(feature = "csr")]
pub fn render_markdown(markdown: &str) -> String {
    let escaped = markdown
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    let script = format!(
        r#"
        (function() {{
            if (!window.marked) return '';
            try {{
                var html = window.marked.parse(`{escaped}`);
                if (window.katex) {{
                    html = html.replace(/\$\$([\s\S]+?)\$\$/g, function(match, tex) {{
                        try {{
                            return window.katex.renderToString(tex.trim(), {{ displayMode: true, throwOnError: false }});
                        }} catch(e) {{ return match; }}
                    }});
                    html = html.replace(/\$([^\$\n]+?)\$/g, function(match, tex) {{
                        try {{
                            return window.katex.renderToString(tex.trim(), {{ displayMode: false, throwOnError: false }});
                        }} catch(e) {{ return match; }}
                    }});
                }}
                html = html.replace(/<pre><code class="language-mermaid">([\s\S]*?)<\/code><\/pre>/g, function(match, code) {{
                    var decoded = code.replace(/&amp;/g, '&').replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"').replace(/&#39;/g, "'");
                    return '<div class="mermaid">' + decoded + '</div>';
                }});
                if (window.mermaid && document.querySelectorAll('.mermaid').length > 0) {{
                    try {{ window.mermaid.run(); }} catch(e) {{}}
                }}
                return html;
            }} catch(e) {{
                return '';
            }}
        }})();
        "#
    );
    js_sys::eval(&script)
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}
