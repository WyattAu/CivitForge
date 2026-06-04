# CivitForge Roadmap

Post-v1.1.0 release plan for a Rust-native forge platform. This document covers
versions v1.1.1 through v2.0.0. Timelines assume a core team of 3-5 engineers.

---

## Vision

CivitForge aims to be a self-hosted, federated forge platform that combines
the feature depth of GitLab with the simplicity of Gitea, built entirely in
safe Rust with zero unsafe code by default. The platform targets organizations
that require air-gapped operation, compliance-grade security, and fine-grained
access control without depending on proprietary cloud services.

---

## Version Timeline

| Version | Focus | Status |
|---|---|---|
| v1.1.1 | Hotfix: accessibility and error hygiene | Shipped |
| v1.2.0 | Quality and completeness | Shipped |
| v1.3.0 | Advanced features | Shipped |
| v1.4.0 | Federation and enterprise | Shipped |
| v1.5.0 | Horizontal scaling | Shipped |
| v2.0.0 | Platform expansion | Shipped |
| v2.0.0 | Platform expansion | Planned |

---

## v1.1.1 -- Hotfix: Accessibility and Error Hygiene

**Goal:** Ship a focused patch that addresses the 3 critical accessibility
failures and stops raw API error bodies from leaking to end users.

### Key Deliverables

- Add skip-navigation link to `civit-ui/src/app.rs` (Leptos Router shell)
- Implement modal focus trap in `civit-ui/src/components/modal.rs`
  (`FocusTrap` via `web-sys` `Element::focus()` and `keydown` listener)
- Add `aria-label` attributes to `civit-ui/src/components/toast.rs`
  notifications (success, error, warning, info variants)
- Sanitize error responses in `civit-core/src/api/mod.rs` -- intercept
  `axum::extract::rejection` and `sqlx::Error` bodies, return structured
  `civit_shared::ErrorResponse` instead of raw text
- Replace 24 error-swallowing `let _ =` / `unwrap_or_default()` sites across
  `civit-core/src/api/*.rs` with proper error propagation
- Update `civit-ui/src/pages/login.rs` and `civit-ui/src/pages/settings.rs`
  to display user-facing error messages from the new structured responses

### Success Metrics

- Zero critical WCAG 2.1 AA violations (automated via axe-core audit)
- No raw SQL/HTTP error text visible in the browser network tab on failure
- All existing 2,653 tests continue to pass

### Estimated Timeline

1-2 weeks after v1.1.0 tag

---

## v1.2.0 -- Quality and Completeness

**Goal:** Resolve all audit findings, wire up every existing API endpoint to a
working frontend, and raise test coverage to 85%+ across all workspace crates.

### Key Deliverables

**UI Completion (all pages backed by live API data)**

- Code browser page (`civit-ui/src/pages/repo_detail.rs`): integrate with
  `civit-vfs` gRPC layer to render file trees, file content, and directory
  listings from bare git repos via `civit-core/src/git/operations.rs`
- Pipelines dashboard page: new `civit-ui/src/pages/pipelines.rs` consuming
  `GET /api/v1/pipelines` and `GET /api/v1/pipelines/{id}` from
  `civit-core/src/api/pipelines.rs` -- list runs, step status, artifact links
- Global search page: new `civit-ui/src/pages/search.rs` wired to
  `GET /api/v1/search` in `civit-core/src/api/search.rs` with repo filter,
  language filter, and ranked result display
- Activity feed page: new `civit-ui/src/pages/activity.rs` consuming events
  from `civit-core/src/events/websocket.rs` (event bus) rendered as a
  chronological timeline
- Settings page forms functional: `civit-ui/src/pages/settings.rs` -- wire
  profile update, SSH key management (`api/ssh_keys.rs`), and 2FA toggle to
  their respective `civit-core` API handlers
- Org detail page: fix raw UUID display in `civit-ui/src/pages/orgs.rs`,
  resolve org slug from `civit-shared::OrgResponse`
- Repos page: replace hardcoded mock data in `civit-ui/src/pages/repos.rs`
  with `GET /api/v1/repos` from `civit-core/src/api/repos.rs`

**Code Quality**

- Remove all 66 `#[allow(dead_code)]` annotations across workspace crates
- Delete or justify all 2,202 commented-out lines (use `git log --blame` to
  determine if code was once shipped; if not, remove)
- Replace 43 stub patterns (`todo!()`, `unimplemented!()`, empty match arms,
  `String::new()` returns in error paths) with real implementations or explicit
  `unimplemented!("reason")` with tracking issues
- Add form validation to all `civit-ui` input components
  (`civit-ui/src/components/input.rs`): required-field checks, email format,
  password strength, repo slug format, duplicate detection
