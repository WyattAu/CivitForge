#![forbid(unsafe_code)]

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A CEL (Common Expression Language) expression for policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CelExpression {
    pub raw: String,
    pub kind: CelKind,
}

/// Supported CEL expression categories.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CelKind {
    /// Simple boolean comparison: `request.time < "2025-01-01"`
    Comparison,
    /// Logical operator: `user.role == "admin" && source.ip.startsWith("10.")
    Logical,
    /// Membership test: `"read" in user.permissions`
    Membership,
    /// Function call: `size(user.groups) > 0`
    FunctionCall,
    /// Ternary: `user.verified ? "allow" : "deny"`
    Ternary,
    /// Negation: `!user.suspended`
    Negation,
    /// Type check: `user.age > 18`
    TypeCheck,
    /// Arithmetic: `user.age + 1`, `price * 0.9`
    Arithmetic,
    /// Complex expression with multiple operators
    Complex,
    /// Unknown or unsupported expression
    Unknown,
}

impl CelExpression {
    pub fn parse(raw: impl Into<String>) -> Self {
        let raw_str = raw.into();
        let kind = Self::classify(&raw_str);
        Self { raw: raw_str, kind }
    }

    fn classify(expr: &str) -> CelKind {
        let trimmed = expr.trim();
        // Ternary (check before logical because && is present in ternary often)
        if trimmed.contains("?") && trimmed.contains(":") {
            return CelKind::Ternary;
        }
        // Function call
        if Self::has_function_call(trimmed) {
            return CelKind::FunctionCall;
        }
        // Logical operators
        if trimmed.contains("&&") || trimmed.contains("||") {
            return CelKind::Logical;
        }
        // Membership
        if trimmed.contains(" in ") || trimmed.contains(" not in ") {
            return CelKind::Membership;
        }
        // Negation
        if trimmed.starts_with('!') {
            return CelKind::Negation;
        }
        // Arithmetic operators (check before comparison to avoid `+` inside string literals)
        let arith_ops = ["+", "-", "*", "/", "%"];
        let has_arith = arith_ops.iter().any(|op| {
            let pos = trimmed.find(*op).unwrap_or(0);
            // Skip if it's inside a comparison operator like `>=` or `<=` or `!=`
            if *op == "-" {
                // Avoid matching `-` inside `!=` or at start of negative number after operator
                if pos > 0 {
                    let prev = trimmed.as_bytes()[pos - 1];
                    if prev == b'!' || prev == b'>' || prev == b'<' || prev == b'=' {
                        return false;
                    }
                }
            }
            pos > 0
                && !trimmed.contains("==")
                && !trimmed.contains("!=")
                && !trimmed.contains(">=")
                && !trimmed.contains("<=")
        });
        if has_arith {
            return CelKind::Arithmetic;
        }
        // Comparison operators
        if trimmed.contains("==")
            || trimmed.contains("!=")
            || trimmed.contains(">=")
            || trimmed.contains("<=")
            || trimmed.contains('>')
            || trimmed.contains('<')
        {
            return CelKind::Comparison;
        }
        CelKind::Unknown
    }

    fn has_function_call(expr: &str) -> bool {
        let functions = [
            "size(",
            "has(",
            "matches(",
            "startsWith(",
            "endsWith(",
            "contains(",
            "indexOf(",
            "lower(",
            "upper(",
            "string(",
            "int(",
            "double(",
            "bool(",
            "timestamp(",
            "duration(",
            "now(",
            "abs(",
            "ceil(",
            "floor(",
            "max(",
            "min(",
            "type(",
            "in(",
        ];
        for func in &functions {
            if expr.contains(func) {
                return true;
            }
        }
        false
    }
}

/// A variable binding for CEL evaluation context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelVariable {
    pub name: String,
    pub value: CelValue,
    pub var_type: CelType,
}

/// Supported CEL value types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CelValue {
    Null,
    Bool(bool),
    Int(i64),
    Uint(u64),
    Double(f64),
    String(String),
    Bytes(Vec<u8>),
    List(Vec<CelValue>),
    Map(HashMap<String, CelValue>),
    Timestamp(chrono::DateTime<chrono::Utc>),
    Duration(std::time::Duration),
}

impl PartialEq for CelValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::Uint(a), Self::Uint(b)) => a == b,
            (Self::Double(a), Self::Double(b)) => (a - b).abs() < f64::EPSILON,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Bytes(a), Self::Bytes(b)) => a == b,
            (Self::List(a), Self::List(b)) => a == b,
            (Self::Map(a), Self::Map(b)) => a == b,
            (Self::Timestamp(a), Self::Timestamp(b)) => a == b,
            (Self::Duration(a), Self::Duration(b)) => a == b,
            _ => false,
        }
    }
}

impl CelValue {
    pub fn type_name(&self) -> &str {
        match self {
            Self::Null => "null",
            Self::Bool(_) => "bool",
            Self::Int(_) => "int",
            Self::Uint(_) => "uint",
            Self::Double(_) => "double",
            Self::String(_) => "string",
            Self::Bytes(_) => "bytes",
            Self::List(_) => "list",
            Self::Map(_) => "map",
            Self::Timestamp(_) => "timestamp",
            Self::Duration(_) => "duration",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Null => false,
            Self::Bool(b) => *b,
            Self::Int(n) => *n != 0,
            Self::Uint(n) => *n != 0,
            Self::Double(f) => *f != 0.0,
            Self::String(s) => !s.is_empty(),
            Self::List(l) => !l.is_empty(),
            Self::Map(m) => !m.is_empty(),
            Self::Bytes(b) => !b.is_empty(),
            Self::Timestamp(_) => true,
            Self::Duration(_) => true,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&Vec<CelValue>> {
        match self {
            Self::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&HashMap<String, CelValue>> {
        match self {
            Self::Map(m) => Some(m),
            _ => None,
        }
    }
}

/// CEL type declarations for static analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CelType {
    Dyn,
    Bool,
    Int,
    Uint,
    Double,
    String,
    Bytes,
    List(Box<CelType>),
    Map(Box<CelType>, Box<CelType>),
    Timestamp,
    Duration,
    NullType,
}

/// Evaluation environment for CEL expressions.
#[derive(Debug, Clone, Default)]
pub struct CelEnvironment {
    variables: HashMap<String, CelVariable>,
    functions: HashMap<String, CelFunctionDef>,
}

impl CelEnvironment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_variable(
        mut self,
        name: impl Into<String>,
        value: CelValue,
        var_type: CelType,
    ) -> Self {
        self.variables.insert(
            name.into(),
            CelVariable {
                name: String::new(),
                value,
                var_type,
            },
        );
        self
    }

    pub fn with_function(mut self, def: CelFunctionDef) -> Self {
        self.functions.insert(def.name.clone(), def);
        self
    }

    pub fn get_variable(&self, name: &str) -> Option<&CelValue> {
        self.variables.get(name).map(|v| &v.value)
    }

    pub fn get_function(&self, name: &str) -> Option<&CelFunctionDef> {
        self.functions.get(name)
    }

    pub fn resolve_path(&self, path: &str) -> Option<CelValue> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return None;
        }
        let root = self.get_variable(parts[0])?;
        let mut current = root.clone();
        for part in &parts[1..] {
            match &mut current {
                CelValue::Map(map) => {
                    let key = part.to_string();
                    current = map.remove(&key).unwrap_or(CelValue::Null);
                }
                _ => return None,
            }
        }
        Some(current)
    }
}

