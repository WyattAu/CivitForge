# Changelog

All notable changes to the CivitForge project are documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Security

- `LICENSE`: replaced incorrect Apache 2.0 text with AGPL-3.0-or-later to match `Cargo.toml` and `README.md` declarations
- `civit-runner/src/podman.rs`: `run()` now fails with a descriptive error when Podman is unreachable, instead of silently returning a stub container (fail-closed)
- `civit-crypto/src/cel/mod.rs`: `matches()` CEL function now returns an error indicating the regex engine is not yet integrated, instead of always returning `true` (fail-closed)
- CI workflow: added explicit `permissions: contents: read` to enforce least-privilege (previously defaulted to write-all)

### Fixed

- `civit-brain/vectordb.rs`: moved `QdrantVectorDbAdapter` before `#[cfg(test)]` to resolve `clippy::items_after_test_module`
- `civit-brain/llm/inference.rs`: removed unused `debug` import
- `civit-core/ssh/auth.rs`: fixed DashMap deadlock in `RateLimiter::check()` -- holding a guard from `.get()` while calling `.remove()` on the same key deadlocked the shard lock; restructured to read ban status in a scoped block before removing
- `civit-core/api/mod.rs`: added `/healthz` and `/ready` endpoints to match Helm liveness/readiness probe paths
- SAML signature validation: changed from insecure stub (always `true`) to fail-closed (always `false`) until XML-DSIG implementation
- Landing page: corrected architecture section to reference ActivityPub/ForgeFed; updated AI layer tech tags (vLLM, tree-sitter, Qdrant)
- Helm chart `Chart.yaml` and `values.yaml`: synchronized version to match workspace `Cargo.toml`
- `civit-core/Cargo.toml`: corrected `sha1` dependency to use `workspace = true`
- Clippy fixes across orphan modules: `uninlined_format_args`, `let_unit_value`, `new_without_default`, `or_insert_with`, `manual_strip`, `collapsible_if`, `too_many_arguments`, `let_and_return`, `unnecessary_map_or`, `unused_mut`
- Refactored `DAGSync::dfs_cycle` from instance method to associated function to resolve `only_used_in_recursion`

### Added

- Integrated 4 orphan modules into crate lib.rs files: `civit-core/src/ssh/`, `civit-brain/src/llm/`, `civit-brain/src/rag_extended/`, `civit-brain/src/review/`
- 112 new unit tests from previously uncompiled orphan code
- Landing page: Open Graph and Twitter Card meta tags, `<link rel="canonical">`, `<noscript>` fallback, `dir="ltr"` attribute
- `#[allow(deprecated)]` annotations on test modules for deprecated legacy signature functions

### Changed

- CI workflow: `--locked` flag on `cargo clippy`, `cargo build --release`, and `cargo test`
- CI workflow: `dtolnay/rust-toolchain` from `@stable` to `@master` for pinned toolchain correctness
- CI workflow: `rust-toolchain.toml` added to cache key hash inputs
- Dockerfile: builds and copies all 5 crate binaries with `--workspace`
- Landing page: added `<main>` landmark, ARIA roles/labels, `aria-expanded`/`aria-controls` on mobile toggle, `aria-hidden` on decorative elements
- 404 page: `noindex` meta tag, canonical link, fixed `/undefined` edge case
- Pre-commit hook: `--locked` flag on `cargo test`

---

## [0.1.0] - 2026-05-30

### Added

- Project specification foundation (Phase 0: Requirements Engineering)
- Domain analysis with 7 applicable standards (ISO 27001, NIST SP 800-53, SLSA L4, FIPS 140-2, ISO 26262, OWASP Top 10, SOC2 Type II)
- 69 EARS-format requirements across 5 areas (VCS, LFS+, CI/CD, AI, Federation)
- 16 formalized non-functional requirements with measurable acceptance criteria
- Traceability matrix, 6 standard conflict resolutions, capability matrix, 10 tooling gap identifications
