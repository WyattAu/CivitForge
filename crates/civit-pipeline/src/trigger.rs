//! Trigger matching logic.
//!
//! Determines whether a pipeline should execute based on event context
//! (push branches/tags/paths, PR branches, schedule, manual dispatch).

use crate::model::{Pipeline, TriggerConfig};

/// Event context for trigger evaluation.
#[derive(Debug, Clone)]
pub struct TriggerContext {
    /// Event type: "push", "pull_request", "schedule", "workflow_dispatch".
    pub event_type: String,
    /// Ref name (branch or tag). For PRs, the target branch.
    pub ref_name: Option<String>,
    /// Changed files (for push path filters).
    pub changed_files: Vec<String>,
    /// Commit SHA that triggered this.
    pub sha: Option<String>,
}

impl TriggerContext {
    /// Create a push trigger context.
    pub fn push(ref_name: impl Into<String>, changed_files: Vec<String>) -> Self {
        Self {
            event_type: "push".to_string(),
            ref_name: Some(ref_name.into()),
            changed_files,
            sha: None,
        }
    }

    /// Create a pull_request trigger context.
    pub fn pull_request(target_branch: impl Into<String>) -> Self {
        Self {
            event_type: "pull_request".to_string(),
            ref_name: Some(target_branch.into()),
            changed_files: Vec::new(),
            sha: None,
        }
    }

    /// Create a schedule trigger context.
    pub fn schedule() -> Self {
        Self {
            event_type: "schedule".to_string(),
            ref_name: None,
            changed_files: Vec::new(),
            sha: None,
        }
    }

    /// Create a manual dispatch context.
    pub fn workflow_dispatch() -> Self {
        Self {
            event_type: "workflow_dispatch".to_string(),
            ref_name: None,
            changed_files: Vec::new(),
            sha: None,
        }
    }
}

/// Evaluate whether a pipeline should trigger for the given context.
///
/// Returns true if the pipeline has matching trigger configuration for this event.
/// If no `on:` is specified, the pipeline triggers on all events (default behavior).
pub fn matches_trigger(pipeline: &Pipeline, ctx: &TriggerContext) -> bool {
    let triggers = match &pipeline.on {
        Some(t) => t,
        None => return true, // No triggers specified → always runs
    };

    match ctx.event_type.as_str() {
        "push" => matches_push(triggers, ctx),
        "pull_request" => matches_pr(triggers, ctx),
        "schedule" => matches_schedule(triggers),
        "workflow_dispatch" => matches_dispatch(triggers),
        _ => false,
    }
}

fn matches_push(triggers: &TriggerConfig, ctx: &TriggerContext) -> bool {
    let push = match &triggers.push {
        Some(p) => p,
        None => return false,
    };

    let ref_name = match &ctx.ref_name {
        Some(r) => r.as_str(),
        None => return true, // No ref context → match
    };

    // Branch filter
    if let Some(branches) = &push.branches {
        if !branches.iter().any(|b| glob_matches(b, ref_name)) {
            return false;
        }
    }

    // Tag filter — check if the ref looks like a tag (starts with refs/tags/ or we check both)
    if let Some(tags) = &push.tags {
        if !tags.iter().any(|t| glob_matches(t, ref_name)) {
            return false;
        }
    }

    // Path include filter
    if let Some(paths) = &push.paths {
        let has_matching_file = ctx
            .changed_files
            .iter()
            .any(|f| paths.iter().any(|p| glob_matches(p, f)));
        if !has_matching_file && !ctx.changed_files.is_empty() {
            return false;
        }
    }

    // Path ignore filter
    if let Some(ignore) = &push.paths_ignore {
        if ctx
            .changed_files
            .iter()
            .any(|f| ignore.iter().any(|p| glob_matches(p, f)))
        {
            return false;
        }
    }

    true
}

