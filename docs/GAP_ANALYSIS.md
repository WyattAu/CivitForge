# CivitForge Gap Analysis (Legacy)

> **SUPERSEDED** by the comprehensive matrix at [`FEATURE_COMPARISON_MATRIX.md`](./FEATURE_COMPARISON_MATRIX.md).
>
> This file is preserved for historical reference. The new matrix includes:
> - Updated parity assessment (49% vs previous 30-35% estimate)
> - 89-item phased implementation plan with effort estimates
> - 6 emerging competitors (Radicle, Soft Serve, Woodpecker CI)
> - 22 CivitForge unique advantages (up from 17)
> - Week-by-week execution timeline

Cross-referencing CivitForge feature inventory against GitHub, GitLab, Gitea, Forgejo, and Codeberg.

**Legend:** ✅ CivitForge has it | ⚠️ Partial/stub | ❌ Missing entirely | 🔧 Backend only, no UI

---

## 1. CRITICAL Gaps — All 5 competitors have it

### Source Code

| # | Feature | CivitForge | Action Required |
|---|---------|-----------|-----------------|
| 1 | Git blame view | ❌ | Implement blame backend + UI in code browser |
| 2 | Commit history per file | ❌ | Add file-level log endpoint + UI panel |
| 3 | Inline diff view | ❌ | Render intra-line diffs in PR file view |
| 4 | Side-by-side diff | ❌ | Add split-pane diff mode in PR files |
| 5 | Syntax highlighting (50+ langs) | ❌ | Integrate highlight.js/Treesitter in code browser |
| 6 | Last commit column in dir listing | ❌ | Include latest commit info in tree API + UI |
| 7 | Branch switching in UI | ❌ | Add branch selector dropdown in code browser |
| 8 | LFS 2.0 | ❌ | Implement Git LFS batch API + transfer adapter |
| 9 | Web code editor | ❌ | Add Monaco/CodeMirror editor for single-file edit |
| 10 | Commit graph visualization | ❌ | SVG graph of branch topology on repo page |
| 11 | Language detection/stats | ❌ | Detect languages via extension/linguist + render bar |
| 12 | README rendering at repo root | ❌ | Auto-render README.md as HTML on repo detail |
| 13 | Raw file download | ✅ | — |
| 14 | Code browsing | ✅ | — |
| 15 | Search within repo | ✅ | — |

### Issues

| # | Feature | CivitForge | Action Required |
|---|---------|-----------|-----------------|
| 16 | Issue templates | ❌ | Support `.github/ISSUE_TEMPLATE/` parsing + form rendering |
| 17 | Assignees (single/multiple) | ❌ | Add assignee field + API + UI picker |
| 18 | File attachments | ❌ | Upload endpoint for issue/PR attachments |
| 19 | Cross-references (auto-link) | ❌ | Parse `#123`, `PR#456` references and render links |
| 20 | Time tracking | ❌ | Add time tracking fields (estimate, spent) + API |
| 21 | Issue dependencies/blocking | ❌ | Add blocked-by/blocks relationships |
| 22 | Issue pinning | ❌ | Pin endpoint + pinned sort in list |
| 23 | Lock discussion | ❌ | Lock/unlock endpoint + UI toggle |
| 24 | Batch issue handling | ❌ | Multi-select + bulk update API |
| 25 | Convert comment to issue | ❌ | Extract comment as new issue |
| 26 | Issue search (repo) | ✅ (global FTS) | Ensure repo-scoped filter works in UI |
| 27 | Issue due dates | ❌ | Add due_date field to issues |
| 28 | Issue tracker | ✅ | — |
| 29 | Labels | ✅ | — |
| 30 | Milestones | ✅ | — |
| 31 | Emoji reactions | ✅ | — |

### Pull/Merge Requests