/// A built-in or custom CEL function definition.
#[derive(Debug, Clone)]
pub struct CelFunctionDef {
    pub name: String,
    pub params: Vec<CelType>,
    pub return_type: CelType,
    pub is_variadic: bool,
}

impl CelFunctionDef {
    pub fn new(name: impl Into<String>, params: Vec<CelType>, return_type: CelType) -> Self {
        Self {
            name: name.into(),
            params,
            return_type,
            is_variadic: false,
        }
    }

    pub fn variadic(name: impl Into<String>, params: Vec<CelType>, return_type: CelType) -> Self {
        Self {
            name: name.into(),
            params,
            return_type,
            is_variadic: true,
        }
    }
}

/// Result of CEL expression evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelResult {
    pub value: CelValue,
    pub expression: String,
    pub success: bool,
    pub error: Option<String>,
}

impl CelResult {
    pub fn ok(value: CelValue, expression: impl Into<String>) -> Self {
        Self {
            value,
            expression: expression.into(),
            success: true,
            error: None,
        }
    }

    pub fn err(error: impl Into<String>, expression: impl Into<String>) -> Self {
        Self {
            value: CelValue::Null,
            expression: expression.into(),
            success: false,
            error: Some(error.into()),
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if self.success {
            self.value.as_bool()
        } else {
            None
        }
    }
}

/// CEL evaluator. Supports comparison, logical, arithmetic, membership,
/// ternary, negation, type checks, and 20+ built-in functions.
pub struct CelEvaluator {
    env: CelEnvironment,
}

impl CelEvaluator {
    pub fn new(env: CelEnvironment) -> Self {
        Self { env }
    }

    /// Evaluates a CEL expression against the current environment.
    /// The stub supports: equality, comparison, negation, membership, and logical operators.
    pub fn evaluate(&self, expr: &CelExpression) -> CelResult {
        let raw = expr.raw.trim();
        match &expr.kind {
            CelKind::Negation => self.eval_negation(raw),
            CelKind::Comparison => self.eval_comparison(raw),
            CelKind::Logical => self.eval_logical(raw),
            CelKind::Membership => self.eval_membership(raw),
            CelKind::FunctionCall => self.eval_function(raw),
            CelKind::Ternary => self.eval_ternary(raw),
            CelKind::TypeCheck => self.eval_comparison(raw),
            CelKind::Arithmetic => self.eval_arithmetic(raw),
            CelKind::Complex => self.eval_complex(raw),
            CelKind::Unknown => CelResult::err(format!("Unsupported expression: {raw}"), &expr.raw),
        }
    }

    fn resolve_value(&self, token: &str) -> CelValue {
        let trimmed = token.trim();
        // Handle parenthesized sub-expressions
        if trimmed.starts_with('(') && trimmed.ends_with(')') {
            let inner = &trimmed[1..trimmed.len() - 1];
            let sub_expr = CelExpression::parse(inner);
            let result = self.evaluate(&sub_expr);
            if result.success {
                return result.value;
            }
            return CelValue::Null;
        }
        // Try literal parsing first
        if trimmed == "true" {
            return CelValue::Bool(true);
        }
        if token == "false" {
            return CelValue::Bool(false);
        }
        if token == "null" {
            return CelValue::Null;
        }
        if let Ok(n) = token.parse::<i64>() {
            return CelValue::Int(n);
        }
        if let Ok(f) = token.parse::<f64>() {
            return CelValue::Double(f);
        }
        if token.starts_with('"') && token.ends_with('"') {
            return CelValue::String(token[1..token.len() - 1].to_string());
        }
        // Try path resolution
        self.env.resolve_path(token).unwrap_or(CelValue::Null)
    }

    fn eval_negation(&self, expr: &str) -> CelResult {
        let inner = expr.trim_start_matches('!').trim();
        let val = self.resolve_value(inner);
        if let CelValue::Bool(b) = val {
            CelResult::ok(CelValue::Bool(!b), expr)
        } else {
            CelResult::err(format!("Cannot negate non-boolean: {inner}"), expr)
        }
    }

    fn eval_comparison(&self, expr: &str) -> CelResult {
        // Find the first matching operator (longest first to avoid ">=" matching as ">")
        let candidates = [">=", "<=", "!=", "==", ">", "<"];
        let mut found_op: Option<&str> = None;
        let mut found_pos = 0;
        for op in &candidates {
            if let Some(pos) = expr.find(*op) {
                found_op = Some(op);
                found_pos = pos;
                break; // candidates are ordered longest-first
            }
        }
        let op = match found_op {
            Some(o) => o,
            None => return CelResult::err("No comparison operator found", expr),
        };
        let left_str = expr[..found_pos].trim();
        let right_str = expr[found_pos + op.len()..].trim();
        let left = self.resolve_value(left_str);
        let right = self.resolve_value(right_str);
        match op {
            "==" => CelResult::ok(CelValue::Bool(left == right), expr),
            "!=" => CelResult::ok(CelValue::Bool(left != right), expr),
            ">" | ">=" | "<" | "<=" => {
                let result = match (&left, &right) {
                    (CelValue::Int(a), CelValue::Int(b)) => match op {
                        ">" => a > b,
                        ">=" => a >= b,
                        "<" => a < b,
                        "<=" => a <= b,
                        _ => false,
                    },
                    (CelValue::Double(a), CelValue::Double(b)) => match op {
                        ">" => a > b,
                        ">=" => a >= b,
                        "<" => a < b,
                        "<=" => a <= b,
                        _ => false,
                    },
                    (CelValue::String(a), CelValue::String(b)) => match op {
                        ">" => a > b,
                        ">=" => a >= b,
                        "<" => a < b,
                        "<=" => a <= b,
                        _ => false,
                    },
                    _ => return CelResult::err("Type mismatch in comparison", expr),
                };
                CelResult::ok(CelValue::Bool(result), expr)
            }
            _ => CelResult::err("No comparison operator found", expr),
        }
    }

    fn eval_logical(&self, expr: &str) -> CelResult {
        if expr.contains("||") {
            return self.eval_or(expr, "||");
        }
        if expr.contains("&&") {
            return self.eval_and(expr, "&&");
        }
        CelResult::err("No logical operator found", expr)
    }

    fn eval_and(&self, expr: &str, op: &str) -> CelResult {
        let parts: Vec<&str> = expr.split(op).collect();
        if parts.len() < 2 {
            return CelResult::err("Invalid AND expression", expr);
        }
        let sub_expr = CelExpression::parse(parts[0].trim());
        if let CelResult {
            value: CelValue::Bool(false),
            ..
        } = self.evaluate(&sub_expr)
        {
            return CelResult::ok(CelValue::Bool(false), expr);
        }
        let sub_expr2 = CelExpression::parse(parts[1].trim());
        let r2 = self.evaluate(&sub_expr2);
        if r2.success {
            CelResult::ok(CelValue::Bool(r2.value.is_truthy()), expr)
        } else {
            r2
        }
    }