fn matches_pr(triggers: &TriggerConfig, ctx: &TriggerContext) -> bool {
    let pr = match &triggers.pull_request {
        Some(p) => p,
        None => return false,
    };

    let ref_name = match &ctx.ref_name {
        Some(r) => r.as_str(),
        None => return true,
    };

    if let Some(branches) = &pr.branches {
        branches.iter().any(|b| glob_matches(b, ref_name))
    } else {
        true
    }
}

fn matches_schedule(triggers: &TriggerConfig) -> bool {
    triggers.schedule.is_some()
}

fn matches_dispatch(triggers: &TriggerConfig) -> bool {
    triggers.workflow_dispatch.is_some()
}

/// Glob-style pattern matching.
///
/// Supports:
/// - `*` — matches any sequence of characters (excluding `/`)
/// - `**` — matches any sequence including `/`
/// - `?` — matches any single character
/// - Exact match
pub fn glob_matches(pattern: &str, value: &str) -> bool {
    if pattern == value {
        return true;
    }

    // Handle ** specially — split on it and match segments
    if pattern.contains("**") {
        return glob_double_star(pattern, value);
    }

    // Simple glob: split by /
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let value_parts: Vec<&str> = value.split('/').collect();

    if pattern_parts.len() != value_parts.len() {
        return false;
    }

    pattern_parts
        .iter()
        .zip(value_parts.iter())
        .all(|(p, v)| glob_segment(p, v))
}

/// Match a single path segment (no `/`).
fn glob_segment(pattern: &str, value: &str) -> bool {
    let mut pi = pattern.chars().peekable();
    let mut vi = value.chars().peekable();

    while pi.peek().is_some() {
        let pc = pi.next().unwrap();
        match pc {
            '*' => {
                // Consume all remaining characters in value segment
                while vi.peek().is_some() {
                    vi.next();
                }
            }
            '?' => {
                if vi.next().is_none() {
                    return false;
                }
            }
            c => {
                if vi.next() != Some(c) {
                    return false;
                }
            }
        }
    }

    vi.peek().is_none()
}

/// Handle `**` patterns (cross-segment matching).
fn glob_double_star(pattern: &str, value: &str) -> bool {
    // Split pattern on **
    let parts: Vec<&str> = pattern.split("**").collect();

    if parts.len() == 2 {
        let (prefix, suffix) = (
            parts[0].trim_end_matches('/'),
            parts[1].trim_start_matches('/'),
        );

        // ** matches everything between prefix and suffix
        if !value.starts_with(prefix) {
            return false;
        }
        let rest = &value[prefix.len()..];

        if suffix.is_empty() {
            return true; // prefix/** matches everything under prefix
        }

        // Find suffix anywhere in rest
        let search = suffix.strip_prefix('/').unwrap_or(suffix);

        // Search for the suffix pattern in remaining value
        rest.contains(search)
    } else {
        // Multiple ** — recursive approach
        let first_star = pattern.find("**").unwrap();
        let prefix = &pattern[..first_star];
        let rest = &pattern[first_star + 2..];

        if !value.starts_with(prefix) {
            return false;
        }

        let remaining = &value[prefix.len()..];
        // Try every possible split point
        for i in 0..=remaining.len() {
            if glob_double_star(rest, &remaining[i..]) {
                return true;
            }
        }
        false
    }
}

/// Basic cron validation (5-field: min hour dom month dow).
/// Does not validate advanced syntax (e.g. step values like */15).
pub fn validate_cron(cron: &str) -> bool {
    let fields: Vec<&str> = cron.split_whitespace().collect();
    if fields.len() != 5 {
        return false;
    }

    // Validate each field is a valid cron token
    for (i, field) in fields.iter().enumerate() {
        if !validate_cron_field(field, i) {
            return false;
        }
    }

    true
}

