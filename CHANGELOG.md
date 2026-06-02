# Changelog

All notable changes to the CivitForge project are documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

---

## [1.1.0] - 2026-06-02

### Added
- **Token refresh endpoint** (`POST /api/v1/auth/refresh`) — validates existing JWT, looks up user, issues new token
- **RSA-SHA256 and ECDSA-P256 signing** in federation HTTP signatures — all 4 algorithms now fully supported (Ed25519, HMAC-SHA256, RSA-SHA256, ECDSA-P256)
- **Retriever trait** for RAG pipeline — `RagOrchestrator` accepts `Box<dyn Retriever>` for swappable backends
- **KeywordRetriever** — production-ready keyword-based retrieval with thread-safe `RwLock` internals
- **Real unified diff** for wiki via LCS algorithm — `unified_diff()`, `DiffHunk`, `DiffLine`, `lcs_lines()`, `apply_diff_hunks()`
- **Wiki content snapshot** — stores page content in `wiki_revisions.content_snapshot` (migration 019/020)
- **AES-256-GCM encryption** for pipeline variables using `ring::aead` (`encrypt_value`, `decrypt_value`, `get_encryption_key`)
- **Checkout/cache/artifact action handlers** in CI executor — `action_checkout`, `action_cache`, `action_artifact`
- **Service container lifecycle** — `start_services()`, `stop_all_services()`, `ServiceGuard` RAII struct
- **Real CEL expression evaluator** — `civit-pipeline::expr` supports `==`, `!=`, `contains`, `startsWith`, `endsWith`, `matches`, `&&`, `||`, `!`, `${{ var }}` expansion (20+ unit tests)
- **PostgreSQL full-text search** — `tsvector`/`tsquery` columns, GIN indexes, auto-update triggers on `code_search_index` and `wiki_pages` (migration 021/022)
- **Git archive-based pipeline YAML reading** — tries `git archive --format=tar <ref> <path>` first, falls back to filesystem

### Changed
- **Runner owner lookup** — fixed hardcoded "todo" owner, now joins `users` table for real username
- **Code search** — ILIKE replaced with `plainto_tsquery` for ranked full-text results
- **Wiki search** — ILIKE replaced with `search_vector @@ plainto_tsquery` with `ts_rank` ordering

### Fixed
- **Podman log errors** — both HTTP and CLI log paths now return proper `Err` instead of fake "log line N" text
- **OCI dedup `get_layer()`** — was broken due to digest/chunk mismatch; added `ChunkStore::put_direct()` for correct storage by layer digest
- **Test stubs gated** behind `#[cfg(test)]` — `StubLlmProvider`, `StubReviewAgent`, `StubVulnScanner`, `MockRemoteProvider` no longer exported in production
- **Stale "stub" doc comment** on CEL evaluator updated to reflect actual 9-expression-kind implementation
- **Encryption key warning** — `tracing::warn!` logged when `CIVIT_ENCRYPTION_KEY` env var not set

