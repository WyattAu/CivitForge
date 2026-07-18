# CivitForge Plugin SDK

Extend CivitForge with custom functionality through the WASM-based plugin system.

## Getting Started

### Prerequisites

- Rust 1.96+ with `wasm32-wasip1` target
- `cargo-component` for component model builds
- CivitForge v3.2.0+

### Install the target

```bash
rustup target add wasm32-wasip1
cargo install cargo-component
```

### Create your first plugin

```bash
mkdir my-plugin && cd my-plugin
cargo component new my-plugin
```

Edit `Cargo.toml`:

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2024"

[dependencies]
civit-plugin-sdk = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[lib]
crate-type = ["cdylib"]
```

Build and install:

```bash
cargo component build --release
cp target/wasm32-wasip1/release/my_plugin.wasm ~/.civitforge/plugins/
```

---

## Plugin Architecture

```
┌─────────────────────────────────────────────────┐
│                  CivitForge Core                 │
│                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐      │
│  │  Issues   │  │ Pipelines│  │   Git    │      │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘      │
│       │              │              │            │
│  ┌────▼──────────────▼──────────────▼────┐      │
│  │           Plugin Host (WASM)          │      │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ │      │
│  │  │ Plugin  │ │ Plugin  │ │ Plugin  │ │      │
│  │  │   A     │ │   B     │ │   C     │ │      │
│  │  └─────────┘ └─────────┘ └─────────┘ │      │
│  └───────────────────────────────────────┘      │
└─────────────────────────────────────────────────┘
```

Plugins run in sandboxed WASM instances. The host provides a stable ABI for:

- **Hook registration** -- declare which events your plugin handles
- **Context API** -- read/write issue data, repository metadata, user info
- **HTTP client** -- make outbound HTTP requests (webhooks, external APIs)
- **Storage** -- persistent key-value store per plugin instance
- **Logging** -- structured logging to CivitForge log output

### Lifecycle

1. **Load** -- WASM module loaded, `on_load()` called
2. **Register** -- plugin registers hooks via `register_hooks()`
3. **Execute** -- hooks fire when events match
4. **Unload** -- `on_unload()` called, resources freed

---

## Hook System

Hooks are named events that plugins subscribe to. When the event fires, all registered plugins execute in dependency order.

### Available Hooks

| Hook | Trigger | Payload |
|------|---------|---------|
| `issue.created` | New issue created | `IssuePayload` |
| `issue.updated` | Issue edited | `IssuePayload` |
| `issue.closed` | Issue closed | `IssuePayload` |
| `pull_request.opened` | PR opened | `PullRequestPayload` |
| `pull_request.merged` | PR merged | `PullRequestPayload` |
| `pipeline.started` | CI pipeline starts | `PipelinePayload` |
| `pipeline.completed` | CI pipeline finishes | `PipelinePayload` |
| `pipeline.failed` | CI pipeline fails | `PipelinePayload` |
| `push` | Git push received | `PushPayload` |
| `release.published` | Release created | `ReleasePayload` |
| `repository.created` | Repo created | `RepositoryPayload` |
| `user.registered` | New user signup | `UserPayload` |
| `cron` | Scheduled interval | `CronPayload` |

### Hook Priority

Plugins can specify execution priority (lower = earlier):

```rust
#[no_mangle]
pub extern "C" fn register_hooks() {
    sdk::register_hook("issue.created", HookPriority::Normal);
    sdk::register_hook("pipeline.completed", HookPriority::High);
}
```

Priorities: `Critical(0)` > `High(100)` > `Normal(200)` > `Low(300)` > `Background(400)`

### Hook Execution Model

- Hooks run asynchronously in a thread pool
- A failing hook does not block other plugins
- Hooks have a 30-second timeout by default
- Hooks can be retried (configurable per plugin)

---

## API Reference

### Core Functions

#### `sdk::register_hook(hook_name, priority)`

Register the plugin to handle a named hook.

```rust
sdk::register_hook("issue.created", sdk::HookPriority::Normal);
```

#### `sdk::context() -> PluginContext`

Access the current execution context.

```rust
let ctx = sdk::context();
let issue_id = ctx.payload.get("issue_id").unwrap();
let repo = ctx.repository.name;
let user = ctx.actor.username;
```

#### `sdk::storage() -> Storage`

Persistent key-value storage scoped to the plugin.

```rust
let storage = sdk::storage();
storage.set("last_run", &chrono::Utc::now().to_rfc3339())?;
let last_run: Option<String> = storage.get("last_run")?;
```

#### `sdk::http() -> HttpClient`

Outbound HTTP client with TLS support.

```rust
let client = sdk::http();
let resp = client.post("https://hooks.slack.com/...")
    .header("Content-Type", "application/json")
    .body(serde_json::to_vec(&payload)?)
    .send()?;
