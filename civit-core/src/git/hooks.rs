#![forbid(unsafe_code)]

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    pub accepted: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushContext {
    pub repo_path: PathBuf,
    pub old_sha: String,
    pub new_sha: String,
    pub ref_name: String,
    pub pusher: String,
}

pub trait PreReceiveHook: Send + Sync {
    fn name(&self) -> &str;
    fn run(&self, ctx: &PushContext) -> Result<HookResult>;
}

pub struct RefNameValidator;

impl PreReceiveHook for RefNameValidator {
    fn name(&self) -> &str {
        "ref-name-validator"
    }

    fn run(&self, ctx: &PushContext) -> Result<HookResult> {
        let ref_name = &ctx.ref_name;
        if ref_name.contains("..")
            || ref_name.contains('\0')
            || ref_name.contains('\\')
            || ref_name.starts_with('-')
        {
            return Ok(HookResult {
                accepted: false,
                message: format!("invalid ref name: {ref_name}"),
            });
        }
        Ok(HookResult {
            accepted: true,
            message: String::new(),
        })
    }
}

pub struct HookRunner {
    hooks: Vec<Box<dyn PreReceiveHook>>,
}

impl HookRunner {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    pub fn add_hook(&mut self, hook: Box<dyn PreReceiveHook>) {
        self.hooks.push(hook);
    }

    pub fn run_hooks(&self, ctx: &PushContext) -> Result<HookResult> {
        for hook in &self.hooks {
            let result = hook.run(ctx)?;
            if !result.accepted {
                info!(
                    hook = %hook.name(),
                    ref_name = %ctx.ref_name,
                    "hook rejected push"
                );
                return Ok(result);
            }
        }
        Ok(HookResult {
            accepted: true,
            message: "all hooks passed".into(),
        })
    }

    pub fn hook_count(&self) -> usize {
        self.hooks.len()
    }
}

impl Default for HookRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx(ref_name: &str) -> PushContext {
        PushContext {
            repo_path: PathBuf::from("/data/repos"),
            old_sha: "abc".into(),
            new_sha: "def".into(),
            ref_name: ref_name.into(),
            pusher: "alice".into(),
        }
    }

    #[test]
    fn test_ref_name_validator_accepts_normal() {
        let hook = RefNameValidator;
        let result = hook.run(&make_ctx("refs/heads/main")).unwrap();
        assert!(result.accepted);
        assert!(result.message.is_empty());
    }

    #[test]
    fn test_ref_name_validator_rejects_dotdot() {
        let hook = RefNameValidator;
        let result = hook.run(&make_ctx("refs/heads/../etc/passwd")).unwrap();
        assert!(!result.accepted);
        assert!(result.message.contains("invalid ref name"));
    }

    #[test]
    fn test_ref_name_validator_rejects_null_byte() {
        let hook = RefNameValidator;
        let ctx = PushContext {
            repo_path: PathBuf::from("/data/repos"),
            old_sha: "abc".into(),
            new_sha: "def".into(),
            ref_name: "refs/heads/main\0evil".into(),
            pusher: "alice".into(),
        };
        let result = hook.run(&ctx).unwrap();
        assert!(!result.accepted);
    }

    #[test]
    fn test_ref_name_validator_rejects_backslash() {
        let hook = RefNameValidator;
        let result = hook.run(&make_ctx("refs/heads\\evil")).unwrap();
        assert!(!result.accepted);
    }

    #[test]
    fn test_ref_name_validator_rejects_leading_dash() {
        let hook = RefNameValidator;
        let result = hook.run(&make_ctx("-refs/heads/main")).unwrap();
        assert!(!result.accepted);
    }

    #[test]
    fn test_hook_runner_all_pass() {
        let mut runner = HookRunner::new();
        runner.add_hook(Box::new(RefNameValidator));
        let result = runner.run_hooks(&make_ctx("refs/heads/main")).unwrap();
        assert!(result.accepted);
        assert_eq!(result.message, "all hooks passed");
    }

    #[test]
    fn test_hook_runner_stops_on_reject() {
        let mut runner = HookRunner::new();
        runner.add_hook(Box::new(RefNameValidator));
        let result = runner.run_hooks(&make_ctx("refs/heads/../evil")).unwrap();
        assert!(!result.accepted);
    }

    #[test]
    fn test_hook_runner_multiple_hooks_all_pass() {
        struct AlwaysAccept;
        impl PreReceiveHook for AlwaysAccept {
            fn name(&self) -> &str {
                "always-accept"
            }
            fn run(&self, _ctx: &PushContext) -> Result<HookResult> {
                Ok(HookResult {
                    accepted: true,
                    message: String::new(),
                })
            }
        }

        let mut runner = HookRunner::new();
        runner.add_hook(Box::new(AlwaysAccept));
        runner.add_hook(Box::new(RefNameValidator));
        assert_eq!(runner.hook_count(), 2);
        let result = runner.run_hooks(&make_ctx("refs/heads/main")).unwrap();
        assert!(result.accepted);
    }

    #[test]
    fn test_hook_runner_empty() {
        let runner = HookRunner::new();
        let result = runner.run_hooks(&make_ctx("refs/heads/main")).unwrap();
        assert!(result.accepted);
        assert_eq!(result.message, "all hooks passed");
    }

    #[test]
    fn test_hook_runner_default() {
        let runner = HookRunner::default();
        assert_eq!(runner.hook_count(), 0);
    }

    #[test]
    fn test_push_context_serialization() {
        let ctx = make_ctx("refs/heads/main");
        let json = serde_json::to_string(&ctx).unwrap();
        assert!(json.contains("refs/heads/main"));
        let de: PushContext = serde_json::from_str(&json).unwrap();
        assert_eq!(de.pusher, "alice");
        assert_eq!(de.old_sha, "abc");
        assert_eq!(de.new_sha, "def");
    }

    #[test]
    fn test_hook_result_serialization() {
        let r = HookResult {
            accepted: true,
            message: "ok".into(),
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("\"accepted\":true"));
        let de: HookResult = serde_json::from_str(&json).unwrap();
        assert!(de.accepted);
        assert_eq!(de.message, "ok");
    }

    #[test]
    fn test_hook_result_rejected_serialization() {
        let r = HookResult {
            accepted: false,
            message: "denied".into(),
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("\"accepted\":false"));
        let de: HookResult = serde_json::from_str(&json).unwrap();
        assert!(!de.accepted);
    }
}