fn validate_cron_field(field: &str, index: usize) -> bool {
    let (min, max) = match index {
        0 => (0, 59), // minute
        1 => (0, 23), // hour
        2 => (1, 31), // day of month
        3 => (1, 12), // month
        4 => (0, 6),  // day of week (0=Sunday)
        _ => return false,
    };

    // "*" is valid
    if field == "*" {
        return true;
    }

    // Step values: */N or min-max/N
    if let Some(rest) = field.strip_prefix("*/") {
        if let Ok(n) = rest.parse::<u32>() {
            return n >= 1;
        }
        return false;
    }

    // Range: min-max or min-max/step
    if field.contains('-') {
        let parts: Vec<&str> = field.split('/').collect();
        let range = parts[0];
        let range_parts: Vec<&str> = range.split('-').collect();
        if range_parts.len() != 2 {
            return false;
        }
        if let (Ok(start), Ok(end)) = (range_parts[0].parse::<u32>(), range_parts[1].parse::<u32>())
        {
            if start < min as u32 || end > max as u32 || start > end {
                return false;
            }
            // Validate step if present
            if parts.len() > 1 {
                if let Ok(step) = parts[1].parse::<u32>() {
                    return step >= 1;
                }
                return false;
            }
            return true;
        }
        return false;
    }

    // Comma-separated values: a,b,c
    if field.contains(',') {
        return field.split(',').all(|v| {
            v.trim()
                .parse::<u32>()
                .is_ok_and(|n| n >= min as u32 && n <= max as u32)
        });
    }

    // Single number
    field
        .parse::<u32>()
        .is_ok_and(|n| n >= min as u32 && n <= max as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_exact_match() {
        assert!(glob_matches("main", "main"));
        assert!(glob_matches("release/v1", "release/v1"));
        assert!(!glob_matches("main", "dev"));
    }

    #[test]
    fn glob_wildcard_star() {
        assert!(glob_matches("release/*", "release/v1.0"));
        assert!(glob_matches("release/*", "release/v2"));
        assert!(!glob_matches("release/*", "release/v1/patch")); // * doesn't cross /
        assert!(glob_matches("*", "main"));
    }

    #[test]
    fn glob_double_star() {
        assert!(glob_matches("**", "any/path/here"));
        assert!(glob_matches("src/**", "src/main.rs"));
        assert!(glob_matches("src/**", "src/deep/nested/file.rs"));
        assert!(!glob_matches("src/**", "lib/main.rs"));
        assert!(glob_matches("**.rs", "src/main.rs"));
        assert!(glob_matches("src/**.rs", "src/deep/test.rs"));
    }

    #[test]
    fn glob_question_mark() {
        assert!(glob_matches("issue-?", "issue-1"));
        assert!(glob_matches("issue-?", "issue-A"));
        assert!(!glob_matches("issue-?", "issue-10"));
    }

    #[test]
    fn cron_valid_expressions() {
        assert!(validate_cron("0 6 * * 1"));
        assert!(validate_cron("* * * * *"));
        assert!(validate_cron("*/15 * * * *"));
        assert!(validate_cron("0 9 1 * *"));
        assert!(validate_cron("30 4 1,15 * *"));
        assert!(validate_cron("0 0 * * 1-5"));
    }

    #[test]
    fn cron_invalid_expressions() {
        assert!(!validate_cron(""));
        assert!(!validate_cron("0 6 * *")); // Only 4 fields
        assert!(!validate_cron("0 6 * * 1 2")); // 6 fields
        assert!(!validate_cron("60 * * * *")); // minute > 59
        assert!(!validate_cron("0 24 * * *")); // hour > 23
        assert!(!validate_cron("0 0 32 * *")); // day > 31
        assert!(!validate_cron("0 0 0 13 *")); // month > 12
        assert!(!validate_cron("0 0 * * 7")); // dow > 6
        assert!(!validate_cron("*/0 * * * *")); // step of 0
    }

    #[test]
    fn trigger_no_on_always_matches() {
        let yaml = "version: '1'\njobs:\n  - name: test\n    steps: []\n";
        let pipeline: Pipeline = serde_yaml::from_str(yaml).unwrap();
        let ctx = TriggerContext::push("main", vec![]);
        assert!(matches_trigger(&pipeline, &ctx));
    }

    #[test]
    fn trigger_push_branch_filter() {
        let yaml = r#"
version: '1'
on:
  push:
    branches: [main, release/*]
jobs:
  - name: test
    steps:
      - name: build
        run: ["cargo build"]
"#;
        let pipeline: Pipeline = serde_yaml::from_str(yaml).unwrap();
        let ctx = TriggerContext::push("main", vec![]);
        assert!(matches_trigger(&pipeline, &ctx));

        let ctx = TriggerContext::push("release/v1.0", vec![]);
        assert!(matches_trigger(&pipeline, &ctx));

        let ctx = TriggerContext::push("dev", vec![]);
        assert!(!matches_trigger(&pipeline, &ctx));
    }

    #[test]
    fn trigger_push_path_filter() {
        let yaml = r#"
version: '1'
on:
  push:
    paths: [src/**, Cargo.toml]
jobs:
  - name: test
    steps:
      - name: build
        run: ["cargo build"]
"#;
        let pipeline: Pipeline = serde_yaml::from_str(yaml).unwrap();

        let ctx = TriggerContext::push("main", vec!["src/main.rs".to_string()]);
        assert!(matches_trigger(&pipeline, &ctx));

        let ctx = TriggerContext::push("main", vec!["Cargo.toml".to_string()]);
        assert!(matches_trigger(&pipeline, &ctx));

        let ctx = TriggerContext::push("main", vec!["README.md".to_string()]);
        assert!(!matches_trigger(&pipeline, &ctx));
    }

    #[test]
    fn trigger_push_path_ignore() {
        let yaml = r#"
version: '1'
on:
  push:
    paths_ignore: [docs/**, "*.md"]
jobs:
  - name: test
    steps:
      - name: build
        run: ["cargo build"]
"#;
        let pipeline: Pipeline = serde_yaml::from_str(yaml).unwrap();

        let ctx = TriggerContext::push("main", vec!["docs/guide.md".to_string()]);
        assert!(!matches_trigger(&pipeline, &ctx));

        let ctx = TriggerContext::push("main", vec!["src/main.rs".to_string()]);
        assert!(matches_trigger(&pipeline, &ctx));
    }

    #[test]
    fn trigger_pr_branch_filter() {
        let yaml = r#"
version: '1'
on:
  pull_request:
    branches: [main, develop]
jobs:
  - name: test
    steps:
      - name: build
        run: ["cargo build"]
"#;
        let pipeline: Pipeline = serde_yaml::from_str(yaml).unwrap();

        let ctx = TriggerContext::pull_request("main");
        assert!(matches_trigger(&pipeline, &ctx));

        let ctx = TriggerContext::pull_request("feature/x");
        assert!(!matches_trigger(&pipeline, &ctx));
    }

    #[test]
    fn trigger_schedule() {
        let yaml = r#"
version: '1'
on:
  schedule:
    - cron: "0 6 * * 1"
      name: "Weekly build"
jobs:
  - name: test
    steps:
      - name: build
        run: ["cargo build"]
"#;
        let pipeline: Pipeline = serde_yaml::from_str(yaml).unwrap();

        let ctx = TriggerContext::schedule();
        assert!(matches_trigger(&pipeline, &ctx));

        let ctx = TriggerContext::push("main", vec![]);
        assert!(!matches_trigger(&pipeline, &ctx));
    }

    #[test]
    fn trigger_dispatch() {
        let yaml = r#"
version: '1'
on:
  workflow_dispatch:
    inputs:
      - name: environment
        type: string
        required: true
jobs:
  - name: deploy
    steps:
      - name: deploy
        run: ["echo deploying"]
"#;
        let pipeline: Pipeline = serde_yaml::from_str(yaml).unwrap();

        let ctx = TriggerContext::workflow_dispatch();
        assert!(matches_trigger(&pipeline, &ctx));

        let ctx = TriggerContext::push("main", vec![]);
        assert!(!matches_trigger(&pipeline, &ctx));
    }
}