- Add integration tests for all 60+ API routes in `civit-core/src/api/*.rs`

**Test Coverage**

- `civit-ui` WASM component tests (wasm-bindgen-test) targeting >60% line coverage
- `civit-core` API handler integration tests targeting >90% coverage
- Overall workspace coverage target: 85%+

**Technical Debt**

- Add UUID-to-slug conversion utilities in `civit-shared/src/id.rs` so UI
  never shows raw UUIDs for repos, orgs, users, or issues
- Extract shared validation logic from `civit-pipeline/src/validate.rs` into
  `civit-shared` for reuse in UI client-side validation

### Success Metrics

- Every API endpoint has a corresponding UI page or is explicitly documented
  as API-only (e.g., internal runner protocol)
- `cargo test --workspace` passes with 3,200+ tests
- Code coverage >= 85% (measured via `cargo-llvm-cov`)
- Zero `#[allow(dead_code)]` annotations
- Zero stub patterns without linked tracking issues

### Estimated Timeline

6-8 weeks after v1.1.1

---

## v1.3.0 -- Advanced Features

**Goal:** Add collaboration features that bring CivitForge to feature parity
with GitLab/GitHub for day-to-day development workflows.

### Key Deliverables

**Project Boards / Kanban**

- DB schema: `boards`, `board_columns`, `board_cards` (migration 025+)
- CRUD API in `civit-core/src/api/` -- create board per repo, drag-and-drop
  columns, link cards to issues
- Leptos board UI: column layout, card drag (via `web-sys` drag events),
  issue linking

**Merge Queue**

- Wire existing `civit-core/src/merge_queue.rs` to a real sequential merge
  pipeline: PRs enter queue, CI runs, merge on green
- API endpoints: `POST /api/v1/repos/{id}/merge-queue`, status polling
- UI: queue list page, per-PR merge queue status badge

**Code Review Inline Comments**

- DB schema: `review_comments` with `path`, `line`, `diff_hunk`, `body`
  (migration 026+)
- API: `POST /api/v1/pulls/{id}/comments`, list, update, resolve
- Diff viewer enhancement in `civit-ui`: click-to-comment on diff lines

**Real-Time WebSocket Log Streaming**

- Integrate `civit-core/src/events/websocket.rs` with
  `civit-runner/src/main.rs` log output -- runner sends log chunks via
  WebSocket channel, browser renders in real time
- Pipeline run detail page: live log panel with auto-scroll, step transition

**WebAuthn Completion**

- Complete ES-256 and RS256 signature verification in
  `civit-crypto/src/hsm/operations.rs` (currently partial)
- Wire to `civit-core/src/api/auth.rs` registration and authentication flows
- UI: WebAuthn key management in settings page

**Tantivy Code Search Upgrade**

- Replace PostgreSQL `tsvector`/`tsquery` in `civit-core/src/api/search.rs`
  with tantivy full-text index (trigram tokenizer, per-repo indexes)
- Index on push: hook into `civit-core/src/git/hooks.rs` post-receive to
  incrementally update the tantivy index
- Cross-repo search with permission filtering via
  `civit-core/src/auth/permission_engine.rs`

### Success Metrics

- Kanban board usable end-to-end (create, drag, link issues)
- Merge queue accepts PRs, runs CI, merges on green without race conditions
- Inline code comments render on diff view with resolve/unresolve toggle
- Pipeline logs stream at <200ms latency via WebSocket
- Code search returns ranked results with snippet highlighting
- WebAuthn registration and authentication work with YubiKey 5 series

### Estimated Timeline

8-12 weeks after v1.2.0

---

## v1.4.0 -- Federation and Enterprise

**Goal:** Deliver production-grade federation via ActivityPub and enterprise
SSO so CivitForge instances can interoperate and integrate with existing
identity providers.

### Key Deliverables

**ActivityPub Federation**

- Complete `civit-core/src/federation/delivery.rs` with 2-instance integration
  tests (repo fork federation, issue cross-posting, star/like activities)
- Implement inbox/outbox in `civit-core/src/federation/inbox_outbox.rs`
  with persistent storage (DB-backed, not in-memory VecDeque)
- WebFinger discovery via `civit-core/src/federation/webfinger.rs`
- Federation documentation and test instance deployment guide

**Multi-Tenancy**

- Per-organization data isolation in `civit-core/src/db/repository.rs`
  (row-level security or application-level filtering)
- Org-scoped runner pools and CI variable namespaces
- Resource quotas per org (storage, runners, pipeline minutes)

**Per-Repo Encryption Keys**

