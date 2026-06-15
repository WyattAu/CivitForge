# CivitForge Functionality Audit Report

**Date:** 2026-06-15
**Version:** v2.2.0
**Auditor:** Autonomous Principal Software Engineer

---

## 1. Duplicated Functionality

### 1.1 Database Layer Duplication (Critical)

`civit-core/src/db/` and `civit-db/src/` contain near-identical copies of
models, repository, pool, and session modules. The copies have diverged:

| File | civit-core (lines) | civit-db (lines) | Drift |
|---|---|---|---|
| models.rs | 472 | 472 | Minimal (field ordering) |
| repository.rs | 2,487 | 2,465 | 392 differing lines |
| pool.rs | 344 | 339 | Minor additions in core |
| session.rs | 314 | 314 | Minor differences |

**Root cause:** civit-core was initially self-contained; civit-db was
extracted later but the original copy was never removed.

**Impact:** Bug fixes applied to one copy (e.g., auto_merge field addition)
do not propagate to the other. Migration sets have diverged (35 vs 42),
causing runtime errors when civit-core's migrations run without newer tables.

**Resolution:** Consolidate to single source (TD-001 in ROADMAP).

### 1.2 Dual LLM Abstractions (Low)

Two parallel abstractions exist for LLM inference:

- `LlmProvider` trait (`crates/civit-brain/src/llm/provider.rs`):
  Test-only `StubLlmProvider`. Never used in production code.
- `InferenceEngine` (`crates/civit-brain/src/llm/inference.rs`):
  Production HTTP client with reqwest, async streaming, health checks.

**Impact:** Minimal. The test-only trait is isolated, but it creates
conceptual confusion about which is canonical.

**Resolution:** Unify behind single trait (TD-005 in ROADMAP).

### 1.3 Migration Divergence (Critical)

civit-core and civit-db maintain separate migration directories:

- `crates/civit-core/src/db/migrations/`: 35 migrations
- `crates/civit-db/src/migrations/`: 42 migrations

The migration content overlaps but has diverged. Running civit-core's
migrations leaves the database missing tables that civit-db's migrations
would create (e.g., site_settings).

**Resolution:** Single migration source of truth (TD-006 in ROADMAP).

---

## 2. Usability Assessment

### 2.1 Sidebar Navigation Icons

The sidebar uses monospace bracket labels ([H], [R], [Q], [O], [S], [U], [X])
instead of emoji or SVG icons. This is intentional for the Spatial Materialism
design language and is consistent across all routes (verified by traversal).

**Assessment:** Functional and on-brand. SVG icons could improve visual
hierarchy in a future iteration without violating the design language.

### 2.2 Theme Toggle

Theme toggle uses text labels ("Dark"/"Light") instead of sun/moon emoji.
The toggle is wired via `data-theme-toggle` attribute with JavaScript event
listeners (avoids a Leptos WebKit auto-fire bug).

**Assessment:** Functional. The JS-based approach is a pragmatic workaround.

### 2.3 CDN Dependencies in WASM UI

The WASM UI loads from CDN:
- `@tailwindcss/browser@4` (CSS framework)
- `highlight.js@11.9` (syntax highlighting)
- `markdown-it@14` (Markdown rendering)
- `github-markdown-css@5` (Markdown styling)

**Impact:** Breaks air-gapped deployments (a core design goal). Adds
runtime overhead from browser-side CSS compilation.

**Resolution:** Compile Tailwind at build time (TD-004). Vendor highlight.js
and markdown-it as static assets.

### 2.4 GUI Capture Mechanism

The index.html includes an auto-capture mechanism (Ctrl+Shift+H for DOM
capture, Ctrl+Shift+S for screenshots, auto-capture every 3s) designed for
the Playwright/Tauri test harness. This runs in production and adds
unnecessary network requests (`/__capture__`, `/__navigate__` polling every
1s).

**Assessment:** The capture and navigate polling should be gated behind a
debug flag or feature flag, not enabled in production builds.

**Resolution:** Gate behind `#[cfg(debug_assertions)]` or a build-time
feature flag.

---

## 3. Component Inventory Assessment

### 3.1 Pages (39 total)

The UI has 39 page components covering: home, auth, repos, issues, PRs,
pipelines, wiki, boards, admin, settings, search, explore, orgs, analytics,
code review, image diff, model viewer, suggested edits, notification
preferences, webhook management, issue templates, and more.

**Assessment:** Comprehensive feature set. Some pages (analytics, model
viewer, image diff) appear to be forward-looking stubs that render placeholder
content. These are acceptable as long as they are clearly marked as
"in development" in the UI.

### 3.2 Components (17 total)

Reusable components: Avatar, Badge, Button, Card, DebugPanel, ErrorBanner,
ErrorBoundary, Footer, FormField, Input, Keyboard, Loading, Modal,
Pagination, Sidebar, Tab, Toast.

**Assessment:** Well-structured component library covering common UI patterns.
The ErrorBoundary with CatchError provides graceful degradation.

---

## 4. Summary

| Category | Findings | Severity |
|---|---|---|
| DB duplication | civit-core/src/db copies civit-db | Critical |
| Migration divergence | 35 vs 42 migrations, schemas diverged | Critical |
| CDN dependencies | Tailwind/highlight.js/markdown-it from CDN | Medium |
| GUI capture in prod | Auto-capture and polling enabled in production | Medium |
| Dual LLM abstractions | Two parallel inference interfaces | Low |
| RSA advisory | Timing side-channel (signing-only path) | Medium |
| gix advisory | Transitive via gix 0.70 | Medium |

All findings are tracked in the ROADMAP.md Technical Debt Register with
target versions for remediation.
