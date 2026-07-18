# Hello World Plugin

Minimal CivitForge plugin example. Logs a message when an issue is created.

## Build

```bash
rustup target add wasm32-wasip1
cargo install cargo-component
cargo component build --release
```

## Install

```bash
cp target/wasm32-wasip1/release/hello_world_plugin.wasm ~/.civitforge/plugins/
```

## Verify

```bash
civitforge plugin list
# Should show: hello_world_plugin (v0.1.0)
```

## Test

```bash
cargo test
```
