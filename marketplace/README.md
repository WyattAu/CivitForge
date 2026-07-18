# CivitForge Marketplace

Community-contributed plugins and CI actions for CivitForge.

## Plugins

Plugins extend CivitForge with custom functionality. They run as sandboxed WASM modules.

| Plugin | Description | Hooks |
|--------|-------------|-------|
| [Hello World](plugins/hello_world.json) | Minimal example plugin | `issue.created` |
| [Webhook Notifier](plugins/webhook_notifier.json) | Send webhooks on events | `issue.*`, `pull_request.*` |
| [Custom Field](plugins/custom_field.json) | Add custom fields to issues | `issue.*` |

## CI Actions

Pre-built CI action definitions for common workflows.

| Action | Description |
|--------|-------------|
| [Lint Code](actions/lint_code.json) | Multi-language linting |
| [Run Tests](actions/run_tests.json) | Execute test suites |
| [Deploy Staging](actions/deploy_staging.json) | Deploy to staging environment |

## Publishing

See [Plugin SDK Documentation](../docs/PLUGIN_SDK.md#publishing-to-marketplace) for instructions on publishing your own plugins.

## Standards

All marketplace entries must:

- Pass automated security scanning
- Include a README with usage instructions
- Have a valid `plugin.toml` manifest
- Use `#![forbid(unsafe_code)]` in Rust plugins
- Not require network access unless explicitly declared