```

#### `sdk::log(level, message)`

Structured logging.

```rust
sdk::log(sdk::LogLevel::Info, "Plugin executed successfully");
sdk::log(sdk::LogLevel::Error, &format!("Failed: {}", err));
```

### Data Types

#### `PluginContext`

```rust
pub struct PluginContext {
    pub hook: String,
    pub payload: serde_json::Value,
    pub repository: RepositoryInfo,
    pub actor: ActorInfo,
    pub config: serde_json::Value,
}
```

#### `RepositoryInfo`

```rust
pub struct RepositoryInfo {
    pub id: Uuid,
    pub name: String,
    pub full_name: String,
    pub owner: ActorInfo,
    pub default_branch: String,
    pub is_private: bool,
}
```

#### `ActorInfo`

```rust
pub struct ActorInfo {
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
}
```

### Error Handling

Plugins return `Result<(), PluginError>`. Errors are logged but do not crash the host.

```rust
pub enum PluginError {
    InvalidPayload(String),
    HttpError(String),
    StorageError(String),
    SerializationError(String),
    Custom(String),
}
```

---

## Example: Hello World Plugin

The simplest possible plugin. Logs a message on issue creation.

See `examples/plugins/hello_world/` for the full source.

```rust
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
    sdk::log(sdk::LogLevel::Info, &format!("New issue: {}", title));
    Ok(())
}

#[no_mangle]
pub extern "C" fn on_unload() {
    sdk::log(sdk::LogLevel::Info, "Hello World plugin unloaded");
}
```

---

## Example: Webhook Notifier Plugin

Sends an HTTP webhook when issues are created or updated.

See `examples/plugins/webhook_notifier/` for the full source.

```rust
use civit_plugin_sdk as sdk;

#[no_mangle]
pub extern "C" fn register_hooks() {
    sdk::register_hook("issue.created", sdk::HookPriority::Normal);
    sdk::register_hook("issue.updated", sdk::HookPriority::Normal);
}

#[no_mangle]
pub extern "C" fn execute() -> Result<(), sdk::PluginError> {
    let ctx = sdk::context();
    let webhook_url = ctx.config["webhook_url"]
        .as_str()
        .ok_or(sdk::PluginError::InvalidPayload("missing webhook_url".into()))?;

    let client = sdk::http();
    let resp = client.post(webhook_url)
        .header("Content-Type", "application/json")
        .header("X-CivitForge-Event", &ctx.hook)
        .body(serde_json::to_vec(&ctx.payload)?)
        .send()?;

    sdk::log(sdk::LogLevel::Info, &format!("Webhook sent: {}", resp.status));
    Ok(())
}
```

---

## Testing Plugins

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use civit_plugin_sdk::testing::*;

    #[test]
    fn test_execute_with_issue_payload() {
        let payload = serde_json::json!({
            "issue_id": "550e8400-e29b-41d4-a716-446655440000",
            "title": "Test issue",
            "body": "Description here"
        });

        let ctx = MockContext::new("issue.created", payload);
        let result = execute_with_context(ctx);
        assert!(result.is_ok());
    }
}
```

### Integration Testing

```bash
# Run plugin tests against a live CivitForge instance
CIVITFORGE_URL=http://localhost:9091 cargo test --features integration
```

### Validation

```bash
# Validate plugin WASM binary
civitforge plugin validate target/wasm32-wasip1/release/my_plugin.wasm
```

---

## Publishing to Marketplace

### 1. Create a `plugin.toml`

```toml
[plugin]
name = "my-plugin"
version = "0.1.0"
description = "Does something useful"
author = "your-username"
license = "AGPL-3.0-or-later"
repository = "https://github.com/you/my-plugin"

[hooks]
"issue.created" = "normal"
"pull_request.opened" = "high"

[config]
webhook_url = { type = "string", required = true, description = "Webhook URL" }
```

### 2. Build the release binary

```bash
cargo component build --release
```

### 3. Package and publish

```bash
civitforge plugin package \
  --wasm target/wasm32-wasip1/release/my_plugin.wasm \
  --config plugin.toml \
  --output my-plugin-0.1.0.tar.gz

civitforge plugin publish my-plugin-0.1.0.tar.gz
```

### Review Process

- Automated scan for unsafe WASM patterns
- Manifest validation (hooks, config, permissions)
- Manual review for first-time publishers
- Approval within 48 hours

---

## Troubleshooting

### Plugin fails to load

```
Error: WASM instantiation failed
```

- Ensure you compiled with `wasm32-wasip1` target
- Check that `civit-plugin-sdk` version matches the host
- Run `wasm-tools validate target/wasm32-wasip1/release/my_plugin.wasm`

### Hook never fires

- Verify the hook name matches exactly (case-sensitive)
- Check plugin logs: `civitforge plugin logs my-plugin`
- Ensure the hook is registered in `register_hooks()`

### HTTP requests fail

- WASM plugins use a restricted HTTP client
- Only HTTPS endpoints are allowed
- Check firewall rules and DNS resolution
- Timeout is 10 seconds by default

### Storage quota exceeded

Each plugin gets 1MB of key-value storage. Check usage:

```bash
civitforge plugin storage-usage my-plugin
```

### Debugging

Enable verbose plugin logging:

```bash
RUST_LOG=civit_plugin_sdk=debug civitforge server
```

Step-through debugging with `wasm-tools`:

```bash
wasm-tools print target/wasm32-wasip1/release/my_plugin.wasm | less
```