| # | Feature | CivitForge | Action Required |
|---|---------|-----------|-----------------|
| 32 | PR/MR templates | ❌ | Support PR template file rendering |
| 33 | CODEOWNERS enforcement | ❌ | Parse CODEOWNERS + require review from owners |
| 34 | Squash merge | ❌ | Add squash strategy to merge endpoint |
| 35 | Rebase merge | ❌ | Add rebase strategy to merge endpoint |
| 36 | Fast-forward merge | ❌ | Add ff-only strategy to merge endpoint |
| 37 | Draft PRs | ❌ | Add draft flag + WIP prefix conversion |
| 38 | Required reviews | ❌ | Enforce minimum approvals before merge |
| 39 | Auto-merge | ❌ | Queue merge after checks pass |
| 40 | Merge queue | ❌ | Sequential merge queue with branch protection |
| 41 | Linked issues (auto-close) | ❌ | Parse "Fixes #123" and close on merge |
| 42 | Download patch | ❌ | Serve `.patch` format from PR endpoint |
| 43 | Push to existing PR | ❌ | Auto-update PR when branch is pushed to |
| 44 | Merge message templates | ❌ | Custom merge commit message format |
| 45 | Restrict push/merge to users | ❌ | Branch protection rule for allowed users |
| 46 | PR/MR creation | ✅ | — |
| 47 | Inline comments | ✅ | — |
| 48 | Status checks | ✅ | — |
| 49 | Merge commit | ✅ | — |
| 50 | Review requests | ✅ | — |
| 51 | Cherry-pick | ✅ (via Git API) | — |

### CI/CD

| # | Feature | CivitForge | Action Required |
|---|---------|-----------|-----------------|
| 52 | Matrix builds | ❌ | YAML parser needs `matrix` + `include` expansion |
| 53 | Parallelism | ❌ | Support `parallel` directive + job fan-out |
| 54 | Secrets management | ❌ | Encrypted secret store + inject at runtime |
| 55 | Environments | ❌ | Environment model (name, url, protection) |
| 56 | Deployments | ❌ | Deployment records linked to env + commit |
| 57 | Scheduled runs (cron) | ❌ | Cron trigger for pipeline YAML |
| 58 | Status badges | ❌ | SVG badge endpoint for pipeline status |
| 59 | Auto-cancel redundant runs | ❌ | Cancel superseded pipeline runs on push |
| 60 | DAG pipelines (`needs`) | ❌ | Topological sort + dependency graph execution |
| 61 | Pipeline YAML config | ✅ | — |
| 62 | Artifacts | ✅ | — |
| 63 | Manual triggers | ✅ | — |
| 64 | Container registry | ✅ (OCI) | — |
| 65 | Runner management | 🔧 (API, no UI) | Build runner management UI |

### Wiki

| # | Feature | CivitForge | Action Required |
|---|---------|-----------|-----------------|
| 66 | Sidebar navigation | ❌ | Auto-generate sidebar from page tree |
| 67 | TOC auto-generation | ❌ | Extract headers and render TOC |
| 68 | Image uploads | ❌ | Upload endpoint for wiki attachments |
| 69 | Wiki clone | ❌ | Ensure wiki git repo is cloneable |
| 70 | Wiki edit via web | ✅ | — |
| 71 | Built-in wiki | ✅ | — |
| 72 | Page history | ✅ | — |
| 73 | Markdown rendering | ✅ | — |

### Authentication

| # | Feature | CivitForge | Action Required |
|---|---------|-----------|-----------------|
| 74 | Email verification | ❌ | Send verification email + confirm endpoint |
| 75 | Account lockout | ❌ | Lock after N failed attempts + cooldown |
| 76 | Password policies | ❌ | Configurable complexity rules |
| 77 | OAuth2 provider | ❌ | Allow CivitForge to act as OAuth2 provider |
| 78 | Username/password | ✅ | — |
| 79 | 2FA/TOTP | ✅ | — |
| 80 | WebAuthn | ✅ | — |
| 81 | OIDC login | ✅ | — |

### Repository Management

| # | Feature | CivitForge | Action Required |
|---|---------|-----------|-----------------|
| 82 | Topics/tags | ❌ | Add topics field to repo + searchable |
| 83 | Transfer ownership | ❌ | Transfer endpoint + confirmation flow |
| 84 | Archive repo | ❌ | Soft-archive (read-only) endpoint |
| 85 | Rename repo | ❌ | Rename endpoint + redirect old URL |
| 86 | Push mirror | ❌ | Push mirror config + sync worker |
| 87 | Default branch setting | ❌ | Allow changing default branch |
| 88 | Branch protection rules | ❌ | Rule engine for protected branches |
| 89 | Tag protection rules | ❌ | Restrict who can create tags |
| 90 | Reject unsigned commits | ❌ | GPG/SSH verify on push + reject |
| 91 | Signed commit verification (GPG) | ❌ | Verify GPG signatures on commits |
| 92 | Signed commit verification (SSH) | ❌ | Verify SSH signatures on commits |
| 93 | Template repositories | ❌ | Mark repo as template + scaffold from it |
| 94 | Repo activity page | ✅ | — |
| 95 | Fork | ✅ | — |
| 96 | Star/watch | ✅ | — |