- AES-256-GCM key derivation per repository in `civit-crypto/` using
  `ring::aead`, extending existing `civit-core/src/secrets.rs` pattern
- Key rotation API with zero-downtime re-encryption
- Pipeline variables and webhook secrets encrypted with repo-scoped keys

**SAML SSO**

- Complete XML-DSig canonicalization and signature verification in
  `civit-crypto/src/` (currently SHA-256 digest integrity only)
- IdP metadata parsing, SP metadata generation
- SSO login flow wired to `civit-core/src/api/auth.rs`

### Success Metrics

- Two CivitForge instances successfully federate a repository fork
- SAML login completes end-to-end with Okta/Keycloak IdP
- Per-repo encryption keys rotate without pipeline interruption
- Multi-tenant isolation prevents cross-org data access (automated test)

### Estimated Timeline

10-14 weeks after v1.3.0

---

## v1.5.0 -- Horizontal Scaling

**Goal:** Enable CivitForge to run across multiple regions with Kubernetes
orchestration and read replicas for high availability.

### Key Deliverables

**Multi-Region Deployment**

- Extend `civit-core/src/federation/multimaster.rs` with async replication
  between regions using the existing `IncrementalSyncEngine`
- Region-aware routing in `civit-core/src/scaling/partitioner.rs`
- Conflict resolution for concurrent writes (last-writer-wins with tombstones)

**Kubernetes Operator Enhancement**

- Extend existing `civit-brain/` K8s operator with node affinity, pod
  disruption budgets, and horizontal pod autoscaling
- Custom resource definitions for CivitForge deployment, upgrade, and backup
- Status subresource with condition-based readiness

**Read Replicas**

- PostgreSQL read replica support in `civit-core/src/db/pool.rs` -- primary for
  writes, replica(s) for read queries via `sqlx::PgPoolOptions`
- Replica health checks and automatic failover detection

**CDN for Artifacts**

- Artifact serving via edge cache in `civit-core/src/cache/edge.rs`
  with `zstd` compression (already implemented)
- Pre-signed URLs for private artifact downloads
- Cache invalidation on artifact upload/overwrite

### Success Metrics

- CivitForge runs with 1 primary + 2 read replicas, all queries routed
  correctly
- Multi-region sync completes within 5s for a typical push
- Kubernetes operator deploys, upgrades, and backs up without manual intervention
- Artifact downloads served from cache with >80% hit rate

### Estimated Timeline

12-16 weeks after v1.4.0

---

## v2.0.0 -- Platform Expansion

**Goal:** Transform CivitForge from a web application into a cross-platform
forge ecosystem with desktop and mobile clients, an extension marketplace,
and a stable public API.

### Key Deliverables

**Tauri Desktop Application**

- Wrap `civit-ui` Leptos frontend in Tauri shell
- Native git operations via `gix` through Tauri commands (no separate server
  required for local repos)
- System tray integration, native notifications, drag-and-drop file upload
- Offline mode with local-first repo browsing

**PWA Mobile**

- Service worker for offline caching of read-only pages
- Push notification support for issue mentions, CI results, review requests
- Responsive touch-optimized layouts for issue list, repo browser, pipeline
  status

**Marketplace / Extensions**

- Extension API: webhook-triggered actions, custom CI steps, UI panels
- Extension manifest format (JSON schema, signed with existing
  `civit-crypto/src/cosign.rs`)
- Registry of community extensions with vulnerability scanning
- Permission sandboxing for extensions (no direct DB access)

**API Stability Guarantee**

- Public API versioning (`/api/v1/` frozen, `/api/v2/` for new features)
- OpenAPI spec generation from `civit-core/src/docs/openapi.rs` served at
  `/api/v1/openapi.json`
- Deprecation policy: 2 minor versions of notice before removal
- API compatibility tests in CI

### Success Metrics

- Tauri desktop app installs and runs on Linux, macOS, and Windows
- PWA passes Lighthouse audit with score >=90 for PWA category
- At least 5 community extensions available in the registry
- `/api/v1/` endpoints maintain backward compatibility across all v2.x
  releases (verified by automated compatibility tests)

### Estimated Timeline

16-20 weeks after v1.5.0

---

## Contributing Guide

### How to Contribute

CivitForge accepts contributions via GitHub pull requests. See
`CONTRIBUTING.md` for setup instructions, coding standards, and the PR process.

### Priority Areas

The following areas have the highest impact for contributors:

1. **`civit-ui` test coverage** -- the frontend crate has 0% test coverage.
   Component unit tests (wasm-bindgen-test), integration tests for API client
   calls, and screenshot regression tests are all needed.
