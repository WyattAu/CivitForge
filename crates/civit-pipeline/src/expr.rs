//! Pipeline expression evaluator for `if:` and `when:` conditions.
//!
//! Supports a subset of CEL commonly used in CI/CD YAML:
//! - String equality: `git.ref_name == "refs/heads/main"`
//! - String inequality: `git.ref_name != "refs/heads/main"`
//! - Contains: `git.ref_name contains "main"`
//! - Starts with: `git.ref_name startsWith "refs/"`
//! - Boolean operators: `&&`, `||`, `!`
//! - Variable expansion: `${{ var }}` resolved from TriggerContext

use std::collections::HashMap;

/// A pipeline expression that can be evaluated against a context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineExpression {
    raw: String,
}

impl PipelineExpression {
    pub fn new(raw: impl Into<String>) -> Self {
        Self { raw: raw.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Evaluate the expression against the given variable context.
    /// Returns true if the expression evaluates to a truthy value.
    pub fn evaluate(&self, context: &HashMap<String, String>) -> bool {
        let expr = self.raw.trim();
        if expr.is_empty() {
            return true;
        }

        let expanded = expand_variables(expr, context);

        // Handle OR (||) — short-circuit: any true → true
        if let Some(idx) = find_operator(&expanded, "||") {
            let left = expanded[..idx].trim();
            let right = expanded[idx + 2..].trim();
            return evaluate_atom(left, context) || evaluate_atom(right, context);
        }

        // Handle AND (&&) — short-circuit: any false → false
        if let Some(idx) = find_operator(&expanded, "&&") {
            let left = expanded[..idx].trim();
            let right = expanded[idx + 2..].trim();
            return evaluate_atom(left, context) && evaluate_atom(right, context);
        }

        evaluate_atom(&expanded, context)
    }

    /// Legacy optimistic check — returns true if non-empty.
    /// Use `evaluate()` instead for actual condition checking.
    pub fn is_truthy(&self) -> bool {
        !self.raw.trim().is_empty()
    }
}

/// Find the first occurrence of a binary operator that's not inside quotes.
fn find_operator(expr: &str, op: &str) -> Option<usize> {
    let mut in_quotes = false;
    let mut i = 0;
    let bytes = expr.as_bytes();

    while i + op.len() <= bytes.len() {
        let ch = bytes[i];

        if ch == b'"' {
            in_quotes = !in_quotes;
        }

        if !in_quotes && expr[i..].starts_with(op) {
            return Some(i);
        }

        i += 1;
    }

    None
}

/// Evaluate a single atomic expression (no && or ||).
fn evaluate_atom(expr: &str, context: &HashMap<String, String>) -> bool {
    let expr = expr.trim();

    // Handle negation: !expr
    if let Some(inner) = expr.strip_prefix('!') {
        return !evaluate_atom(inner.trim(), context);
    }

    // Handle parenthesized expressions
    if expr.starts_with('(') && expr.ends_with(')') {
        let inner = &expr[1..expr.len() - 1];
        return evaluate_expression_inner(inner, context);
    }

    // Handle == operator
    if let Some(idx) = find_operator(expr, "==") {
        let left = resolve_variable(expr[..idx].trim(), context);
        let right = resolve_string_value(expr[idx + 2..].trim());
        return left == right;
    }

    // Handle != operator
    if let Some(idx) = find_operator(expr, "!=") {
        let left = resolve_variable(expr[..idx].trim(), context);
        let right = resolve_string_value(expr[idx + 2..].trim());
        return left != right;
    }

    // Handle "contains" operator: var contains "value"
    if let Some(idx) = expr.find(" contains ") {
        let left = resolve_variable(expr[..idx].trim(), context);
        let right = resolve_string_value(expr[idx + 10..].trim());
        return left.contains(&right);
    }

    // Handle "startsWith" operator: var startsWith "value"
    if let Some(idx) = expr.find(" startsWith ") {
        let left = resolve_variable(expr[..idx].trim(), context);
        let right = resolve_string_value(expr[idx + 12..].trim());
        return left.starts_with(&right);
    }

    // Handle "endsWith" operator: var endsWith "value"
    if let Some(idx) = expr.find(" endsWith ") {
        let left = resolve_variable(expr[..idx].trim(), context);
        let right = resolve_string_value(expr[idx + 10..].trim());
        return left.ends_with(&right);
    }

    // Handle "matches" operator: var matches "pattern" (simplified glob)
    if let Some(idx) = expr.find(" matches ") {
        let left = resolve_variable(expr[..idx].trim(), context);
        let right = resolve_string_value(expr[idx + 9..].trim());
        return simple_glob_match(&right, &left);
    }

    // Handle bare boolean variable: "true" or "false"
    if expr == "true" {
        return true;
    }
    if expr == "false" {
        return false;
    }

    // Handle variable reference: resolve and check if non-empty
    let resolved = resolve_variable(expr, context);
    !resolved.is_empty()
}

/// Evaluate an expression string (used for parenthesized sub-expressions).
fn evaluate_expression_inner(expr: &str, context: &HashMap<String, String>) -> bool {
    let expr = expr.trim();
    if expr.is_empty() {
        return true;
    }

    // Handle OR (||)
    if let Some(idx) = find_operator(expr, "||") {
        let left = expr[..idx].trim();
        let right = expr[idx + 2..].trim();
        return evaluate_atom(left, context) || evaluate_atom(right, context);
    }

    // Handle AND (&&)
    if let Some(idx) = find_operator(expr, "&&") {
        let left = expr[..idx].trim();
        let right = expr[idx + 2..].trim();
        return evaluate_atom(left, context) && evaluate_atom(right, context);
    }

    evaluate_atom(expr, context)
}

/// Resolve a variable name to its value from context.
/// Supports both `variable.name` and `"literal"` (quoted).
fn resolve_variable(token: &str, context: &HashMap<String, String>) -> String {
    let token = token.trim();

    // Quoted string literal
    if (token.starts_with('"') && token.ends_with('"'))
        || (token.starts_with('\'') && token.ends_with('\''))
    {
        return token[1..token.len() - 1].to_string();
    }

    // Variable lookup
    context.get(token).cloned().unwrap_or_default()
}

/// Extract a string value from a token (removing quotes if present).
fn resolve_string_value(token: &str) -> String {
    resolve_variable(token.trim(), &HashMap::new())
}

/// Expand `${{ var }}` expressions in the input string.
/// Handles both `${{var}}` and `${{ var }}` (with spaces).
/// Expanded values are wrapped in quotes to prevent further variable lookup.
fn expand_variables(expr: &str, context: &HashMap<String, String>) -> String {
    let mut result = expr.to_string();
    for (key, value) in context {
        // Try both with and without spaces inside ${{ }}
        let patterns = [
            ["${{", " ", key, " ", "}}"].concat(),
            ["${{", key, "}}"].concat(),
        ];
        for pattern in &patterns {
            result = result.replace(pattern, &format!("\"{value}\""));
        }
    }
    result
}

/// Simple glob matching: `*` matches any substring, `?` matches single char.
fn simple_glob_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if pattern.is_empty() {
        return text.is_empty();
    }
    // Convert simple glob to case-insensitive substring match
    let clean_pattern = pattern.trim_matches('*').trim_matches('?');
    text.to_lowercase().contains(&clean_pattern.to_lowercase())
}

impl std::fmt::Display for PipelineExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl From<String> for PipelineExpression {
    fn from(raw: String) -> Self {
        Self { raw }
    }
}

impl From<&str> for PipelineExpression {
    fn from(raw: &str) -> Self {
        Self {
            raw: raw.to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_expression() {
        let expr = PipelineExpression::new("");
        assert!(expr.evaluate(&HashMap::new()));
    }

    #[test]
    fn test_equality_true() {
        let mut ctx = HashMap::new();
        ctx.insert("git.ref_name".to_string(), "refs/heads/main".to_string());
        let expr = PipelineExpression::new("git.ref_name == \"refs/heads/main\"");
        assert!(expr.evaluate(&ctx));
    }

    #[test]
    fn test_equality_false() {
        let mut ctx = HashMap::new();
        ctx.insert("git.ref_name".to_string(), "refs/heads/develop".to_string());
        let expr = PipelineExpression::new("git.ref_name == \"refs/heads/main\"");
        assert!(!expr.evaluate(&ctx));
    }

    #[test]
    fn test_inequality() {
        let mut ctx = HashMap::new();
        ctx.insert("git.ref_name".to_string(), "refs/heads/develop".to_string());
        let expr = PipelineExpression::new("git.ref_name != \"refs/heads/main\"");
        assert!(expr.evaluate(&ctx));
    }

    #[test]
    fn test_contains() {
        let mut ctx = HashMap::new();
        ctx.insert("git.ref_name".to_string(), "refs/heads/main".to_string());
        let expr = PipelineExpression::new("git.ref_name contains \"main\"");
        assert!(expr.evaluate(&ctx));
    }

    #[test]
    fn test_starts_with() {
        let mut ctx = HashMap::new();
        ctx.insert("git.ref_name".to_string(), "refs/heads/main".to_string());
        let expr = PipelineExpression::new("git.ref_name startsWith \"refs/\"");
        assert!(expr.evaluate(&ctx));
    }

    #[test]
    fn test_ends_with() {
        let mut ctx = HashMap::new();
        ctx.insert("git.ref_name".to_string(), "refs/heads/main".to_string());
        let expr = PipelineExpression::new("git.ref_name endsWith \"main\"");
        assert!(expr.evaluate(&ctx));
    }

    #[test]
    fn test_and_operator() {
        let mut ctx = HashMap::new();
        ctx.insert("git.ref_name".to_string(), "refs/heads/main".to_string());
        ctx.insert("pipeline.name".to_string(), "build".to_string());
        let expr = PipelineExpression::new(
            "git.ref_name == \"refs/heads/main\" && pipeline.name == \"build\"",
        );
        assert!(expr.evaluate(&ctx));
    }

    #[test]
    fn test_and_operator_false() {
        let mut ctx = HashMap::new();
        ctx.insert("git.ref_name".to_string(), "refs/heads/main".to_string());
        ctx.insert("pipeline.name".to_string(), "test".to_string());
        let expr = PipelineExpression::new(
            "git.ref_name == \"refs/heads/main\" && pipeline.name == \"build\"",
        );
        assert!(!expr.evaluate(&ctx));
    }

    #[test]
    fn test_or_operator() {
        let mut ctx = HashMap::new();
        ctx.insert("git.ref_name".to_string(), "refs/heads/develop".to_string());
        ctx.insert("pipeline.name".to_string(), "build".to_string());
        let expr = PipelineExpression::new(
            "git.ref_name == \"refs/heads/main\" || pipeline.name == \"build\"",
        );
        assert!(expr.evaluate(&ctx));
    }

    #[test]
    fn test_or_operator_both_false() {
        let mut ctx = HashMap::new();
        ctx.insert("git.ref_name".to_string(), "refs/heads/develop".to_string());
        ctx.insert("pipeline.name".to_string(), "test".to_string());
        let expr = PipelineExpression::new(
            "git.ref_name == \"refs/heads/main\" || pipeline.name == \"build\"",
        );
        assert!(!expr.evaluate(&ctx));
    }

    #[test]
    fn test_negation() {
        let mut ctx = HashMap::new();
        ctx.insert("git.ref_name".to_string(), "refs/heads/develop".to_string());
        let expr = PipelineExpression::new("!git.ref_name == \"refs/heads/main\"");
        assert!(expr.evaluate(&ctx));
    }

    #[test]
    fn test_negation_false() {
        let mut ctx = HashMap::new();
        ctx.insert("git.ref_name".to_string(), "refs/heads/main".to_string());
        let expr = PipelineExpression::new("!git.ref_name == \"refs/heads/main\"");
        assert!(!expr.evaluate(&ctx));
    }

    #[test]
    fn test_matches_glob() {
        let mut ctx = HashMap::new();
        ctx.insert("git.ref_name".to_string(), "refs/heads/main".to_string());
        let expr = PipelineExpression::new("git.ref_name matches \"*main*\"");
        assert!(expr.evaluate(&ctx));
    }

    #[test]
    fn test_variable_expansion() {
        let mut ctx = HashMap::new();
        ctx.insert("branch".to_string(), "main".to_string());
        let expr = PipelineExpression::new("${{ branch }} == \"main\"");
        assert!(expr.evaluate(&ctx));
    }

    #[test]
    fn test_missing_variable() {
        let expr = PipelineExpression::new("missing.var == \"value\"");
        assert!(!expr.evaluate(&HashMap::new()));
    }

    #[test]
    fn test_display() {
        let expr = PipelineExpression::new("git.ref_name == \"main\"");
        assert_eq!(format!("{expr}"), "git.ref_name == \"main\"");
    }

    #[test]
    fn test_from_string() {
        let expr: PipelineExpression = "test".into();
        assert_eq!(expr.as_str(), "test");
    }

    #[test]
    fn test_from_str_ref() {
        let expr: PipelineExpression = String::from("hello").as_str().into();
        assert_eq!(expr.as_str(), "hello");
    }

    #[test]
    fn test_is_truthy_legacy() {
        let expr = PipelineExpression::new("some expression");
        assert!(expr.is_truthy());
        let empty = PipelineExpression::new("");
        assert!(!empty.is_truthy());
    }
}