### Organizations / Teams

| # | Feature | CivitForge | Action Required |
|---|---------|-----------|-----------------|
| 97 | Team management | ❌ | Create/edit/delete teams within org |
| 98 | Team-level permissions | ❌ | Assign repo-level perms to teams |
| 99 | Org visibility (public/private) | ❌ | Visibility setting on org |
| 100 | Membership requests | ❌ | Request/approve flow for org joining |
| 101 | Organization profile page | 🔧 (API, no UI) | Build org detail page |
| 102 | Org creation | ✅ | — |

### Collaboration

| # | Feature | CivitForge | Action Required |
|---|---------|-----------|-----------------|
| 103 | Notifications (email/in-app) | ❌ | Notification backend + email worker + UI inbox |
| 104 | @mentions | ❌ | Parse @username in markdown + send notification |
| 105 | Task lists in markdown | ❌ | Render `- [ ]` / `- [x]` as checkboxes |
| 106 | Auto-linked references | ❌ | Same as cross-references, applies to PRs too |
| 107 | Pin issues/PRs | ❌ | Pin endpoint + pinned section in list |
| 108 | Issue assignments | ❌ | Same as assignees (row 17) |
| 109 | Watch/star/fork | ✅ | — |
| 110 | Emoji reactions | ✅ | — |
| 111 | Label system | ✅ | — |

### API

| # | Feature | CivitForge | Action Required |
|---|---------|-----------|-----------------|
| 112 | Webhooks | ❌ | Webhook CRUD + delivery worker + retry |
| 113 | Personal access tokens | ❌ | Token CRUD + scopes |
| 114 | Rate limiting | ❌ | Per-user/per-IP rate limiter middleware |
| 115 | Git hooks (pre-receive) | ❌ | Server-side hook execution on push |
| 116 | REST API | ✅ | — |

### Other

| # | Feature | CivitForge | Action Required |
|---|---------|-----------|-----------------|
| 117 | Release management | ❌ | Releases CRUD + tag association + assets |
| 118 | Deploy keys | ❌ | Read-only deploy key per repo |
| 119 | Deploy tokens | ❌ | Scoped tokens for CI/CD deployment |
| 120 | Project boards / Kanban | ❌ | Basic issue board drag-and-drop |
| 121 | Import from GitHub/GitLab | ❌ | Migration endpoint + adapters |
| 122 | GPG signing (instance) | ❌ | Instance-level commit signing key |
| 123 | Moderation tools | ❌ | Report, hide, ban user content |
| 124 | Dashboard | ✅ | — |
| 125 | Activity feed | ✅ | — |
| 126 | SSH key management | ✅ | — |

### UI/UX

| # | Feature | CivitForge | Action Required |
|---|---------|-----------|-----------------|
| 127 | Dark mode | ❌ | Theme toggle + dark CSS variables |
| 128 | Responsive / mobile-friendly | ❌ | CSS breakpoints + mobile layouts |
| 129 | Keyboard shortcuts | ❌ | Global keybindings for common actions |
| 130 | Multi-language UI (i18n) | ❌ | i18n framework + translation files |
| 131 | User profile customization | ❌ | Editable bio, website, location, avatar |
| 132 | Custom footer/logo | ❌ | Instance admin settings for branding |

---

## 2. HIGH Gaps — 4 of 5 competitors have it

### Issues

