# ADR-0001: Theme system uses `.dark` class strategy, not `prefers-color-scheme`

- Status: accepted
- Date: 2026-09-06
- Deciders: Wyatt

## Context

Tailwind v4 defaults `dark:` variants to the `prefers-color-scheme` media query.
This made runtime theme toggling impossible: toggling the `dark` class on `<html>`
had zero effect on CSS. Additionally, Leptos `on:click` events are swallowed by
WebKitGTK (both `<div>` and `<button>`), and the theme state had two sources of
truth (localStorage + DOM class list, no Leptos signal).

## Decision

1. Force class-based dark mode via `@custom-variant dark (&:is(.dark *));` in `input.css`.
2. Theme toggle dispatches through `js_sys::eval("window.toggleTheme()")`, calling a
   plain JS function in `index.html` — bypassing Leptos event dispatch entirely.
3. Single source of truth: `ThemeContext` (Leptos `StoredValue<Theme>`) provided by
   `App`; `index.html` inline script applies the class **before** WASM mounts (Layer 0)
   to prevent FOUC; `Theme::persist()` writes `civit-theme` localStorage key.

## Consequences

- Theme toggling works on WebKitGTK/NVIDIA Wayland and every browser.
- Label state reads the DOM (one acceptable DOM observation); actual state flows:
  signal → `Theme::toggle_and_persist` → DOM + localStorage, atomically.
- `prefers-color-scheme` is honored only as the *fallback* when no stored preference exists.