    fn eval_or(&self, expr: &str, op: &str) -> CelResult {
        let parts: Vec<&str> = expr.split(op).collect();
        if parts.len() < 2 {
            return CelResult::err("Invalid OR expression", expr);
        }
        let sub_expr = CelExpression::parse(parts[0].trim());
        if let CelResult {
            value: CelValue::Bool(true),
            ..
        } = self.evaluate(&sub_expr)
        {
            return CelResult::ok(CelValue::Bool(true), expr);
        }
        let sub_expr2 = CelExpression::parse(parts[1].trim());
        let r2 = self.evaluate(&sub_expr2);
        if r2.success {
            CelResult::ok(CelValue::Bool(r2.value.is_truthy()), expr)
        } else {
            r2
        }
    }

    fn eval_membership(&self, expr: &str) -> CelResult {
        let negated = expr.contains(" not in ");
        let separator = if negated { " not in " } else { " in " };
        if let Some(pos) = expr.find(separator) {
            let left_str = expr[..pos].trim();
            let right_str = expr[pos + separator.len()..].trim();
            let left = self.resolve_value(left_str);
            let right = self.resolve_value(right_str);
            let found = match &right {
                CelValue::List(items) => items.contains(&left),
                CelValue::String(s) => {
                    if let CelValue::String(l) = &left {
                        s.contains(l)
                    } else {
                        false
                    }
                }
                CelValue::Map(m) => {
                    if let CelValue::String(k) = &left {
                        m.contains_key(k)
                    } else {
                        false
                    }
                }
                _ => false,
            };
            let result = if negated { !found } else { found };
            return CelResult::ok(CelValue::Bool(result), expr);
        }
        CelResult::err("Invalid membership expression", expr)
    }