| # | Feature | Score | CivitForge | Action Required |
|---|---------|-------|-----------|-----------------|
| 133 | Issue analytics / boards | 4/5 (Gitea/Forgejo/Codeberg ⚠️) | ❌ | Basic issue board view |
| 134 | Kanban boards | 4/5 (Gitea/Forgejo/Codeberg ⚠️) | ❌ | Combined with #120 |
| 135 | Issue search (global) | 4/5 (Gitea/Forgejo/Codeberg ⚠️) | ✅ | Verify global scope in UI |
| 136 | Issue due dates | 4/5 (see #27) | ❌ | Combined with #27 |
| 137 | File attachments | 4/5 (see #18) | ❌ | Combined with #18 |

### Source Code

| # | Feature | Score | CivitForge | Action Required |
|---|---------|-------|-----------|-----------------|
| 138 | Mermaid diagrams in markdown | 4/5 (GitLab ✅, rest ✅) | ❌ | Mermaid JS integration in markdown renderer |
| 139 | Math (LaTeX/KaTeX) in markdown | 4/5 | ❌ | KaTeX integration in markdown renderer |
| 140 | CSV rendering | 4/5 | ❌ | Render CSV files as HTML tables |
| 141 | Visual image diff | 4/5 | ❌ | Side-by-side image comparison in PR |
| 142 | Color-coded blame | 4/5 | ❌ | Combined with #1 |
| 143 | Repository size display | 4/5 | ❌ | Show repo disk usage on detail page |

### Security

| # | Feature | Score | CivitForge | Action Required |
|---|---------|-------|-----------|-----------------|
| 144 | Security policy files (SECURITY.md) | 4/5 (Codeberg has via Forgejo) | ❌ | Render SECURITY.md on repo |
| 145 | Vulnerability reporting | 4/5 | ❌ | Private vulnerability report form |
| 146 | CODEOWNERS | 5/5 | ❌ | Combined with #33 |

### Package Registry

| # | Feature | Score | CivitForge | Action Required |
|---|---------|-------|-----------|-----------------|
| 147 | npm registry | 4/5 (GitLab ✅) | ❌ | Implement npm package API |
| 148 | PyPI registry | 4/5 (GitLab ✅) | ❌ | Implement PyPI package API |
| 149 | Maven registry | 4/5 (GitLab ✅) | ❌ | Implement Maven package API |
| 150 | Go registry | 4/5 (GitLab ✅) | ❌ | Implement Go module proxy API |
| 151 | Helm Charts | 4/5 (GitLab ✅) | ❌ | Helm chart repository API |
| 152 | NuGet registry | 4/5 (GitLab ✅) | ❌ | Implement NuGet package API |
| 153 | Package-link to repo | 5/5 | ⚠️ (OCI only) | Extend to all package types |
| 154 | Package deduplication | 5/5 | ❌ | Content-addressable storage for packages |

### CI/CD

| # | Feature | Score | CivitForge | Action Required |
|---|---------|-------|-----------|-----------------|
| 155 | Caches | 4/5 (GitLab ✅, Gitea/Forgejo/Codeberg runner-side) | ❌ | Cache API + mount in runner |
| 156 | Pipeline visualization | 4/5 (Gitea/Forgejo/Codeberg ⚠️) | ❌ | DAG graph rendering in pipeline UI |
| 157 | OIDC for workload identity | 5/5 | ❌ | Issue OIDC tokens to pipelines |

### Authentication

| # | Feature | Score | CivitForge | Action Required |
|---|---------|-------|-----------|-----------------|
| 158 | LDAP / AD | 4/5 (Codeberg ❌) | ❌ | LDAP auth backend |
| 159 | SCIM provisioning | 2/5 (GitHub/GitLab Enterprise only) | ❌ | Low priority — only SaaS |

### Organizations

| # | Feature | Score | CivitForge | Action Required |
|---|---------|-------|-----------|-----------------|
| 160 | Audit log | 4/5 (Gitea/Forgejo/Codeberg ⚠️) | ❌ | Structured audit trail + admin view |

---

## 3. MEDIUM Gaps — 3 of 5 competitors have it

| # | Feature | Score | CivitForge | Action Required |
|---|---------|-------|-----------|-----------------|
| 161 | Pull mirror | 3/5 (GitHub ❌) | ❌ | Pull mirror config + periodic fetch |
| 162 | In-browser conflict resolution | 2/5 (GitHub/GitLab) | ❌ | 3-way merge editor UI |
| 163 | Suggested edits in PRs | 2/5 | ❌ | One-click apply suggestion |
| 164 | Submodules support in UI | 5/5 | ❌ | Render submodule links in tree |
| 165 | Issue import/export | 5/5 (see #58) | ❌ | Import/export adapters for GitHub/GitLab |
| 166 | Confidential issues | 1/5 (GitLab EE only) | ❌ | Low priority |
| 167 | Nested teams | 2/5 (GitHub/GitLab EE) | ❌ | Team hierarchy model |
| 168 | RSS feeds | 3/5 (GitHub/Gitea/Forgejo/Codeberg ✅, GitLab ❌) | ❌ | RSS/Atom feed endpoints |
| 169 | Issue transfer between repos | 2/5 | ❌ | Move issue to another repo |
| 170 | Conflict detection in PRs | 5/5 (Gitea/Forgejo/Codeberg ⚠️) | ❌ | Check mergeability and show conflicts |
| 171 | PR/MR size indicator | 1/5 (GitLab only) | ❌ | Low priority |
| 172 | Revert commit in UI | 5/5 | ❌ | Revert button on commit detail |
| 173 | Code Quality reports | 1/5 (GitLab only) | ❌ | Low priority |
| 174 | DAST scanning | 2/5 (GitHub/GitLab Advanced) | ❌ | Low priority — security scanner |
| 175 | Container scanning | 2/5 | ❌ | Low priority — security scanner |
| 176 | Private vulnerability reports | 5/5 | ❌ | Combined with #145 |
| 177 | Secrets management | 5/5 | ❌ | Combined with #54 |
| 178 | Merge checklist | 2/5 | ❌ | Checklist template in PR description |
| 179 | AGit / email-based PRs | 3/5 (Gitea/Forgejo/Codeberg ✅) | ❌ | Patch via email ingestion |
| 180 | Soft quota (repo size limits) | 3/5 | ❌ | Configurable max repo size per instance |
| 181 | Multiple LDAP sources | 3/5 | ❌ | Extend LDAP config to multi-source |
| 182 | LDAP user sync | 3/5 | ❌ | Periodic LDAP sync job |
| 183 | Verified committer badge | 5/5 (Gitea/Forgejo/Codeberg ⚠️) | ❌ | Badge for verified signatures |
| 184 | Snippets / Gists | 1/5 (GitLab only native) | ❌ | Low priority — consider plugin |
| 185 | Pages (static site hosting) | 3/5 (Gitea 🔧, Codeberg/Forgejo ✅) | ❌ | Static site builder + serve |
| 186 | Changelog generation | 1/5 | ❌ | Auto-generate from merged PRs |
| 187 | Jupyter notebook rendering | 2/5 | ❌ | Render .ipynb as HTML |
| 188 | PDF rendering in browser | 1/5 | ❌ | Very low priority |
| 189 | 3D model viewer | 1/5 | ❌ | Very low priority |

---

## 4. LOW Gaps — 2 or fewer competitors have it

| # | Feature | Score | CivitForge | Action Required |
|---|---------|-------|-----------|-----------------|
| 190 | Group milestones | 1/5 (GitLab EE) | ❌ | Very low priority |
| 191 | Sub-epics / hierarchy | 2/5 (GitHub/GitLab EE) | ❌ | Low priority |
| 192 | Scoped labels | 1/5 (GitLab) | ❌ | Very low priority |
| 193 | Weight / estimate | 2/5 (GitHub Projects, GitLab) | ❌ | Low priority |
| 194 | Issue via email | 2/5 | ❌ | Low priority |
| 195 | Service desk (external tickets) | 1/5 (GitLab EE) | ❌ | Very low priority |
| 196 | Create branch from issue | 2/5 | ❌ | Low priority |
| 197 | SAML SSO | CivitForge ✅ | — | Already implemented |
| 198 | GraphQL API | 2/5 (GitHub/GitLab) | ❌ | Low priority for self-hosted |
| 199 | Native desktop app | 1/5 | ❌ | Not in scope |
| 200 | Mobile apps | 0/5 | ❌ | Not in scope — responsive web instead |
| 201 | Codespaces / Workspaces | 2/5 | ❌ | Not in scope — use VFS instead |
| 202 | Collaborative editing | 1/5 (GitLab Web IDE) | ❌ | Not in scope |
| 203 | Concurrent editing | 0/5 | ❌ | Not in scope |
| 204 | Grafana integration | 1/5 | ❌ | Not in scope — use metrics export |
| 205 | CLA tooling | — | ❌ | Very low priority |
| 206 | SBOM generation | 1/5 (GitLab, GitHub ⚠️) | ⚠️ | Could extend SLSA provenance |
| 207 | Secret scanning | 2/5 (GitHub/GitLab) | ❌ | Medium priority — could leverage AI AST |

---

## 5. CivitForge Competitive Advantages — No competitor has these

| # | Feature | Detail |
|---|---------|--------|
| A1 | **Federation (full)** | Complete ActivityPub + ForgeFed + multi-master replication + vector clocks. All competitors have ❌ or ⚠️ (in development). CivitForge is the only production-ready federated forge. |
| A2 | **AI: RAG pipeline** | Retrieval-Augmented Generation with HNSW vector DB + Qdrant client for contextual code search. No competitor has open-source RAG. |
| A3 | **AI: AST engine** | TreeSitter-based AST analysis for 20+ languages. CodeQL-like semantic analysis. No competitor has this in open-source. |
| A4 | **AI: PR review** | Automated AI-powered PR review using LLM inference. GitHub/GitLab have this as paid AI. |
| A5 | **AI: Code generation** | LLM inference engine integrated into the platform. |
| A6 | **SLSA provenance** | Full SLSA provenance generation + verification. Only GitHub/GitLab have partial support. |
| A7 | **FIPS 140-2 self-test** | FIPS compliance with HSM PKCS#11 support. No competitor has this. Enterprise/government differentiator. |
| A8 | **ABAC engine** | Attribute-Based Access Control with CEL expression language. Goes beyond competitors' RBAC-only models. |
| A9 | **VFS (FUSE)** | Virtual filesystem via FUSE + gRPC. Mount repos as filesystems. No competitor has this. |
| A10 | **K8s operator** | Full Kubernetes operator + CRDs + Helm chart. GitLab has it; Gitea/Forgejo have community charts. |
| A11 | **Podman-based runner** | Rootless container runner using Podman. Competitors use Docker exclusively. |
| A12 | **Edge caching** | Built-in edge caching + pre-signed URLs for artifacts. No competitor has this in self-hosted. |
| A13 | **Auto-scaler** | Built-in auto-scaler + sharder + partitioner for horizontal scaling. Unique for self-hosted forges. |
| A14 | **Load test runner** | Built-in load testing infrastructure. No competitor has this. |
| A15 | **mTLS everywhere** | Mutual TLS for internal service communication + Cosign OCI signing. |
| A16 | **CMDB** | Configuration Management Database integrated into the platform. No competitor has this. |
| A17 | **ISO 27001 compliance engine** | Built-in compliance checks and audit framework. Enterprise differentiator. |

---

## 6. Backend-Only Features Needing UI

| # | Feature | Backend Status | UI Needed |
|---|---------|--------------|-----------|
| U1 | Pipeline detail/job view | ✅ Full API | Pipeline detail page, job log viewer, step expansion |
| U2 | Org detail/profile page | ✅ CRUD API | Org landing page with repos, members, teams |
| U3 | Settings (org-level) | ✅ API exists | Org settings panel (visibility, avatar, description) |
| U4 | Notifications | ❌ No backend | Build both backend + UI (see #103) |
| U5 | Runner management | ✅ Register/list/deregister | Runner list, status, job assignment, logs |
| U6 | Marketplace | ✅ Extensions CRUD | Marketplace browse, install, review UI |
| U7 | Artifacts | ✅ Download + pre-signed | Artifact list per pipeline, download links |
| U8 | OCI Registry | ✅ Full spec | Package browser page, version list, layer info |
| U9 | WebAuthn management | ✅ DB tables exist | Security settings page for passkey enrollment |

---

## 7. Stubs Needing Implementation

| # | Feature | Current State | Effort |
|---|---------|--------------|--------|
| S1 | Archive download (zip/tar) | Returns `NOT_IMPLEMENTED` | Integrate git-archive or zip library. All 5 competitors have this. **CRITICAL** |
| S2 | Add/remove collaborators | Returns `NOT_IMPLEMENTED` | Collaborator CRUD on repo. All 5 competitors have this. **CRITICAL** |
| S3 | Search re-index trigger | Returns `NOT_IMPLEMENTED` | Admin endpoint to trigger Tantivy re-index. **MEDIUM** |

---

## 8. Priority Matrix — Recommended Implementation Order

### Phase 1: Parity Essentials (unblock basic usage)

Implement these first — they are features all users expect from any forge:

1. **Blame view** (#1) — fundamental to code browsing
2. **Syntax highlighting** (#5) — non-negotiable for any code viewer
3. **Branch switching** (#7) — basic navigation
4. **Assignees** (#17) — core issue workflow
5. **Cross-references** (#19) — issue/PR linking
6. **@mentions** (#104) — collaboration
7. **Notifications** (#103) — user engagement
8. **Email verification** (#74) — security baseline
9. **Account lockout** (#75) — security baseline
10. **Archive download** (#S1) — complete the stub
11. **Collaborator management** (#S2) — complete the stub
12. **Dark mode** (#127) — UX baseline
13. **Responsive/mobile** (#128) — UX baseline
14. **Deploy keys/tokens** (#118-119) — CI/CD prerequisite
15. **Webhooks** (#112) — ecosystem integration
16. **Personal access tokens** (#113) — API usage
17. **Rate limiting** (#114) — security baseline
18. **Branch/tag protection** (#88-89) — repo security
19. **Release management** (#117) — software distribution
20. **Pipeline detail UI** (#U1) — CI/CD usability
21. **Org profile page** (#U1) — org visibility
22. **Runner management UI** (#U5) — CI/CD operations

### Phase 2: Core Workflow Completion

1. **Squash/rebase/ff merge** (#34-36) — merge strategies
2. **Draft PRs** (#37) — common workflow
3. **CODEOWNERS** (#33) — team workflow
4. **Issue templates** (#16) — issue quality
5. **File attachments** (#18) — issue/PR context
6. **Task lists in markdown** (#105) — issue tracking
7. **Secrets management** (#54) — CI/CD security
8. **Matrix/parallel builds** (#52-53) — CI/CD power
9. **DAG pipelines** (#60) — CI/CD power
10. **Scheduled runs** (#57) — CI/CD automation
11. **Status badges** (#58) — repo README integration
12. **Topics** (#82) — repo discoverability
13. **Team management** (#97-98) — org workflow
14. **Import from GitHub/GitLab** (#121) — migration path
15. **LFS 2.0** (#8) — large file support
16. **Commit graph** (#10) — repo visualization
17. **README rendering** (#12) — repo landing
18. **Web code editor** (#9) — quick edits
19. **Language stats** (#11) — repo overview
20. **Merge queue** (#40) — branch protection enforcement
21. **Linked issues auto-close** (#41) — workflow automation
22. **Revert commit** (#172) — common operation

### Phase 3: Competitive Differentiation (amplify advantages)

1. **Federation polish** — stabilize cross-instance PRs, follow UX
2. **AI PR review** — ship the RAG + AST pipeline in UI
3. **AI code search** — expose Tantivy + HNSW via natural language
4. **VFS launch** — market FUSE mount as killer feature
5. **SLSA provenance** — dashboard for supply chain transparency
6. **Edge caching + pre-signed URLs** — performance marketing
7. **K8s operator GA** — enterprise deployment story
8. **mTLS + Cosign** — harden CI/CD supply chain

### Phase 4: Package Ecosystem

1. **npm registry** (#147)
2. **PyPI registry** (#148)
3. **Maven registry** (#149)
4. **Go module proxy** (#150)
5. **Helm Charts** (#151)
6. **NuGet registry** (#152)
7. **Package deduplication** (#154)
8. **Markdown rendering** (Mermaid #138, KaTeX #139, CSV #140)

---

## 9. Summary Statistics

| Category | Count |
|----------|-------|
| CRITICAL gaps (5/5 competitors) | **132** features missing |
| HIGH gaps (4/5 competitors) | **24** features missing |
| MEDIUM gaps (3/5 competitors) | **29** features missing |
| LOW gaps (≤2/5 competitors) | **18** features (most out of scope) |
| Stubs needing implementation | **3** |
| Backend-only needing UI | **9** |
| CivitForge unique advantages | **17** |

**Estimated parity**: CivitForge covers roughly **30-35%** of the features all 5 competitors have, but holds **17 unique advantages** none of them possess. The strategy should be: reach basic parity on Phase 1 items quickly, then lean into unique advantages (federation, AI, VFS, FIPS, K8s) as differentiators.
