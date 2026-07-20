#!/bin/bash
# Fix Wayland/WebKitGTK GBM buffer issue
export WEBKIT_DISABLE_COMPOSITING_MODE=1
export GDK_RENDERING=image
export LIBGL_ALWAYS_SOFTWARE=1

# Find backend
BACKEND_URL="${CIVIT_SERVER_URL:-http://127.0.0.1:9091}"

# Launch
exec ./crates/civit-desktop/target/release/civit-desktop --server-url "$BACKEND_URL"