    fn eval_function(&self, expr: &str) -> CelResult {
        if expr.starts_with("has(") && expr.ends_with(')') {
            let path = expr[4..expr.len() - 1].trim();
            let val = self.env.resolve_path(path);
            let result = val.is_some() && val != Some(CelValue::Null);
            return CelResult::ok(CelValue::Bool(result), expr);
        }
        if expr.starts_with("size(") && expr.ends_with(')') {
            let path = expr[5..expr.len() - 1].trim();
            let val = self.env.resolve_path(path);
            let size = match val.as_ref().unwrap_or(&CelValue::Null) {
                CelValue::List(l) => l.len() as i64,
                CelValue::Map(m) => m.len() as i64,
                CelValue::String(s) => s.len() as i64,
                _ => 0,
            };
            return CelResult::ok(CelValue::Int(size), expr);
        }
        if expr.starts_with("type(") && expr.ends_with(')') {
            let path = expr[5..expr.len() - 1].trim();
            let val = self.env.resolve_path(path);
            let type_name = match val {
                Some(v) => v.type_name().to_string(),
                None => "null".to_string(),
            };
            return CelResult::ok(CelValue::String(type_name), expr);
        }
        if expr.starts_with("startsWith(") && expr.ends_with(')') {
            let args_str = expr[11..expr.len() - 1].trim();
            let args: Vec<&str> = split_args(args_str);
            if args.len() != 2 {
                return CelResult::err("startsWith() requires exactly 2 arguments", expr);
            }
            let val = self.resolve_value(args[0].trim());
            let prefix = self.resolve_value(args[1].trim());
            match (&val, &prefix) {
                (CelValue::String(s), CelValue::String(p)) => {
                    return CelResult::ok(CelValue::Bool(s.starts_with(p)), expr);
                }
                _ => return CelResult::err("startsWith() requires string arguments", expr),
            }
        } else if expr.starts_with("endsWith(") && expr.ends_with(')') {
            let args_str = expr[9..expr.len() - 1].trim();
            let args: Vec<&str> = split_args(args_str);
            if args.len() != 2 {
                return CelResult::err("endsWith() requires exactly 2 arguments", expr);
            }
            let val = self.resolve_value(args[0].trim());
            let suffix = self.resolve_value(args[1].trim());
            match (&val, &suffix) {
                (CelValue::String(s), CelValue::String(p)) => {
                    return CelResult::ok(CelValue::Bool(s.ends_with(p)), expr);
                }
                _ => return CelResult::err("endsWith() requires string arguments", expr),
            }
        } else if expr.starts_with("contains(") && expr.ends_with(')') {
            let args_str = expr[9..expr.len() - 1].trim();
            let args: Vec<&str> = split_args(args_str);
            if args.len() != 2 {
                return CelResult::err("contains() requires exactly 2 arguments", expr);
            }
            let val = self.resolve_value(args[0].trim());
            let substr = self.resolve_value(args[1].trim());
            match (&val, &substr) {
                (CelValue::String(s), CelValue::String(p)) => {
                    return CelResult::ok(CelValue::Bool(s.contains(p)), expr);
                }
                _ => return CelResult::err("contains() requires string arguments", expr),
            }
        } else if expr.starts_with("matches(") && expr.ends_with(')') {
            let args_str = expr[8..expr.len() - 1].trim();
            let args: Vec<&str> = split_args(args_str);
            if args.len() != 2 {
                return CelResult::err("matches() requires exactly 2 arguments", expr);
            }
            let val = self.resolve_value(args[0].trim());
            let pattern = self.resolve_value(args[1].trim());
            match (&val, &pattern) {
                (CelValue::String(s), CelValue::String(p)) => {
                    return match Regex::new(p) {
                        Ok(re) => CelResult::ok(CelValue::Bool(re.is_match(s)), expr),
                        Err(e) => CelResult::err(format!("Invalid regex in matches(): {e}"), expr),
                    };
                }
                _ => return CelResult::err("matches() requires string arguments", expr),
            }
        }
        if expr.starts_with("now()") {
            return CelResult::ok(CelValue::Timestamp(chrono::Utc::now()), expr);
        }
        // Math functions
        if expr.starts_with("abs(") && expr.ends_with(')') {
            let arg = expr[4..expr.len() - 1].trim();
            let val = self.resolve_value(arg);
            match val {
                CelValue::Int(n) => return CelResult::ok(CelValue::Int(n.abs()), expr),
                CelValue::Double(n) => return CelResult::ok(CelValue::Double(n.abs()), expr),
                _ => return CelResult::err("abs() requires numeric argument", expr),
            }
        }
        if expr.starts_with("ceil(") && expr.ends_with(')') {
            let arg = expr[5..expr.len() - 1].trim();
            let val = self.resolve_value(arg);
            match val {
                CelValue::Double(n) => return CelResult::ok(CelValue::Double(n.ceil()), expr),
                CelValue::Int(n) => {
                    return CelResult::ok(CelValue::Double((n as f64).ceil()), expr);
                }
                _ => return CelResult::err("ceil() requires numeric argument", expr),
            }
        }
        if expr.starts_with("floor(") && expr.ends_with(')') {
            let arg = expr[6..expr.len() - 1].trim();
            let val = self.resolve_value(arg);
            match val {
                CelValue::Double(n) => return CelResult::ok(CelValue::Double(n.floor()), expr),
                CelValue::Int(n) => {
                    return CelResult::ok(CelValue::Double((n as f64).floor()), expr);
                }
                _ => return CelResult::err("floor() requires numeric argument", expr),
            }
        }
        if expr.starts_with("max(") && expr.ends_with(')') {
            let args_str = expr[4..expr.len() - 1].trim();
            let args: Vec<&str> = split_args(args_str);
            if args.len() != 2 {
                return CelResult::err("max() requires exactly 2 arguments", expr);
            }
            let a = self.resolve_value(args[0].trim());
            let b = self.resolve_value(args[1].trim());
            match (&a, &b) {
                (CelValue::Int(x), CelValue::Int(y)) => {
                    return CelResult::ok(CelValue::Int((*x).max(*y)), expr);
                }
                (CelValue::Double(x), CelValue::Double(y)) => {
                    return CelResult::ok(CelValue::Double(x.max(*y)), expr);
                }
                _ => return CelResult::err("max() requires matching numeric arguments", expr),
            }
        }
        if expr.starts_with("min(") && expr.ends_with(')') {
            let args_str = expr[4..expr.len() - 1].trim();
            let args: Vec<&str> = split_args(args_str);
            if args.len() != 2 {
                return CelResult::err("min() requires exactly 2 arguments", expr);
            }
            let a = self.resolve_value(args[0].trim());
            let b = self.resolve_value(args[1].trim());
            match (&a, &b) {
                (CelValue::Int(x), CelValue::Int(y)) => {
                    return CelResult::ok(CelValue::Int((*x).min(*y)), expr);
                }
                (CelValue::Double(x), CelValue::Double(y)) => {
                    return CelResult::ok(CelValue::Double(x.min(*y)), expr);
                }
                _ => return CelResult::err("min() requires matching numeric arguments", expr),
            }
        }
        // String functions
        if expr.starts_with("indexOf(") && expr.ends_with(')') {
            let args_str = expr[8..expr.len() - 1].trim();
            let args: Vec<&str> = split_args(args_str);
            if args.len() != 2 {
                return CelResult::err("indexOf() requires exactly 2 arguments", expr);
            }
            let val = self.resolve_value(args[0].trim());
            let substr = self.resolve_value(args[1].trim());
            match (&val, &substr) {
                (CelValue::String(s), CelValue::String(p)) => {
                    return CelResult::ok(
                        CelValue::Int(s.find(p).map(|i| i as i64).unwrap_or(-1)),
                        expr,
                    );
                }
                _ => return CelResult::err("indexOf() requires string arguments", expr),
            }
        }
        if expr.starts_with("lower(") && expr.ends_with(')') {
            let arg = expr[6..expr.len() - 1].trim();
            let val = self.resolve_value(arg);
            match val {
                CelValue::String(s) => {
                    return CelResult::ok(CelValue::String(s.to_lowercase()), expr);
                }
                _ => return CelResult::err("lower() requires string argument", expr),
            }
        }
        if expr.starts_with("upper(") && expr.ends_with(')') {
            let arg = expr[6..expr.len() - 1].trim();
            let val = self.resolve_value(arg);
            match val {
                CelValue::String(s) => {
                    return CelResult::ok(CelValue::String(s.to_uppercase()), expr);
                }
                _ => return CelResult::err("upper() requires string argument", expr),
            }
        }
        // Type conversion functions
        if expr.starts_with("int(") && expr.ends_with(')') {
            let arg = expr[4..expr.len() - 1].trim();
            let val = self.resolve_value(arg);
            match val {
                CelValue::Int(n) => return CelResult::ok(CelValue::Int(n), expr),
                CelValue::Double(n) => return CelResult::ok(CelValue::Int(n as i64), expr),
                CelValue::String(ref s) => {
                    if let Ok(n) = s.parse::<i64>() {
                        return CelResult::ok(CelValue::Int(n), expr);
                    }
                    if let Ok(f) = s.parse::<f64>() {
                        return CelResult::ok(CelValue::Int(f as i64), expr);
                    }
                    return CelResult::err(format!("Cannot convert to int: {s}"), expr);
                }
                CelValue::Uint(n) => return CelResult::ok(CelValue::Int(n as i64), expr),
                CelValue::Bool(b) => {
                    return CelResult::ok(CelValue::Int(if b { 1 } else { 0 }), expr);
                }
                _ => return CelResult::err("int() requires compatible argument", expr),
            }
        }
        if expr.starts_with("double(") && expr.ends_with(')') {
            let arg = expr[7..expr.len() - 1].trim();
            let val = self.resolve_value(arg);
            match val {
                CelValue::Double(n) => return CelResult::ok(CelValue::Double(n), expr),
                CelValue::Int(n) => return CelResult::ok(CelValue::Double(n as f64), expr),
                CelValue::String(ref s) => {
                    if let Ok(f) = s.parse::<f64>() {
                        return CelResult::ok(CelValue::Double(f), expr);
                    }
                    return CelResult::err(format!("Cannot convert to double: {s}"), expr);
                }
                CelValue::Uint(n) => return CelResult::ok(CelValue::Double(n as f64), expr),
                CelValue::Bool(b) => {
                    return CelResult::ok(CelValue::Double(if b { 1.0 } else { 0.0 }), expr);
                }
                _ => return CelResult::err("double() requires compatible argument", expr),
            }
        }
        if expr.starts_with("bool(") && expr.ends_with(')') {
            let arg = expr[5..expr.len() - 1].trim();
            let val = self.resolve_value(arg);
            match val {
                CelValue::Bool(b) => return CelResult::ok(CelValue::Bool(b), expr),
                CelValue::String(ref s) => {
                    if s == "true" {
                        return CelResult::ok(CelValue::Bool(true), expr);
                    }
                    if s == "false" {
                        return CelResult::ok(CelValue::Bool(false), expr);
                    }
                    return CelResult::err(format!("Cannot convert to bool: {s}"), expr);
                }
                _ => return CelResult::err("bool() requires string or bool argument", expr),
            }
        }
        if expr.starts_with("string(") && expr.ends_with(')') {
            let arg = expr[7..expr.len() - 1].trim();
            let val = self.resolve_value(arg);
            let s = match &val {
                CelValue::String(s) => s.clone(),
                CelValue::Int(n) => n.to_string(),
                CelValue::Double(n) => n.to_string(),
                CelValue::Bool(b) => b.to_string(),
                _ => String::new(),
            };
            return CelResult::ok(CelValue::String(s), expr);
        }
        CelResult::err(format!("Unsupported function: {expr}"), expr)
    }

