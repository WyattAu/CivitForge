# Changelog

All notable changes to the CivitForge project are documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added
- Integrated 4 orphan modules into their crate lib.rs files: `civit-core/src/ssh/` (3 files), `civit-brain/src/llm/` (3 files), `civit-brain/src/rag_extended/` (3 files), `civit-brain/src/review/` (3 files)
- Added re-exports: `SshConfig`, `SshServer`, `SshAuthService` in civit-core; module declarations in civit-brain
- 112 new unit tests from previously uncompiled orphan code (total: 773 -> 885)

### Changed
- Updated VERSION.md artifact counts: 115 source files, 29,086 LOC, 885 unit tests (was 27,362 LOC, 773 tests)
- CI workflow: added `--locked` flag to `cargo clippy` and `cargo build --release` for reproducible builds
- CI workflow: changed `cargo install cargo-llvm-cov` to use `--force` to handle existing installations

### Fixed
- SAML signature validation: changed from insecure stub (always returns `true`) to fail-closed (always returns `false`) until XML-DSIG implementation
- SAML test: updated `test_is_valid_signature_stub` to `test_is_valid_signature_fail_closed` asserting rejection
- Clippy fixes in orphan modules: `uninlined_format_args`, `let_unit_value`, `new_without_default`, `or_insert_with` -> `or_default`, `manual_strip`, `collapsible_if`, `too_many_arguments`, `let_and_return`, `unnecessary_map_or`, `unused_mut`
- Review analyzer test: fixed `test_analyze_clean_diff` test data to use proper diff `+` prefix instead of space prefix
- Review rules test: fixed `test_evaluate_secret` and `test_secret_case_insensitive` test inputs to match assignment pattern detection logic

### Security
- SAML `is_valid_signature()` now returns `false` (fail-closed) instead of `true` (fail-open)
- Added `rust-toolchain.toml` for reproducible builds across all environments
- Pinned CI pipeline to Rust 1.88 (was `rust:latest` container, non-deterministic)
- Applied `clippy::uninlined_format_args` fixes across civit-core, civit-crypto, civit-runner (Rust 1.88 lint change)
- Refactored `DAGSync::dfs_cycle` from instance method to associated function (`Self::dfs_cycle`) to resolve `only_used_in_recursion` lint
- Updated VERSION.md artifact counts: 115 source files, 27,362 LOC, 773 unit tests (was 44 files, 6,282 LOC, 184 tests)
- CI workflow: switched `dtolnay/rust-toolchain` from `@stable` to `@master` for pinned toolchain correctness
- CI workflow: added `rust-toolchain.toml` to cache key hash inputs for cache invalidation on toolchain changes
- CI workflow: added `--locked` flag to `cargo test` for reproducible dependency resolution
- Release workflow: added `--locked` flag to `cargo build` for reproducible builds
- Dockerfile: now builds and copies all 5 crate binaries (was only `civit-core`)
- Dockerfile: added `--workspace` flag to build step for complete workspace compilation

### Fixed
- CI workflow: removed redundant `rust:latest` container in favor of `dtolnay/rust-toolchain` with pinned version
- Release workflow: now uploads all 5 crate binaries (was only `civit-core`)
- Landing page: replaced undefined CSS class `blob-alt-morph` with `blob-morph-alt`
- Landing page: added `<main>` landmark element for accessibility
- Landing page: added `aria-label` attributes to stat values for screen readers
- Landing page: added `<meta name="description">` and `<meta name="theme-color">` for SEO
- Landing page: added ARIA `role="menubar"` and `role="menuitem"` to navigation for screen reader support
- Landing page: added `aria-expanded` and `aria-controls` to mobile menu toggle button
- Landing page: added `aria-hidden="true"` to decorative background shapes and logo text
- 404 page: added `<meta name="robots" content="noindex, follow">` to prevent indexation
- 404 page: fixed JavaScript edge case where root path displayed `/undefined` instead of `/unknown`
- Pre-commit hook: added `--locked` flag to `cargo test` invocation
- Doctest: converted `ignore` code block to `text` in websocket.rs to eliminate ignored test count
- Roadmap: corrected artifact counts to match actual codebase state

### Security
- README badge updated from Rust 1.78+ to 1.88+ to reflect actual toolchain requirement
- CONTRIBUTING.md updated to specify Rust 1.88+

---

## [0.1.0] - 2026-05-30

### Added
- Established project specification foundation (Phase 0: Requirements Engineering)
- Created domain analysis with 7 applicable standards mapped (ISO 27001, NIST SP 800-53, SLSA L4, FIPS 140-2, ISO 26262, OWASP Top 10, SOC2 Type II)
- Converted 21 PRD functional requirements into 69 EARS-format requirements across 5 areas (VCS, LFS+, CI/CD, AI, Federation)
- Added 16 formalized non-functional requirements with measurable acceptance criteria
- Created traceability matrix mapping all 69 requirements to components, test cases, standards, and roadmap phases
- Identified 6 standard conflicts with proposed resolution strategies
- Generated capability matrix mapping available tooling against required stack
- Identified 10 tooling gaps requiring procurement or installation before development

### Project Structure
- `.specs/00_requirements/` -- Requirements engineering artifacts
- `.reports/` -- Phase reports and analysis documents
