//! Trigger matching logic.
//!
//! Determines whether a pipeline should execute based on event context
//! (push branches/tags/paths, PR branches, schedule, manual dispatch).

use crate::model::{Pipeline, TriggerConfig};
use chrono::{Datelike, Duration, NaiveDate, TimeZone, Timelike, Utc};

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

/// Compute the next run time for a 5-field cron expression after `after`.
///
/// Returns `None` if the cron expression is invalid or no future run can be found
/// within a reasonable horizon (366 days).
pub fn compute_next_cron_run(
    cron: &str,
    after: &chrono::DateTime<Utc>,
) -> Option<chrono::DateTime<Utc>> {
    let fields: Vec<&str> = cron.split_whitespace().collect();
    if fields.len() != 5 {
        return None;
    }

    let minutes = parse_cron_field(fields[0], 0, 59)?;
    let hours = parse_cron_field(fields[1], 0, 23)?;
    let days_of_month = parse_cron_field(fields[2], 1, 31)?;
    let months = parse_cron_field(fields[3], 1, 12)?;
    let days_of_week = parse_cron_field(fields[4], 0, 6)?;

    // Start searching from the next minute
    let mut candidate = *after + Duration::minutes(1);
    // Zero out seconds and sub-minute precision
    candidate = candidate.with_second(0)?.with_nanosecond(0)?;

    // Limit search to 366 days to avoid infinite loops
    let horizon = *after + Duration::days(366);

    while candidate <= horizon {
        let month = candidate.month();
        if !months.contains(&month) {
            // Jump to first day of next month
            candidate = advance_to_next_month(candidate)?;
            continue;
        }

        let dom = candidate.day();
        let dow = candidate.weekday().num_days_from_sunday();
        if !days_of_month.contains(&dom) || !days_of_week.contains(&dow) {
            // Jump to next day
            candidate = advance_to_next_day(candidate)?;
            continue;
        }

        let hour = candidate.hour();
        if !hours.contains(&hour) {
            // Jump to next hour
            candidate = advance_to_next_hour(candidate)?;
            continue;
        }

        let minute = candidate.minute();
        if !minutes.contains(&minute) {
            // Jump to next minute
            candidate += Duration::minutes(1);
            continue;
        }

        return Some(candidate);
    }

    None
}

fn parse_cron_field(field: &str, min: u32, max: u32) -> Option<Vec<u32>> {
    let mut values = Vec::new();

    for part in field.split(',') {
        let part = part.trim();
        if part == "*" {
            values.extend(min..=max);
        } else if let Some((range, step_str)) = part.split_once('/') {
            let step: u32 = step_str.parse().ok()?;
            if step == 0 {
                return None;
            }
            if let Some((start_s, end_s)) = range.split_once('-') {
                let start: u32 = start_s.parse().ok()?;
                let end: u32 = end_s.parse().ok()?;
                if start > end || start < min || end > max {
                    return None;
                }
                let mut i = start;
                while i <= end {
                    values.push(i);
                    i += step;
                }
            } else if range == "*" {
                let mut i = min;
                while i <= max {
                    values.push(i);
                    i += step;
                }
            } else {
                return None;
            }
        } else if let Some((start_s, end_s)) = part.split_once('-') {
            let start: u32 = start_s.parse().ok()?;
            let end: u32 = end_s.parse().ok()?;
            if start > end || start < min || end > max {
                return None;
            }
            values.extend(start..=end);
        } else {
            let val: u32 = part.parse().ok()?;
            if val < min || val > max {
                return None;
            }
            values.push(val);
        }
    }

    values.sort_unstable();
    values.dedup();
    Some(values)
}

fn advance_to_next_month(dt: chrono::DateTime<Utc>) -> Option<chrono::DateTime<Utc>> {
    let (y, m) = if dt.month() == 12 {
        (dt.year() + 1, 1)
    } else {
        (dt.year(), dt.month() + 1)
    };
    let naive = NaiveDate::from_ymd_opt(y, m, 1)?.and_hms_opt(0, 0, 0)?;
    Utc.from_local_datetime(&naive).single()
}

fn advance_to_next_day(dt: chrono::DateTime<Utc>) -> Option<chrono::DateTime<Utc>> {
    let next = dt.date_naive() + Duration::days(1);
    let naive = next.and_hms_opt(0, 0, 0)?;
    Utc.from_local_datetime(&naive).single()
}

fn advance_to_next_hour(dt: chrono::DateTime<Utc>) -> Option<chrono::DateTime<Utc>> {
    let naive = dt.date_naive().and_hms_opt(dt.hour() + 1, 0, 0)?;
    Utc.from_local_datetime(&naive).single()
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

    #[test]
    fn compute_next_cron_every_minute() {
        // "* * * * *" — every minute
        let after = Utc.with_ymd_and_hms(2025, 6, 1, 12, 0, 0).unwrap();
        let next = compute_next_cron_run("* * * * *", &after).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2025, 6, 1, 12, 1, 0).unwrap());
    }

    #[test]
    fn compute_next_cron_weekly_monday_2am() {
        // "0 2 * * 1" — every Monday at 2:00am
        // 2025-06-01 is a Sunday
        let after = Utc.with_ymd_and_hms(2025, 6, 1, 12, 0, 0).unwrap();
        let next = compute_next_cron_run("0 2 * * 1", &after).unwrap();
        // Next Monday is 2025-06-02
        assert_eq!(next, Utc.with_ymd_and_hms(2025, 6, 2, 2, 0, 0).unwrap());
    }

    #[test]
    fn compute_next_cron_weekday_830am() {
        // "30 8 * * 1-5" — weekdays at 8:30am
        // 2025-06-01 is Sunday, next weekday is Monday 2025-06-02
        let after = Utc.with_ymd_and_hms(2025, 6, 1, 12, 0, 0).unwrap();
        let next = compute_next_cron_run("30 8 * * 1-5", &after).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2025, 6, 2, 8, 30, 0).unwrap());
    }

    #[test]
    fn compute_next_cron_step() {
        // "*/15 * * * *" — every 15 minutes
        let after = Utc.with_ymd_and_hms(2025, 6, 1, 12, 7, 0).unwrap();
        let next = compute_next_cron_run("*/15 * * * *", &after).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2025, 6, 1, 12, 15, 0).unwrap());
    }

    #[test]
    fn compute_next_cron_invalid() {
        assert!(compute_next_cron_run("", &Utc::now()).is_none());
        assert!(compute_next_cron_run("60 * * * *", &Utc::now()).is_none());
        assert!(compute_next_cron_run("0 24 * * *", &Utc::now()).is_none());
    }

    #[test]
    fn compute_next_cron_range() {
        // "0 9-17 * * 1-5" — every hour from 9am to 5pm on weekdays
        // 2025-06-06 is a Friday
        let after = Utc.with_ymd_and_hms(2025, 6, 6, 14, 30, 0).unwrap();
        let next = compute_next_cron_run("0 9-17 * * 1-5", &after).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2025, 6, 6, 15, 0, 0).unwrap());
    }
}