    fn eval_arithmetic(&self, expr: &str) -> CelResult {
        // Find arithmetic operator: +, -, *, /
        // Order: find * and / first (higher precedence), then + and -
        let mul_div = [("*", false), ("/", false), ("%", false)];
        let add_sub = [("+", false), ("-", false)];

        // First pass: multiply, divide, modulo
        for (op, _) in &mul_div {
            // Find rightmost occurrence of this operator (left-to-right evaluation)
            if let Some(pos) = expr.rfind(*op) {
                // Make sure it's not part of a comparison or logical operator
                if pos + 1 < expr.len() {
                    let next = expr.as_bytes()[pos + 1];
                    if next == b'=' {
                        continue;
                    }
                }
                if pos > 0 {
                    let prev = expr.as_bytes()[pos - 1];
                    if prev == b'!' || prev == b'>' || prev == b'<' {
                        continue;
                    }
                }
                let left_str = expr[..pos].trim();
                let right_str = expr[pos + 1..].trim();
                let left = self.resolve_value(left_str);
                let right = self.resolve_value(right_str);
                let op_str = *op;
                let result = match (&left, &right, op_str) {
                    (CelValue::Int(a), CelValue::Int(b), "*") => CelValue::Int(a * b),
                    (CelValue::Int(a), CelValue::Int(b), "/") => {
                        if *b == 0 {
                            return CelResult::err("Division by zero", expr);
                        }
                        CelValue::Int(a / b)
                    }
                    (CelValue::Int(a), CelValue::Int(b), "%") => {
                        if *b == 0 {
                            return CelResult::err("Modulo by zero", expr);
                        }
                        CelValue::Int(a % b)
                    }
                    (CelValue::Double(a), CelValue::Double(b), "*") => CelValue::Double(a * b),
                    (CelValue::Double(a), CelValue::Double(b), "/") => {
                        if *b == 0.0 {
                            return CelResult::err("Division by zero", expr);
                        }
                        CelValue::Double(a / b)
                    }
                    (CelValue::Double(a), CelValue::Double(b), "%") => {
                        if *b == 0.0 {
                            return CelResult::err("Modulo by zero", expr);
                        }
                        CelValue::Double(a % b)
                    }
                    // Int op Double → Double
                    (CelValue::Int(a), CelValue::Double(b), "*") => CelValue::Double(*a as f64 * b),
                    (CelValue::Int(a), CelValue::Double(b), "/") => {
                        if *b == 0.0 {
                            return CelResult::err("Division by zero", expr);
                        }
                        CelValue::Double(*a as f64 / b)
                    }
                    (CelValue::Int(a), CelValue::Double(b), "%") => {
                        if *b == 0.0 {
                            return CelResult::err("Modulo by zero", expr);
                        }
                        CelValue::Double(*a as f64 % b)
                    }
                    (CelValue::Double(a), CelValue::Int(b), "*") => CelValue::Double(a * *b as f64),
                    (CelValue::Double(a), CelValue::Int(b), "/") => {
                        let bf = *b as f64;
                        if bf == 0.0 {
                            return CelResult::err("Division by zero", expr);
                        }
                        CelValue::Double(a / bf)
                    }
                    (CelValue::Double(a), CelValue::Int(b), "%") => {
                        let bf = *b as f64;
                        if bf == 0.0 {
                            return CelResult::err("Modulo by zero", expr);
                        }
                        CelValue::Double(a % bf)
                    }
                    _ => return CelResult::err("Type mismatch in arithmetic", expr),
                };
                return CelResult::ok(result, expr);
            }
        }

        // Second pass: add, subtract
        for (op, _) in &add_sub {
            if let Some(pos) = expr.rfind(*op) {
                // Skip if it's part of comparison
                if pos + 1 < expr.len() {
                    let next = expr.as_bytes()[pos + 1];
                    if next == b'=' || next == b'>' {
                        continue;
                    }
                }
                if pos > 0 {
                    let prev = expr.as_bytes()[pos - 1];
                    if prev == b'!' || prev == b'>' || prev == b'<' || prev == b'=' {
                        continue;
                    }
                }
                let left_str = expr[..pos].trim();
                let right_str = expr[pos + 1..].trim();
                let left = self.resolve_value(left_str);
                let right = self.resolve_value(right_str);
                let op_str = *op;
                let result = match (&left, &right, op_str) {
                    (CelValue::Int(a), CelValue::Int(b), "+") => CelValue::Int(a + b),
                    (CelValue::Int(a), CelValue::Int(b), "-") => CelValue::Int(a - b),
                    (CelValue::Double(a), CelValue::Double(b), "+") => CelValue::Double(a + b),
                    (CelValue::Double(a), CelValue::Double(b), "-") => CelValue::Double(a - b),
                    // String concatenation with +
                    (CelValue::String(a), CelValue::String(b), "+") => {
                        CelValue::String(format!("{a}{b}"))
                    }
                    // Int op Double → Double
                    (CelValue::Int(a), CelValue::Double(b), "+") => CelValue::Double(*a as f64 + b),
                    (CelValue::Int(a), CelValue::Double(b), "-") => CelValue::Double(*a as f64 - b),
                    // Double op Int → Double
                    (CelValue::Double(a), CelValue::Int(b), "+") => CelValue::Double(a + *b as f64),
                    (CelValue::Double(a), CelValue::Int(b), "-") => CelValue::Double(a - *b as f64),
                    _ => return CelResult::err("Type mismatch in arithmetic", expr),
                };
                return CelResult::ok(result, expr);
            }
        }

        CelResult::err("No arithmetic operator found", expr)
    }

    fn eval_ternary(&self, expr: &str) -> CelResult {
        // Simple ternary: condition ? true_val : false_val
        if let Some(q_pos) = expr.find('?') {
            if let Some(c_pos) = expr[q_pos..].find(':') {
                let condition = expr[..q_pos].trim();
                let true_val = expr[q_pos + 1..q_pos + c_pos].trim();
                let false_val = expr[q_pos + c_pos + 1..].trim();
                // Try to evaluate as a CEL expression first; fall back to direct value resolution
                let cond_val = self.resolve_value(condition);
                if cond_val.is_truthy() {
                    return CelResult::ok(self.resolve_value(true_val), expr);
                }
                return CelResult::ok(self.resolve_value(false_val), expr);
            }
        }
        CelResult::err("Invalid ternary expression", expr)
    }

