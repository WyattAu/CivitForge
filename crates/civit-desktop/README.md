# CivitForge Desktop

Tauri-based desktop application wrapping the CivitForge web UI.

## Build Requirements

- Rust 1.88+
- Platform-specific system dependencies (see Tauri docs)
- Tauri CLI: `cargo install tauri-cli`

## Build

```bash
# Start backend server first
cargo run --package civit-core &

# Build and run desktop app
cargo tauri dev --package civit-desktop
```

## Production Build

```bash
cargo tauri build --package civit-desktop
```

## Features

- Native window with system tray integration
- Offline local repo browsing via gix
- Native git clone/pull/push operations
- System notifications
- Deep linking (civitforge:// protocol)
