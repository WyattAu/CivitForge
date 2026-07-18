#![forbid(unsafe_code)]

use civit_plugin_sdk as sdk;

#[no_mangle]
pub extern "C" fn on_load() {
    sdk::log(sdk::LogLevel::Info, "Webhook Notifier plugin loaded");
}

#[no_mangle]
pub extern "C" fn register_hooks() {
    sdk::register_hook("issue.created", sdk::HookPriority::Normal);
    sdk::register_hook("issue.updated", sdk::HookPriority::Normal);
    sdk::register_hook("pull_request.opened", sdk::HookPriority::Normal);
    sdk::register_hook("pull_request.merged", sdk::HookPriority::Normal);
}

#[no_mangle]
pub extern "C" fn execute() -> Result<(), sdk::PluginError> {
    let ctx = sdk::context();

    let webhook_url = ctx.config["webhook_url"]
        .as_str()
        .ok_or_else(|| sdk::PluginError::InvalidPayload("missing webhook_url config".into()))?;

    let event_type = match ctx.hook.as_str() {
        "issue.created" => "Issue Created",
        "issue.updated" => "Issue Updated",
        "pull_request.opened" => "PR Opened",
        "pull_request.merged" => "PR Merged",
        other => other,
    };

    let payload = serde_json::json!({
        "event": event_type,
        "hook": ctx.hook,
        "repository": ctx.repository.name,
        "actor": ctx.actor.username,
        "data": ctx.payload,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let client = sdk::http();
    let resp = client
        .post(webhook_url)
        .header("Content-Type", "application/json")
        .header("X-CivitForge-Event", &ctx.hook)
        .header("X-CivitForge-Delivery", &uuid::Uuid::new_v4().to_string())
        .body(serde_json::to_vec(&payload)?)
        .send()
        .map_err(|e| sdk::PluginError::HttpError(e.to_string()))?;

    sdk::log(
        sdk::LogLevel::Info,
        &format!("Webhook sent for '{}' (HTTP {})", ctx.hook, resp.status),
    );

    Ok(())
}

#[no_mangle]
pub extern "C" fn on_unload() {
    sdk::log(sdk::LogLevel::Info, "Webhook Notifier plugin unloaded");
}

#[cfg(test)]
mod tests {
    use super::*;
    use civit_plugin_sdk::testing::*;

    #[test]
    fn test_issue_created_webhook() {
        let config = serde_json::json!({
            "webhook_url": "https://hooks.example.com/civitforge"
        });
        let payload = serde_json::json!({
            "title": "Bug report",
            "body": "Something is broken",
            "issue_id": "550e8400-e29b-41d4-a716-446655440000"
        });
        let ctx = MockContext::with_config("issue.created", payload, config);
        let result = execute_with_context(ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_webhook_url() {
        let payload = serde_json::json!({"title": "test"});
        let ctx = MockContext::new("issue.created", payload);
        let result = execute_with_context(ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_pr_opened_webhook() {
        let config = serde_json::json!({
            "webhook_url": "https://hooks.example.com/civitforge"
        });
        let payload = serde_json::json!({
            "pr_id": "aaa-bbb-ccc",
            "title": "New feature",
            "source_branch": "feature-x",
            "target_branch": "main"
        });
        let ctx = MockContext::with_config("pull_request.opened", payload, config);
        let result = execute_with_context(ctx);
        assert!(result.is_ok());
    }
}