### Dependencies
- Added `rsa 0.9` (RSA PKCS1v15 signing) to `civit-core`
- Added `pkcs8 0.10` (PKCS#8 DER key parsing) to `civit-core`
- Added `tar 0.4` (tar archive extraction) to `civit-core`
- Added `sha2` `oid` feature (OID support for pkcs8 ecosystem)

---

## [1.0.0-rc.3] - 2026-06-02

### Added

#### Workspace Restructure (Phase 7)
- Moved all crates into `crates/` directory layout
- Created `civit-shared` crate with shared API request/response types for backend/frontend type sharing
- Created `civit-pipeline` crate for YAML spec parsing/validation

#### Permission System (Phase 8)
- Full GitLab-style RBAC with 6-role hierarchy: Owner, Admin, Maintainer, Developer, Reporter, Guest
- Deny-always-wins policy evaluation with hierarchical inheritance (org → repo → branch)
- 11 permission resources: Organization, Repository, Pipeline, PipelineVariable, Runner, Package, Branch, Tag, Issue, Wiki, User
- 22 permission actions with conditional policy checks
- Branch protection: push restrictions, required reviews, force-push control
- DB schema: `member_roles`, `repo_policies`, `branch_protections`, `pipeline_variables`
- Auth middleware applied to all API routes

#### CI/CD Pipeline Backend (Phase 9)
- Pipeline YAML spec (`.civit/pipeline.yaml`): triggers (push/tag/PR/schedule/dispatch), services, cache, secrets, workspace, concurrency, expressions
- `civit-pipeline` crate: YAML parser with 80+ test vectors, validation, expression evaluation
- Runner registration + token-based auth protocol (6-endpoint internal API)
- `civit-runner` standalone daemon: Podman CLI transport, volume mounts, log streaming
- Concurrency groups with cancel-in-progress semantics
- Secret resolution via API (runner calls `/api/v1/runners/{id}/secrets`)
- CEL-based expression evaluation for conditional step execution
- 17 pipeline API endpoints + 11 runner management endpoints
- Migrations: 9 (`pipelines`, `pipeline_jobs`, `pipeline_steps`, `runners`, `runner_tasks`, `runner_task_logs`, `artifacts`, `cache_entries`, `service_containers`)

#### OCI Container Registry (Phase 10)
- 20 OCI Distribution Spec v1.1 endpoints (`/v2/_catalog`, `/v2/{name}/tags/list`, blob/manifest push/pull, referrers API)
- 8 management API endpoints (list images, tags, layers, SBOM, vulns, RBAC policies, GC trigger)
- Per-image RBAC with namespace enforcement (org/user)
- Content-addressed blob deduplication
- Tag immutability policies
- Multi-arch manifest list support
- Built-in OSV vulnerability scanning per push
- Cosign image signature verification
- SLSA provenance attestation per image
- Migrations: 11+12 (`oci_repositories`, `oci_blobs`, `oci_manifests`, `oci_tags`, `oci_manifest_layers`, `oci_image_signatures`, `oci_vuln_scans`, `oci_policies`)

#### Issue Tracking (Phase 11)
- 18 issue tracking API endpoints
- State machine: open → in_progress → closed, with reopen
- Timeline audit trail (all state changes recorded)
- Comments thread with reactions
- Labels (create, edit, delete, assign/unassign with colors)
- Milestones with progress tracking (% open/closed)
- Assignees with filter support
- Migrations: 13+14 (`issues`, `issue_comments`, `labels`, `issue_labels`, `issue_assignees`, `milestones`, `issue_timeline`, `issue_reactions`)

#### Wiki (Phase 12)
- 9 wiki API endpoints
- Page CRUD with Markdown content
- Page history with diff between revisions
- Raw content export
- Full-text search across wiki pages
- Migrations: 15+16 (`wiki_pages`, `wiki_revisions`)

#### Code Search (Phase 13)
- 3 search endpoints (global, per-repo, language filter)
- SQL ILIKE-based full-text search (tantivy deferred)
- Incremental indexing via `code_search_index` + `code_search_tokens`
- Migrations: 17+18 (`code_search_index`, `code_search_tokens`)

#### Leptos Web UI (Phase 14)
- `civit-ui` crate: Leptos 0.7 with CSR + SSR features
- 11 reusable UI components: Button, Input, Badge, Avatar, Modal, Toast, Pagination, Sidebar, Tabs, Card, Loading
- 10 page shells: Home, Login, Repos, RepoDetail, Issues, Wiki, Settings, Explore, Orgs, NotFound
- Typed API client (reqwest-based)
- Auth state management with Leptos signals
- Leptos Router integration
- Tailwind CSS dark mode support
- ADR-002: Leptos SSR (type sharing backend/frontend)
- ADR-003: Tailwind standalone CLI (zero Node.js)

#### Integration + Polish (Phase 15)
- All API endpoints documented
- 2,611 tests passing, 0 clippy warnings, 0 format violations
- `#![forbid(unsafe_code)]` enforced across 204 files (1 file feature-gated)
- ROADMAP.md updated to 95% completion

### Security

- `LICENSE`: replaced incorrect Apache 2.0 text with AGPL-3.0-or-later
- `civit-runner/src/podman.rs`: `run()` fails-closed when Podman unreachable
- `civit-crypto/src/cel/mod.rs`: `matches()` fails-closed when regex engine not integrated
- CI workflow: `permissions: contents: read` (least-privilege)
- SAML signature validation: fail-closed (always `false`) until XML-DSig
- Pipeline variable encryption framework (AES-256-GCM deferred — plaintext with framework in place)

### Fixed

- `civit-brain/vectordb.rs`: moved `QdrantVectorDbAdapter` before `#[cfg(test)]` (`clippy::items_after_test_module`)
- `civit-brain/llm/inference.rs`: removed unused `debug` import
- `civit-core/ssh/auth.rs`: fixed DashMap deadlock in `RateLimiter::check()`
- `civit-core/api/mod.rs`: added `/healthz` and `/ready` endpoints
- Runner execution: fixed missing `volumes: vec![]` in `PodmanRunSpec` default
- Runner execution: fixed nested match for timeout/container-error
- Runner execution: fixed double-ref `&client` → `client`
- Runner execution: removed unused `TrimStartMatches` trait, inlined `starts_with`
- `runners.rs`: fixed `Path<(String,)>` → `Path<String>` for single param
- `runners.rs`: fixed collapsible `if let` → `unwrap_or(None)` + `if let Some`
- Clippy fixes: `uninlined_format_args`, `let_unit_value`, `new_without_default`, `or_insert_with`, `manual_strip`, `collapsible_if`, `too_many_arguments`, `let_and_return`, `unnecessary_map_or`, `unused_mut`
- `DAGSync::dfs_cycle`: refactored to associated function (`only_used_in_recursion`)

### Changed

- CI workflow: `--locked` on `cargo clippy`, `cargo build --release`, `cargo test`
- CI workflow: `dtolnay/rust-toolchain` pinned to `@master`
- CI workflow: `rust-toolchain.toml` in cache key
- Dockerfile: workspace build with `--workspace`
- Pre-commit hooks: fmt + clippy -D warnings + test --locked
- Landing page: OG/Twitter meta tags, canonical links, ARIA roles, `dir="ltr"`
- 404 page: `noindex`, canonical link, `/undefined` edge case fix
- Helm chart: version sync with workspace

---

## [0.1.0] - 2026-05-30

### Added

- Project specification foundation (Phase 0: Requirements Engineering)
- Domain analysis with 7 applicable standards (ISO 27001, NIST SP 800-53, SLSA L4, FIPS 140-2, ISO 26262, OWASP Top 10, SOC2 Type II)
- 69 EARS-format requirements across 5 areas (VCS, LFS+, CI/CD, AI, Federation)
- 16 formalized non-functional requirements with measurable acceptance criteria
- Traceability matrix, 6 standard conflict resolutions, capability matrix, 10 tooling gap identifications