    fn eval_complex(&self, expr: &str) -> CelResult {
        if expr.contains("&&") || expr.contains("||") {
            return self.eval_logical(expr);
        }
        CelResult::err(format!("Cannot evaluate complex expression: {expr}"), expr)
    }
}

fn split_args(args_str: &str) -> Vec<&str> {
    let mut args = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    let chars: Vec<char> = args_str.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                args.push(args_str[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    if start < args_str.len() {
        args.push(args_str[start..].trim());
    }
    args
}

/// Policy rule binding a CEL expression to a decision effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelPolicyRule {
    pub id: String,
    pub description: String,
    pub expression: CelExpression,
    pub effect: PolicyEffect,
    pub priority: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyEffect {
    Allow,
    Deny,
}

/// Evaluates a set of policy rules against an environment.
pub struct CelPolicyEvaluator {
    rules: Vec<CelPolicyRule>,
}

impl CelPolicyEvaluator {
    pub fn new(rules: Vec<CelPolicyRule>) -> Self {
        Self { rules }
    }

    pub fn add_rule(&mut self, rule: CelPolicyRule) {
        self.rules.push(rule);
    }

    /// Evaluates all rules and returns the highest-priority matching decision.
    /// Deny takes precedence over Allow at the same priority level.
    pub fn evaluate(&self, env: &CelEnvironment) -> CelPolicyDecision {
        let evaluator = CelEvaluator::new(env.clone());
        let mut allow_match: Option<(&CelPolicyRule, CelResult)> = None;
        let mut deny_match: Option<(&CelPolicyRule, CelResult)> = None;
        let mut highest_allow_priority = 0u32;
        let mut highest_deny_priority = 0u32;
        for rule in &self.rules {
            let result = evaluator.evaluate(&rule.expression);
            if let Some(true) = result.as_bool() {
                match rule.effect {
                    PolicyEffect::Allow => {
                        if rule.priority > highest_allow_priority {
                            highest_allow_priority = rule.priority;
                            allow_match = Some((rule, result));
                        }
                    }
                    PolicyEffect::Deny => {
                        if rule.priority > highest_deny_priority {
                            highest_deny_priority = rule.priority;
                            deny_match = Some((rule, result));
                        }
                    }
                }
            }
        }
        // Deny takes precedence at same priority
        if let Some(deny) = deny_match {
            if highest_deny_priority >= highest_allow_priority {
                return CelPolicyDecision {
                    effect: PolicyEffect::Deny,
                    rule_id: deny.0.id.clone(),
                    reason: format!("Matched deny rule: {}", deny.0.description),
                };
            }
        }
        if let Some(allow) = allow_match {
            return CelPolicyDecision {
                effect: PolicyEffect::Allow,
                rule_id: allow.0.id.clone(),
                reason: format!("Matched allow rule: {}", allow.0.description),
            };
        }
        CelPolicyDecision {
            effect: PolicyEffect::Deny,
            rule_id: "default-deny".into(),
            reason: "No matching rule; default deny".into(),
        }
    }
}

/// Final policy decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelPolicyDecision {
    pub effect: PolicyEffect,
    pub rule_id: String,
    pub reason: String,
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use super::*;

    fn test_env() -> CelEnvironment {
        let mut user_map = HashMap::new();
        user_map.insert("role".to_string(), CelValue::String("admin".to_string()));
        user_map.insert("name".to_string(), CelValue::String("alice".to_string()));
        user_map.insert("age".to_string(), CelValue::Int(30));
        user_map.insert("suspended".to_string(), CelValue::Bool(false));
        user_map.insert(
            "permissions".to_string(),
            CelValue::List(vec![
                CelValue::String("read".into()),
                CelValue::String("write".into()),
                CelValue::String("admin".into()),
            ]),
        );
        CelEnvironment::new()
            .with_variable(
                "user",
                CelValue::Map(user_map),
                CelType::Map(Box::new(CelType::String), Box::new(CelType::Dyn)),
            )
            .with_variable(
                "source",
                CelValue::Map({
                    let mut m = HashMap::new();
                    m.insert("ip".to_string(), CelValue::String("10.0.0.1".to_string()));
                    m
                }),
                CelType::Map(Box::new(CelType::String), Box::new(CelType::Dyn)),
            )
    }

    #[test]
    fn test_cel_expression_parse_comparison() {
        let expr = CelExpression::parse("user.age > 18");
        assert_eq!(expr.kind, CelKind::Comparison);
    }

    #[test]
    fn test_cel_expression_parse_logical() {
        let expr = CelExpression::parse("user.role == \"admin\" && source.ip == \"10.0.0.1\"");
        assert_eq!(expr.kind, CelKind::Logical);
    }

    #[test]
    fn test_cel_expression_parse_membership() {
        let expr = CelExpression::parse("\"read\" in user.permissions");
        assert_eq!(expr.kind, CelKind::Membership);
    }

    #[test]
    fn test_cel_expression_parse_function() {
        let expr = CelExpression::parse("size(user.permissions) > 0");
        assert_eq!(expr.kind, CelKind::FunctionCall);
    }

    #[test]
    fn test_cel_expression_parse_negation() {
        let expr = CelExpression::parse("!user.suspended");
        assert_eq!(expr.kind, CelKind::Negation);
    }

    #[test]
    fn test_cel_expression_parse_ternary() {
        let expr = CelExpression::parse("user.verified ? \"allow\" : \"deny\"");
        assert_eq!(expr.kind, CelKind::Ternary);
    }

    #[test]
    fn test_cel_expression_parse_unknown() {
        let expr = CelExpression::parse("just_a_word");
        assert_eq!(expr.kind, CelKind::Unknown);
    }

    #[test]
    fn test_evaluate_equality_string() {
        let expr = CelExpression::parse("user.role == \"admin\"");
        let evaluator = CelEvaluator::new(test_env());
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_evaluate_inequality() {
        let expr = CelExpression::parse("user.role != \"guest\"");
        let evaluator = CelEvaluator::new(test_env());
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_evaluate_comparison_int() {
        let expr = CelExpression::parse("user.age > 18");
        let evaluator = CelEvaluator::new(test_env());
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_evaluate_comparison_false() {
        let expr = CelExpression::parse("user.age < 18");
        let evaluator = CelEvaluator::new(test_env());
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_evaluate_negation() {
        let expr = CelExpression::parse("!user.suspended");
        let evaluator = CelEvaluator::new(test_env());
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_evaluate_negation_false() {
        let mut env = test_env();
        let user_map = if let Some(CelValue::Map(m)) = env.get_variable("user").cloned() {
            m
        } else {
            panic!()
        };
        let mut new_map = user_map;
        new_map.insert("suspended".to_string(), CelValue::Bool(true));
        env = CelEnvironment::new().with_variable("user", CelValue::Map(new_map), CelType::Dyn);
        let expr = CelExpression::parse("!user.suspended");
        let evaluator = CelEvaluator::new(env);
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_evaluate_membership_in_list() {
        let expr = CelExpression::parse("\"read\" in user.permissions");
        let evaluator = CelEvaluator::new(test_env());
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_evaluate_membership_not_in_list() {
        let expr = CelExpression::parse("\"delete\" not in user.permissions");
        let evaluator = CelEvaluator::new(test_env());
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_evaluate_function_has() {
        let expr = CelExpression::parse("has(user.name)");
        let evaluator = CelEvaluator::new(test_env());
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_evaluate_function_size() {
        let expr = CelExpression::parse("size(user.permissions)");
        let evaluator = CelEvaluator::new(test_env());
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value.as_int(), Some(3));
    }

    #[test]
    fn test_evaluate_function_type() {
        let expr = CelExpression::parse("type(user.role)");
        let evaluator = CelEvaluator::new(test_env());
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value.as_string(), Some("string"));
    }

    #[test]
    fn test_evaluate_logical_and() {
        let expr = CelExpression::parse("user.role == \"admin\" && user.age > 18");
        let evaluator = CelEvaluator::new(test_env());
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_evaluate_logical_and_false() {
        let expr = CelExpression::parse("user.role == \"guest\" && user.age > 18");
        let evaluator = CelEvaluator::new(test_env());
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_evaluate_logical_or() {
        let expr = CelExpression::parse("user.role == \"guest\" || user.role == \"admin\"");
        let evaluator = CelEvaluator::new(test_env());
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_evaluate_ternary_true() {
        let env = CelEnvironment::new().with_variable("x", CelValue::Bool(true), CelType::Bool);
        let expr = CelExpression::parse("x ? \"yes\" : \"no\"");
        let evaluator = CelEvaluator::new(env);
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value.as_string(), Some("yes"));
    }

    #[test]
    fn test_evaluate_ternary_false() {
        let env = CelEnvironment::new().with_variable("x", CelValue::Bool(false), CelType::Bool);
        let expr = CelExpression::parse("x ? \"yes\" : \"no\"");
        let evaluator = CelEvaluator::new(env);
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value.as_string(), Some("no"));
    }

    #[test]
    fn test_evaluate_unknown() {
        let expr = CelExpression::parse("nonsense");
        let evaluator = CelEvaluator::new(test_env());
        let result = evaluator.evaluate(&expr);
        assert!(!result.success);
    }

    #[test]
    fn test_resolve_path() {
        let env = test_env();
        assert_eq!(
            env.resolve_path("user.role"),
            Some(CelValue::String("admin".into()))
        );
        assert_eq!(env.resolve_path("user.age"), Some(CelValue::Int(30)));
        assert_eq!(env.resolve_path("user.nonexistent"), Some(CelValue::Null));
        assert_eq!(env.resolve_path("nonexistent.path"), None);
    }

    #[test]
    fn test_cel_value_type_name() {
        assert_eq!(CelValue::Bool(true).type_name(), "bool");
        assert_eq!(CelValue::Int(42).type_name(), "int");
        assert_eq!(CelValue::String("x".into()).type_name(), "string");
        assert_eq!(CelValue::Null.type_name(), "null");
        assert_eq!(CelValue::List(vec![]).type_name(), "list");
    }

    #[test]
    fn test_cel_value_is_truthy() {
        assert!(CelValue::Bool(true).is_truthy());
        assert!(!CelValue::Bool(false).is_truthy());
        assert!(CelValue::Int(1).is_truthy());
        assert!(!CelValue::Int(0).is_truthy());
        assert!(CelValue::String("hello".into()).is_truthy());
        assert!(!CelValue::String("".into()).is_truthy());
        assert!(!CelValue::Null.is_truthy());
    }

    #[test]
    fn test_policy_evaluator_default_deny() {
        let evaluator = CelPolicyEvaluator::new(vec![]);
        let env = test_env();
        let decision = evaluator.evaluate(&env);
        assert_eq!(decision.effect, PolicyEffect::Deny);
        assert_eq!(decision.rule_id, "default-deny");
    }

    #[test]
    fn test_policy_evaluator_allow_rule() {
        let rules = vec![CelPolicyRule {
            id: "r1".into(),
            description: "Admin access".into(),
            expression: CelExpression::parse("user.role == \"admin\""),
            effect: PolicyEffect::Allow,
            priority: 10,
        }];
        let evaluator = CelPolicyEvaluator::new(rules);
        let env = test_env();
        let decision = evaluator.evaluate(&env);
        assert_eq!(decision.effect, PolicyEffect::Allow);
        assert_eq!(decision.rule_id, "r1");
    }

    #[test]
    fn test_policy_evaluator_deny_takes_precedence() {
        let rules = vec![
            CelPolicyRule {
                id: "allow".into(),
                description: "Allow admin".into(),
                expression: CelExpression::parse("user.role == \"admin\""),
                effect: PolicyEffect::Allow,
                priority: 10,
            },
            CelPolicyRule {
                id: "deny".into(),
                description: "Deny suspended".into(),
                expression: CelExpression::parse("user.suspended"),
                effect: PolicyEffect::Deny,
                priority: 10,
            },
        ];
        let evaluator = CelPolicyEvaluator::new(rules);
        let env = test_env();
        let decision = evaluator.evaluate(&env);
        // user.suspended is false, so deny rule doesn't match, allow should win
        assert_eq!(decision.effect, PolicyEffect::Allow);
    }

    #[test]
    fn test_policy_evaluator_higher_priority_wins() {
        let rules = vec![
            CelPolicyRule {
                id: "low-allow".into(),
                description: "Low priority allow".into(),
                expression: CelExpression::parse("user.age > 18"),
                effect: PolicyEffect::Allow,
                priority: 5,
            },
            CelPolicyRule {
                id: "high-deny".into(),
                description: "High priority deny".into(),
                expression: CelExpression::parse("user.age > 18"),
                effect: PolicyEffect::Deny,
                priority: 20,
            },
        ];
        let evaluator = CelPolicyEvaluator::new(rules);
        let env = test_env();
        let decision = evaluator.evaluate(&env);
        assert_eq!(decision.effect, PolicyEffect::Deny);
        assert_eq!(decision.rule_id, "high-deny");
    }

    #[test]
    fn test_cel_expression_serialization() {
        let expr = CelExpression::parse("user.age > 18");
        let json = serde_json::to_string(&expr).unwrap();
        let de: CelExpression = serde_json::from_str(&json).unwrap();
        assert_eq!(de.kind, CelKind::Comparison);
    }

    #[test]
    fn test_cel_result_serialization() {
        let result = CelResult::ok(CelValue::Bool(true), "test");
        let json = serde_json::to_string(&result).unwrap();
        let de: CelResult = serde_json::from_str(&json).unwrap();
        assert!(de.success);
    }

    #[test]
    fn test_cel_policy_decision_serialization() {
        let decision = CelPolicyDecision {
            effect: PolicyEffect::Allow,
            rule_id: "r1".into(),
            reason: "test".into(),
        };
        let json = serde_json::to_string(&decision).unwrap();
        let de: CelPolicyDecision = serde_json::from_str(&json).unwrap();
        assert_eq!(de.rule_id, "r1");
    }

    #[test]
    fn test_cel_environment_serialization() {
        let env = CelEnvironment::new().with_variable("x", CelValue::Int(42), CelType::Int);
        // CelEnvironment itself doesn't need serialization, but values do
        let val = env.get_variable("x").unwrap();
        let json = serde_json::to_string(val).unwrap();
        let de: CelValue = serde_json::from_str(&json).unwrap();
        assert_eq!(de.as_int(), Some(42));
    }

    #[test]
    fn test_cel_value_list_equality() {
        let a = CelValue::List(vec![
            CelValue::String("a".into()),
            CelValue::String("b".into()),
        ]);
        let b = CelValue::List(vec![
            CelValue::String("a".into()),
            CelValue::String("b".into()),
        ]);
        assert_eq!(a, b);
    }

    #[test]
    fn test_cel_value_map_equality() {
        let mut m1 = HashMap::new();
        m1.insert("k".to_string(), CelValue::Int(1));
        let mut m2 = HashMap::new();
        m2.insert("k".to_string(), CelValue::Int(1));
        assert_eq!(CelValue::Map(m1), CelValue::Map(m2));
    }

    #[test]
    fn test_evaluate_function_starts_with() {
        let env = CelEnvironment::new().with_variable(
            "x",
            CelValue::String("hello world".into()),
            CelType::String,
        );
        let expr = CelExpression::parse("startsWith(x, \"hello\")");
        let evaluator = CelEvaluator::new(env);
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(true));

        let env = CelEnvironment::new().with_variable(
            "x",
            CelValue::String("hello world".into()),
            CelType::String,
        );
        let expr = CelExpression::parse("startsWith(x, \"world\")");
        let evaluator = CelEvaluator::new(env);
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_evaluate_function_ends_with() {
        let env = CelEnvironment::new().with_variable(
            "x",
            CelValue::String("hello world".into()),
            CelType::String,
        );
        let expr = CelExpression::parse("endsWith(x, \"world\")");
        let evaluator = CelEvaluator::new(env);
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(true));

        let env = CelEnvironment::new().with_variable(
            "x",
            CelValue::String("hello world".into()),
            CelType::String,
        );
        let expr = CelExpression::parse("endsWith(x, \"hello\")");
        let evaluator = CelEvaluator::new(env);
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_evaluate_function_contains() {
        let env = CelEnvironment::new().with_variable(
            "x",
            CelValue::String("hello world".into()),
            CelType::String,
        );
        let expr = CelExpression::parse("contains(x, \"lo wo\")");
        let evaluator = CelEvaluator::new(env);
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(true));

        let env = CelEnvironment::new().with_variable(
            "x",
            CelValue::String("hello world".into()),
            CelType::String,
        );
        let expr = CelExpression::parse("contains(x, \"xyz\")");
        let evaluator = CelEvaluator::new(env);
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_evaluate_function_matches() {
        let env = CelEnvironment::new().with_variable(
            "x",
            CelValue::String("hello 123".into()),
            CelType::String,
        );
        let expr = CelExpression::parse(r#"matches(x, "hello \d+")"#);
        let evaluator = CelEvaluator::new(env);
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_evaluate_function_matches_invalid_regex() {
        let env = CelEnvironment::new().with_variable(
            "x",
            CelValue::String("hello".into()),
            CelType::String,
        );
        let expr = CelExpression::parse(r#"matches(x, "[invalid")"#);
        let evaluator = CelEvaluator::new(env);
        let result = evaluator.evaluate(&expr);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Invalid regex"));
    }

    // -----------------------------------------------------------------------
    // Arithmetic tests
    // -----------------------------------------------------------------------

    fn make_env() -> CelEnvironment {
        CelEnvironment::new()
            .with_variable("x", CelValue::Int(10), CelType::Int)
            .with_variable("y", CelValue::Int(3), CelType::Int)
            .with_variable("pi", CelValue::Double(3.14159), CelType::Double)
            .with_variable("name", CelValue::String("hello".into()), CelType::String)
    }

    #[test]
    fn test_arithmetic_int_add() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse("x + y");
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value, CelValue::Int(13));
    }

    #[test]
    fn test_arithmetic_int_subtract() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse("x - y");
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value, CelValue::Int(7));
    }

    #[test]
    fn test_arithmetic_int_multiply() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse("x * y");
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value, CelValue::Int(30));
    }

    #[test]
    fn test_arithmetic_int_divide() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse("x / y");
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value, CelValue::Int(3));
    }

    #[test]
    fn test_arithmetic_int_modulo() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse("10 % 3");
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value, CelValue::Int(1));
    }

    #[test]
    fn test_arithmetic_divide_by_zero() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse("x / 0");
        let result = evaluator.evaluate(&expr);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Division by zero"));
    }

    #[test]
    fn test_arithmetic_double() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse("pi * 2");
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        match result.value {
            CelValue::Double(v) => assert!((v - 6.28318).abs() < 0.01),
            _ => panic!("expected Double"),
        }
    }

    #[test]
    fn test_arithmetic_int_double_mix() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse("x + pi");
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        match result.value {
            CelValue::Double(v) => assert!((v - 13.14159).abs() < 0.01),
            _ => panic!("expected Double"),
        }
    }

