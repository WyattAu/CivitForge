# CivitForge Roadmap

Post-audit-cycle roadmap for a Rust-native forge platform. This document covers
versions v2.1.3 through v3.0.0. Timelines assume a core team of 3-5 engineers.

---

## Vision

CivitForge aims to be a self-hosted, federated forge platform that combines
the feature depth of GitLab with the simplicity of Gitea, built entirely in
safe Rust with zero unsafe code by default. The platform targets organizations
that require air-gapped operation, compliance-grade security, and fine-grained
access control without depending on proprietary cloud services.

---

## Current State (v2.1.2)

| Metric | Value |
|---|---|
| Workspace crates | 8 active + 1 standalone (desktop) |
| Unit tests | 3,047 passing |
| Clippy warnings | 0 |
| Format violations | 0 |
| `#![forbid(unsafe_code)]` | Enforced across all crates |
| API endpoints | ~85 routes |
| Database migrations | 001-027 |
| CI/CD | GitHub Actions: fmt, clippy, test, audit, build |
| Pre-commit hooks | Husky + .githooks (fmt, clippy, test) |
| WASM bundle | 2.8MB |
| E2E tests | Playwright (15 pages, 30 actions) |
| GUI tests | Playwright (12 pages, 63 actions, 0 errors) |
| Desktop smoke | Xvfb + GTK + WebKit (11 checks, 0 failures) |
| Landing page | GitHub Pages (https://wyattau.github.io/CivitForge/) |

---

## Version Timeline

| Version | Focus | Status |
|---|---|---|
| v2.1.3 | Audit cycle: code quality, CI/CD, UI/UX, docs | In Progress |
| v2.2.0 | Feature gap closure: Phase 0-1 from comparison matrix | Planned |
| v2.3.0 | Collaboration: Phase 2 from comparison matrix | Planned |
| v2.4.0 | CI/CD maturity: Phase 3 from comparison matrix | Planned |
| v2.5.0 | Organization and admin: Phase 4 from comparison matrix | Planned |
| v3.0.0 | Differentiation amplification: Phase 5-6 from comparison matrix | Planned |

---

## v2.1.3 -- Audit Cycle Completion

**Goal:** Complete the comprehensive audit cycle across all 8 phases.

### Completed Deliverables

**Phase 1: Code Quality**
- Replaced `expect()`/`unwrap()` with proper error propagation in `civit-core/src/main.rs`
- Replaced `DefaultHasher` with SHA-256 for policy checksums in `civit-crypto`
- Removed dead HMAC computation in `repo_keys.rs`
- Fixed HSM `sign`/`verify` to strip session key prefix for correct key lookup
- Fixed `fuse/remote.rs` mutex `unwrap()` to use safe patterns
- Fixed `grpc_server.rs` address parsing with logged fallback
- Consolidated duplicate type definitions within `civit-crypto`:
  - `AssetType`: expanded to 11 variants (superset of cmdb + iso27001)
  - `Criticality`: expanded to 4 variants (+Critical)
  - `RiskStatus`: expanded to 5 variants (+Mitigated, +Transferred)
  - `AuditTrail`: consolidated into single implementation with retention support
- Updated test expectations for fixed HSM operations

**Phase 2: CI/CD**
- Pinned Rust toolchain to 1.88 in `ci.yml` (was using `@stable`)
- Added security audit job with `cargo-audit` (non-blocking for warnings)
- Added `protobuf-compiler` installation for `civit-vfs` build
- Removed `|| true` from release.yml artifact collection
- CI pipeline: fmt, clippy, test, audit, build -- all passing

**Phase 3: UI/UX**
- Replaced `<a>` with `<A>` router component in sidebar for SPA navigation
- Added `aria-label` to search input and PR comment textarea
- Removed duplicated `get_input_value()` from `new_repo.rs`
- Fixed redundant double `set_filter.set()` in `issues.rs`

**Phase 4: Documentation**
- Replaced all emoji indicators with text equivalents in feature comparison docs
- Landing page deployed to GitHub Pages

**Phase 5: CI/CD Debug Loop**
- Fixed 3 CI pipeline failures (protobuf-compiler, cargo-audit toolchain, audit warnings)
- All pipelines green on main branch

### Remaining Work

- Complete stub implementations in `civit-vfs` (fetch_object, list_directory)
- Complete stub implementations in `civit-crypto` HSM operations (import_key, export_public_key)
- Add `aria-label` to remaining form inputs across UI pages
- Complete WebAuthn ES-256/RS256 verification
- Complete SAML XML-DSig canonicalization

---

## v2.2.0 -- Feature Gap Closure (Phase 0-1)

**Goal:** Close the 52 critical feature gaps identified in the comparison matrix.

### Key Deliverables

**Phase 0: Foundation**
- Complete `NOT_IMPLEMENTED` stubs (archive download, collaborator add/remove)
- Personal access tokens CRUD + scopes
- Webhooks CRUD + delivery worker + retry
- Email verification on registration
- Account lockout after N failed attempts
- Notifications backend (in-app + email)

**Phase 1: Code Browser Polish**
- Syntax highlighting via highlight.js CDN
- README rendering at repo root
- Language stats bar
- Commit history per file
- Git blame view
- Responsive / mobile-friendly layouts
- Keyboard shortcuts

### Estimated Timeline

4-6 weeks

---

## v2.3.0 -- Collaboration (Phase 2)

**Goal:** Full issue/PR workflow with collaboration features.

### Key Deliverables

- Assignees, cross-references, @mentions
- Task lists in markdown
- Issue templates, file attachments
- Squash, rebase, fast-forward merge strategies
- Draft PRs, merge message templates
- Linked issues auto-close
- Push to existing PR
- Release management
- Branch/tag protection rules

### Estimated Timeline

6-8 weeks

---

## v2.4.0 -- CI/CD Maturity (Phase 3)

**Goal:** Bring CI/CD to competitive parity with Gitea Actions.

### Key Deliverables

- Matrix builds, parallelism
- Secrets management, caches
- Environments + deployments
- Scheduled runs (cron)
- Status badges, auto-cancel redundant runs
- Runner management UI
- Pipeline DAG visualization
- GitHub Actions workflow compatibility (85%+)
- OIDC workload identity

### Estimated Timeline

6-8 weeks

---

## v2.5.0 -- Organization and Admin (Phase 4)

**Goal:** Full org/team management, LDAP, audit, import/migration.

### Key Deliverables

- Team management within orgs
- LDAP / AD auth backend
- Audit log admin view
- Org profile page
- Topics/tags for repos
- Transfer/rename/archive repo
- Moderation tools
- Import from GitHub/GitLab

### Estimated Timeline

4-6 weeks

---

## v3.0.0 -- Differentiation Amplification (Phases 5-6)

**Goal:** Leverage 22 unique advantages as the primary competitive moat.

### Key Deliverables

**Phase 5: Differentiation**
- AI PR review in UI (RAG + AST pipeline)
- AI code search (natural language to code)
- VFS FUSE mount UI
- Federation polish (cross-instance PRs, follow UX)
- SLSA provenance dashboard
- Edge caching UI
- Compliance dashboard (ISO 27001, audit trail)
- Secret scanning (AI-powered)
- Inline diff + side-by-side for PRs
- Commit graph visualization
- KaTeX + Mermaid in markdown
- Push/pull mirrors
- Git LFS 2.0

**Phase 6: Ecosystem**
- Package registries (npm, PyPI, Maven, Go, Helm)
- Static Pages (GitHub Pages equivalent)
- Kanban boards
- Web code editor (CodeMirror)
- i18n framework

### Estimated Timeline

12-16 weeks

---

## Technical Debt Tracker

| Item | Priority | Affected Crates | Target Version |
|---|---|---|---|
| VFS stub: fetch_object returns zeros | High | civit-vfs | v2.2.0 |
| VFS stub: list_directory returns empty | High | civit-vfs | v2.2.0 |
| HSM stub: import_key ignores data | Medium | civit-crypto | v2.3.0 |
| HSM stub: export_public_key returns fake | Medium | civit-crypto | v2.3.0 |
| HSM stub: generate_certificate returns synthetic DER | Medium | civit-crypto | v2.3.0 |
| WebAuthn ES-256/RS256 attestation incomplete | Medium | civit-crypto | v2.4.0 |
| SAML XML-DSig canonicalization incomplete | Medium | civit-crypto | v2.5.0 |
| Dual error systems (civit-core vs civit-shared) | Medium | civit-core, civit-shared | v2.2.0 |
| Duplicate API response types (server/client) | Medium | civit-core, civit-ui | v2.3.0 |
| FUSE kernel mount incomplete | Low | civit-vfs | v2.5.0 |
| Tauri desktop app (standalone build) | Low | civit-desktop | v3.0.0 |

---

## Contributing Guide

### How to Contribute

CivitForge accepts contributions via GitHub pull requests. See
`CONTRIBUTING.md` for setup instructions, coding standards, and the PR process.

### Priority Areas

1. **Stub implementations** -- VFS client/server stubs, HSM key import/export
2. **UI accessibility** -- Remaining form labels, keyboard navigation
3. **Test coverage** -- `civit-ui` WASM component tests, `civit-core` API integration tests
4. **Documentation** -- API endpoint docs, ADR updates, user-facing guides
5. **Error handling** -- Replace remaining `let _ =` and `unwrap_or_default()` patterns

### Community Guidelines

- All code must pass `cargo clippy --workspace -- -D warnings` and `cargo fmt --check --all`
- Every `.rs` file must start with `#![forbid(unsafe_code)]`
- Commit messages follow Conventional Commits format: `type(scope): description`
- PRs require one approval and a green CI before merge (squash merge)
- No emojis in code, commit messages, or documentation

---

*Last updated: 2026-06-09*
*Document owner: CivitForge core team*
