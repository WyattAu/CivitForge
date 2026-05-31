# Changelog

All notable changes to the CivitForge project are documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Changed
- Updated MSRV from Rust 1.85 to 1.88 (transitive dependency requirements from `home` 0.5 and `time` 0.3)
- Added `rust-toolchain.toml` for reproducible builds across all environments
- Pinned CI pipeline to Rust 1.88 (was `rust:latest` container, non-deterministic)
- Applied `clippy::uninlined_format_args` fixes across civit-core, civit-crypto, civit-runner (Rust 1.88 lint change)
- Refactored `DAGSync::dfs_cycle` from instance method to associated function (`Self::dfs_cycle`) to resolve `only_used_in_recursion` lint
- Updated VERSION.md artifact counts: 115 source files, 27,362 LOC, 640 unit tests (was 44 files, 6,282 LOC, 184 tests)

### Fixed
- CI workflow: removed redundant `rust:latest` container in favor of `dtolnay/rust-toolchain` with pinned version
- Release workflow: now uploads all 5 crate binaries (was only `civit-core`)
- Landing page: replaced undefined CSS class `blob-alt-morph` with `blob-morph-alt`
- Landing page: added `<main>` landmark element for accessibility
- Landing page: added `aria-label` attributes to stat values for screen readers
- Landing page: added `<meta name="description">` and `<meta name="theme-color">` for SEO
- 404 page: added `<meta name="robots" content="noindex, follow">` to prevent indexation
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
