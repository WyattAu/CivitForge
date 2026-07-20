#!/bin/bash
# CivitForge Desktop — Native Wayland support with NVIDIA GPU detection

# NVIDIA: disable DMABUF renderer (fixes GBM allocation failures on NVIDIA+Wayland)
if grep -q nvidia /proc/modules 2>/dev/null; then
    export WEBKIT_DISABLE_DMABUF_RENDERER=1

    # On Wayland: force NVIDIA GBM backend and GLX vendor
    if [ -n "$WAYLAND_DISPLAY" ]; then
        export GBM_BACKEND=nvidia-drm
        export __GLX_VENDOR_LIBRARY_NAME=nvidia
    fi
fi

# Find backend
BACKEND_URL="${CIVIT_SERVER_URL:-http://127.0.0.1:9091}"

# Launch
exec ./crates/civit-desktop/target/release/civit-desktop --server-url "$BACKEND_URL"
