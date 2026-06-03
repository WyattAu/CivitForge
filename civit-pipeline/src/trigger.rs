//! Trigger matching: evaluate whether a git event matches pipeline triggers.

use crate::model::{TriggerConfig, PushTrigger, PrTrigger, ScheduleTrigger};

/// Represents the type of event that occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    Push,
    PullRequest,
    Schedule,
    Manual,
}

/// A git event context for trigger matching.
#[derive(Debug, Clone)]
pub struct TriggerContext {
    pub trigger_type: Trigger,
    pub ref_name: String,
    pub commit_sha: String,
    pub changed_files: Vec<String>,
    pub base_ref: Option<String>,
}

/// Check if a trigger context matches the pipeline's trigger configuration.
pub fn matches_trigger(config: &TriggerConfig, ctx: &TriggerContext) -> bool {
    match ctx.trigger_type {
        Trigger::Push => matches_push(config.push.as_ref(), ctx),
        Trigger::PullRequest => matches_pr(config.pull_request.as_ref(), ctx),
        Trigger::Schedule => false, // Schedule triggers are evaluated by the scheduler
        Trigger::Manual => true, // Manual dispatch always triggers
    }
}

fn matches_push(trigger: Option<&PushTrigger>, ctx: &TriggerContext) -> bool {
    let t = match trigger {
        Some(t) => t,
        None => return false, // No push trigger = don't auto-trigger on push
    };

    // Branch match (glob-style)
    if let Some(branches) = &t.branches {
        if !branches.iter().any(|b| pattern_matches(b, &ctx.ref_name)) {
            return false;
        }
    }

    // Tag match
    if let Some(tags) = &t.tags {
        if !tags.iter().any(|tag| ctx.ref_name.ends_with(tag)) {
            return false;
        }
    }

    // If no branch/tag filters, any push matches
    if t.branches.is_none() && t.tags.is_none() {
        return true;
    }

    // Path filter
    if let Some(paths) = &t.paths {
        if !ctx.changed_files.is_empty() {
            if !ctx.changed_files.iter().any(|f| paths_match(paths, f)) {
                return false;
            }
        }
    }

    // paths_ignore filter
    if let Some(exclude) = &t.paths_ignore {
        if !ctx.changed_files.is_empty() {
            if ctx.changed_files.iter().any(|f| paths_match(exclude, f)) {
                return false;
            }
        }
    }

    true
}

fn matches_pr(trigger: Option<&PrTrigger>, ctx: &TriggerContext) -> bool {
    let t = match trigger {
        Some(t) => t,
        None => return false,
    };

    if let Some(branches) = &t.branches {
        if !branches.iter().any(|b| pattern_matches(b, &ctx.ref_name)) {
            return false;
        }
    }
    true
}

/// Simple glob-style pattern matching.
/// Supports exact match, prefix wildcard "release/*", and suffix wildcard "*.md".
fn pattern_matches(pattern: &str, value: &str) -> bool {
    if pattern == value {
        return true;
    }

    // Suffix wildcard: "*.md" matches "README.md"
    if let Some(prefix) = pattern.strip_suffix('*') {
        return value.starts_with(prefix);
    }

    // Prefix wildcard: "release/*" matches "release/1.0"
    if let Some(rest) = pattern.strip_suffix("/*") {
        if let Some(after) = value.strip_prefix(rest) {
            return after.starts_with('/') && !after[1..].contains('/');
        }
        return false;
    }

    // "**" matches any path
    if pattern == "**" {
        return true;
    }

    false
}

/// Check if a file path matches any of the given patterns.
fn paths_match(patterns: &[String], file: &str) -> bool {
    patterns.iter().any(|p| pattern_matches(p, file))
}

/// Simple cron validation (not full cron parser, just sanity check).
pub fn validate_cron(expr: &str) -> bool {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    // Standard cron: min hour dom month dow
    if parts.len() != 5 {
        return false;
    }
    // Validate each field is either *, a number, a range, a comma-list, or slash-separated
    parts.iter().all(|field| {
        field.split(',').all(|part| {
            part.split('/').all(|sub| {
                sub == "*" || sub.parse::<i32>().is_ok() || sub.parse::<u32>().is_ok()
            })
        })
    })
}

/// Simple glob matching for trigger expressions (e.g., "refs/heads/main").
pub fn glob_match(pattern: &str, value: &str) -> bool {
    pattern_matches(pattern, value)
}
