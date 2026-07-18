#![forbid(unsafe_code)]

use civit_plugin_sdk as sdk;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct CustomFieldConfig {
    field_name: String,
    field_type: String,
    required: bool,
    default_value: Option<String>,
    allowed_values: Option<Vec<String>>,
}

impl CustomFieldConfig {
    fn from_ctx(ctx: &sdk::PluginContext) -> Result<Self, sdk::PluginError> {
        serde_json::from_value(ctx.config.clone())
            .map_err(|e| sdk::PluginError::InvalidPayload(format!("invalid config: {}", e)))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CustomField {
    name: String,
    field_type: String,
    value: serde_json::Value,
}

fn validate_field_value(field: &CustomFieldConfig, value: &serde_json::Value) -> bool {
    match field.field_type.as_str() {
        "select" => {
            if let Some(allowed) = &field.allowed_values {
                allowed.contains(&value.as_str().unwrap_or("").to_string())
            } else {
                true
            }
        }
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "text" => value.is_string(),
        _ => true,
    }
}

#[no_mangle]
pub extern "C" fn on_load() {
    sdk::log(sdk::LogLevel::Info, "Custom Field plugin loaded");
}

#[no_mangle]
pub extern "C" fn register_hooks() {
    sdk::register_hook("issue.created", sdk::HookPriority::Normal);
    sdk::register_hook("issue.updated", sdk::HookPriority::Normal);
}

#[no_mangle]
pub extern "C" fn execute() -> Result<(), sdk::PluginError> {
    let ctx = sdk::context();
    let config = CustomFieldConfig::from_ctx(&ctx)?;

    let custom_fields = ctx.payload["custom_fields"]
        .as_object()
        .ok_or_else(|| {
            sdk::PluginError::InvalidPayload("missing custom_fields in payload".into())
        })?;

    let field_value = custom_fields.get(&config.field_name).ok_or_else(|| {
        sdk::PluginError::InvalidPayload(format!("missing field: {}", config.field_name))
    })?;

    if config.required && field_value.is_null() {
        return Err(sdk::PluginError::InvalidPayload(format!(
            "field '{}' is required",
            config.field_name
        )));
    }

    if !field_value.is_null() && !validate_field_value(&config, field_value) {
        return Err(sdk::PluginError::InvalidPayload(format!(
            "invalid value for field '{}': {:?}",
            config.field_name, field_value
        )));
    }

    let field = CustomField {
        name: config.field_name.clone(),
        field_type: config.field_type.clone(),
        value: field_value.clone(),
    };

    let storage = sdk::storage();
    let issue_id = ctx.payload["issue_id"]
        .as_str()
        .unwrap_or("unknown");
    let storage_key = format!("custom_field:{}:{}", issue_id, config.field_name);
    storage.set(&storage_key, &serde_json::to_string(&field)?)?;

    sdk::log(
        sdk::LogLevel::Info,
        &format!(
            "Custom field '{}' set on issue {}",
            config.field_name, issue_id
        ),
    );

    Ok(())
}

#[no_mangle]
pub extern "C" fn on_unload() {
    sdk::log(sdk::LogLevel::Info, "Custom Field plugin unloaded");
}

#[cfg(test)]
mod tests {
    use super::*;
    use civit_plugin_sdk::testing::*;

    fn select_config() -> serde_json::Value {
        serde_json::json!({
            "field_name": "priority",
            "field_type": "select",
            "required": true,
            "allowed_values": ["low", "medium", "high", "critical"]
        })
    }

    #[test]
    fn test_valid_select_value() {
        let payload = serde_json::json!({
            "issue_id": "test-123",
            "custom_fields": { "priority": "high" }
        });
        let ctx = MockContext::with_config("issue.created", payload, select_config());
        assert!(execute_with_context(ctx).is_ok());
    }

    #[test]
    fn test_invalid_select_value() {
        let payload = serde_json::json!({
            "issue_id": "test-123",
            "custom_fields": { "priority": "urgent" }
        });
        let ctx = MockContext::with_config("issue.created", payload, select_config());
        assert!(execute_with_context(ctx).is_err());
    }

    #[test]
    fn test_missing_required_field() {
        let payload = serde_json::json!({
            "issue_id": "test-123",
            "custom_fields": {}
        });
        let ctx = MockContext::with_config("issue.created", payload, select_config());
        assert!(execute_with_context(ctx).is_err());
    }

    #[test]
    fn test_number_field_type() {
        let config = serde_json::json!({
            "field_name": "estimate",
            "field_type": "number",
            "required": false
        });
        let payload = serde_json::json!({
            "issue_id": "test-123",
            "custom_fields": { "estimate": 5 }
        });
        let ctx = MockContext::with_config("issue.created", payload, config);
        assert!(execute_with_context(ctx).is_ok());
    }

    #[test]
    fn test_boolean_field_type() {
        let config = serde_json::json!({
            "field_name": "needs_review",
            "field_type": "boolean",
            "required": true
        });
        let payload = serde_json::json!({
            "issue_id": "test-123",
            "custom_fields": { "needs_review": true }
        });
        let ctx = MockContext::with_config("issue.created", payload, config);
        assert!(execute_with_context(ctx).is_ok());
    }
}
