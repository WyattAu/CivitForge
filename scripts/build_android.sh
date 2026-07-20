#!/bin/bash
cd crates/civit-desktop
cargo tauri android init
cargo tauri android build --target aarch64
