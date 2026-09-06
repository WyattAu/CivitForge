# ADR-0003: Desktop app defaults to X11/XWayland; native Wayland is opt-in

- Status: accepted
- Date: 2026-09-06
- Deciders: Wyatt

## Context

WebKitGTK 2.52.5 + NVIDIA (RTX 2060, driver 610.43.03) on KDE Wayland exhibits
upstream bugs: GBM buffer allocation failures and Wayland protocol errors
(tauri-apps/wry#1366, tauri-apps/tauri#9394, #10702). Native Wayland rendering
works intermittently; the X11 backend through XWayland is stable (verified 13+
minute sessions, full UI interaction).

## Decision

- `civit-desktop` sets, before any thread spawns, on NVIDIA + Wayland:
  `WEBKIT_DISABLE_DMABUF_RENDERER=1`, `GBM_BACKEND=nvidia-drm`,
  `__GLX_VENDOR_LIBRARY_NAME=nvidia`, **and** `GDK_BACKEND=x11` as the default
  rendering path.
- No `LIBGL_ALWAYS_SOFTWARE=1` (degrades quality without fixing stability).
- Revisit native Wayland when upstream WebKitGTK fixes land.

## Consequences

- Window management flows through XWayland (xdotool/scrot can see it — a bonus
  for automation/E2E).
- Native Wayland purists lose fractional-scaling fidelity until upstream fixes;
  tracked, not fought.