    #[test]
    fn test_arithmetic_string_concat() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse(r#"name + " world""#);
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value, CelValue::String("hello world".into()));
    }

    #[test]
    fn test_arithmetic_parens() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse("(x + y) * 2");
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value, CelValue::Int(26));
    }

    // -----------------------------------------------------------------------
    // Math function tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_function_abs_int() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse("abs(-5)");
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value, CelValue::Int(5));
    }

    #[test]
    fn test_function_abs_double() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse("abs(-3.7)");
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        match result.value {
            CelValue::Double(v) => assert!((v - 3.7).abs() < 0.01),
            _ => panic!("expected Double"),
        }
    }

    #[test]
    fn test_function_ceil() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse("ceil(3.2999)");
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        match result.value {
            CelValue::Double(v) => assert!((v - 4.0).abs() < 0.01),
            _ => panic!("expected Double"),
        }
    }

    #[test]
    fn test_function_floor() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse("floor(3.9001)");
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        match result.value {
            CelValue::Double(v) => assert!((v - 3.0).abs() < 0.01),
            _ => panic!("expected Double"),
        }
    }

    #[test]
    fn test_function_max() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse("max(x, y)");
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value, CelValue::Int(10));
    }

    #[test]
    fn test_function_min() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse("min(x, y)");
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value, CelValue::Int(3));
    }

    // -----------------------------------------------------------------------
    // String function tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_function_indexof_found() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse(r#"indexOf(name, "ll")"#);
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value, CelValue::Int(2));
    }

    #[test]
    fn test_function_indexof_not_found() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse(r#"indexOf(name, "xyz")"#);
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value, CelValue::Int(-1));
    }

    #[test]
    fn test_function_lower() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse(r#"lower("HELLO")"#);
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value, CelValue::String("hello".into()));
    }

    #[test]
    fn test_function_upper() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse(r#"upper("hello")"#);
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value, CelValue::String("HELLO".into()));
    }

    // -----------------------------------------------------------------------
    // Type conversion function tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_function_int_from_string() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse(r#"int("42")"#);
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value, CelValue::Int(42));
    }

    #[test]
    fn test_function_int_from_double() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse("int(pi)");
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value, CelValue::Int(3));
    }

    #[test]
    fn test_function_int_from_bool() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse("int(true)");
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value, CelValue::Int(1));
    }

    #[test]
    fn test_function_double_from_int() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse("double(x)");
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        match result.value {
            CelValue::Double(v) => assert!((v - 10.0).abs() < 0.01),
            _ => panic!("expected Double"),
        }
    }

    #[test]
    fn test_function_double_from_string() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse(r#"double("3.14159")"#);
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        match result.value {
            CelValue::Double(v) => assert!((v - 3.14159).abs() < 0.01),
            _ => panic!("expected Double"),
        }
    }

    #[test]
    fn test_function_bool_from_string() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse(r#"bool("true")"#);
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value, CelValue::Bool(true));
    }

    #[test]
    fn test_function_bool_from_string_false() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse(r#"bool("false")"#);
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value, CelValue::Bool(false));
    }

    #[test]
    fn test_function_string_from_int() {
        let env = make_env();
        let evaluator = CelEvaluator::new(env);
        let expr = CelExpression::parse("string(x)");
        let result = evaluator.evaluate(&expr);
        assert!(result.success);
        assert_eq!(result.value, CelValue::String("10".into()));
    }
}
