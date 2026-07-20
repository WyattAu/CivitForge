#!/bin/bash
# Detect Wayland and force X11 backend for WebKitGTK compatibility
if [ -n "$WAYLAND_DISPLAY" ]; then
    export GDK_BACKEND=x11
fi

# Only disable DMABUF and force software GL on NVIDIA+Wayland
if grep -q nvidia /proc/modules 2>/dev/null; then
    export WEBKIT_DISABLE_DMABUF_RENDERER=1
    if [ -n "$WAYLAND_DISPLAY" ]; then
        export LIBGL_ALWAYS_SOFTWARE=1
    fi
fi

# Find backend
BACKEND_URL="${CIVIT_SERVER_URL:-http://127.0.0.1:9091}"

# Launch
exec ./crates/civit-desktop/target/release/civit-desktop --server-url "$BACKEND_URL"
