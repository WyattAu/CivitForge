#![forbid(unsafe_code)]

use civit_plugin_sdk as sdk;

#[no_mangle]
pub extern "C" fn on_load() {
    sdk::log(sdk::LogLevel::Info, "Hello World plugin loaded");
}

#[no_mangle]
pub extern "C" fn register_hooks() {
    sdk::register_hook("issue.created", sdk::HookPriority::Normal);
}

#[no_mangle]
pub extern "C" fn execute() -> Result<(), sdk::PluginError> {
    let ctx = sdk::context();
    let title = ctx.payload["title"].as_str().unwrap_or("untitled");
    let repo = &ctx.repository.name;
    sdk::log(
        sdk::LogLevel::Info,
        &format!("New issue '{}' created in {}", title, repo),
    );
    Ok(())
}

#[no_mangle]
pub extern "C" fn on_unload() {
    sdk::log(sdk::LogLevel::Info, "Hello World plugin unloaded");
}

#[cfg(test)]
mod tests {
    use super::*;
    use civit_plugin_sdk::testing::*;

    #[test]
    fn test_execute_with_issue_payload() {
        let payload = serde_json::json!({
            "title": "Test issue",
            "body": "Description here"
        });
        let ctx = MockContext::new("issue.created", payload);
        let result = execute_with_context(ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_with_empty_title() {
        let payload = serde_json::json!({});
        let ctx = MockContext::new("issue.created", payload);
        let result = execute_with_context(ctx);
        assert!(result.is_ok());
    }
}
