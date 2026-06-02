//! CEL expression stub for pipeline conditions.
//! Full CEL evaluation will delegate to the existing civit-crypto CEL evaluator.

/// A simple pipeline expression (CEL subset for if/when conditions).
/// Full evaluation requires the civit-crypto CEL evaluator.
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

    /// Placeholder: full evaluation happens at runtime via civit-crypto.
    /// Returns true if the expression is non-empty (optimistic default).
    pub fn is_truthy(&self) -> bool {
        !self.raw.trim().is_empty()
    }
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