2. **Stub removal** -- 43 stub patterns exist across workspace crates. Check
   `grep -rn "todo!\|unimplemented!\|FIXME\|STUB" crates/` for the current
   list. Each stub should either be implemented or replaced with a tracked
   issue reference.
3. **Error handling** -- 24 sites swallow errors with `let _ =` or
   `unwrap_or_default()`. Propagate errors properly and add user-facing
   messages where appropriate.
4. **Dead code cleanup** -- 66 `#[allow(dead_code)]` annotations need
   investigation: either remove the dead code or wire it into the application.
5. **Documentation** -- API endpoint docs, ADR updates, and user-facing
   guides for new features.

### Community Guidelines

- All code must pass `cargo clippy --workspace -- -D warnings` and
  `cargo fmt --check --all`
- Every `.rs` file must start with `#![forbid(unsafe_code)]`
- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)
  format: `<type>(<scope>): <description>`
- PRs require one approval and a green CI before merge (squash merge)
- No emojis in code, commit messages, or documentation
- Be respectful. See `CODE_OF_CONDUCT.md` (if present) for details

---

## Technical Debt Tracker

| Item | Priority | Affected Crates | Target Version |
|---|---|---|---|
| 3 critical WCAG 2.1 AA violations (skip nav, modal focus trap, toast aria-label) | Critical | civit-ui | v1.1.1 |
| Error messages leak raw API bodies to users | Critical | civit-core, civit-ui | v1.1.1 |
| 24 error-swallowing sites (`let _ =`, `unwrap_or_default`) | High | civit-core | v1.1.1 |
| 10 high-severity UX issues (missing labels, hardcoded data, raw UUIDs) | High | civit-ui | v1.2.0 |
| civit-ui at 0% test coverage | High | civit-ui | v1.2.0 |
| Pipelines CI/CD dashboard UI missing (API exists) | High | civit-ui | v1.2.0 |
| Global search UI missing (API exists) | High | civit-ui | v1.2.0 |
| Activity feed / notifications UI missing | High | civit-ui | v1.2.0 |
| Code browser page missing (needs gitfs/VFS integration) | High | civit-ui, civit-vfs | v1.2.0 |
| Settings page form non-functional | Medium | civit-ui | v1.2.0 |
| Org detail page shows raw UUID | Medium | civit-ui | v1.2.0 |
| Repos page uses hardcoded mock data | Medium | civit-ui | v1.2.0 |
| Form validation is minimal (empty checks only) | Medium | civit-ui | v1.2.0 |
| 66 `#[allow(dead_code)]` annotations | Medium | all crates | v1.2.0 |
| 2,202 commented-out lines | Medium | all crates | v1.2.0 |
| 43 stub patterns (`todo!`, `unimplemented!`, empty returns) | Medium | all crates | v1.2.0 |
| FUSE kernel mount incomplete (in-memory HashMap, no `mount()` syscall) | Low | civit-vfs | v1.2.0 |
| SAML XML-DSig canonicalization and signature verification incomplete | Medium | civit-crypto | v1.4.0 |
| WebAuthn ES-256/RS256 attestation verification incomplete | Medium | civit-crypto | v1.3.0 |
| HSM PKCS#11 real hardware integration (currently software-only) | Low | civit-crypto | v1.4.0 |
| Project boards / Kanban (no schema or API) | Medium | civit-core, civit-ui | v1.3.0 |
| Merge queue (scaffold exists, not wired) | Medium | civit-core | v1.3.0 |
| Dependency graph visualization (no implementation) | Low | civit-brain, civit-ui | v1.3.0 |
| Real-time WebSocket log streaming (event bus exists, runner not connected) | Medium | civit-core, civit-runner | v1.3.0 |
| Tantivy code search (PostgreSQL tsvector in use, tantivy deferred) | Medium | civit-brain, civit-core | v1.3.0 |
| Git-backed wiki storage (currently DB-only) | Low | civit-core | v1.3.0 |
| Per-repo encryption keys (global key currently) | Low | civit-crypto, civit-core | v1.4.0 |
| Multi-region replication (transport layer + vector clocks shipped) | Low | civit-core | v1.5.0 |
| K8s operator (CRD + reconciler shipped, node affinity done) | Low | civit-brain | v1.5.0 |
| CDN artifact pre-signed URLs and cache headers (shipped) | Low | civit-core | v1.5.0 |
| Password change does not verify current password | High | civit-core | v1.4.0 |
| Tauri desktop app (no implementation) | Low | new crate | v2.0.0 |
| PWA mobile (no implementation) | Low | civit-ui | v2.0.0 |
| Marketplace / extensions (no implementation) | Low | new crate | v2.0.0 |

---

*Last updated: 2026-06-04*
*Document owner: CivitForge core team*
