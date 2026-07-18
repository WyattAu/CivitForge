# Webhook Notifier Plugin

Sends HTTP webhooks to external services on issue and pull request events.

## Configuration

Set the webhook URL in your CivitForge plugin config (`~/.civitforge/plugins.json`):

```json
{
  "webhook_notifier": {
    "webhook_url": "https://hooks.slack.com/services/T00/B00/xxxx"
  }
}
```

### Supported Events

| Hook | Description |
|------|-------------|
| `issue.created` | New issue opened |
| `issue.updated` | Issue edited |
| `pull_request.opened` | PR created |
| `pull_request.merged` | PR merged |

### Payload Format

```json
{
  "event": "Issue Created",
  "hook": "issue.created",
  "repository": "my-repo",
  "actor": "username",
  "data": { "title": "...", "body": "..." },
  "timestamp": "2026-07-18T12:00:00Z"
}
```

### Headers

- `Content-Type: application/json`
- `X-CivitForge-Event: <hook-name>`
- `X-CivitForge-Delivery: <uuid>`

## Build

```bash
cargo component build --release
```

## Install

```bash
cp target/wasm32-wasip1/release/webhook_notifier_plugin.wasm ~/.civitforge/plugins/
```

## Test

```bash
cargo test
```
