# Platform Feature Comparison

GitHub vs GitLab vs Gitea vs Forgejo vs Codeberg

**Legend:** [YES] Supported | [NO] Not supported | [PARTIAL] Partial support | [BE] Via plugin/3rd party | [PAID] Requires paid tier

---

## 1. Source Code

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Code browsing | [YES] | [YES] | [YES] | [YES] | [YES] |
| Git blame view | [YES] | [YES] | [YES] | [YES] | [YES] |
| Commit history per file | [YES] | [YES] | [YES] | [YES] | [YES] |
| Inline diff view | [YES] | [YES] | [YES] | [YES] | [YES] |
| Side-by-side diff | [YES] | [YES] | [YES] | [YES] | [YES] |
| Syntax highlighting | [YES] (rich, 100+ langs) | [YES] (rich, Rouge) | [YES] (Chroma, many langs) | [YES] (Chroma) | [YES] (Chroma) |
| Last commit column | [YES] | [YES] | [YES] | [YES] | [YES] |
| File finder (`.` or `t`) | [YES] | [YES] | [PARTIAL] (basic tree nav) | [PARTIAL] (basic tree nav) | [PARTIAL] (basic tree nav) |
| Search within repo | [YES] | [YES] | [YES] | [YES] | [YES] |
| Branch switching in UI | [YES] | [YES] | [YES] | [YES] | [YES] |
| Submodules support | [YES] | [YES] | [YES] | [YES] | [YES] |
| LFS 2.0 | [YES] | [YES] | [YES] | [YES] | [YES] |
| Raw file download | [YES] | [YES] | [YES] | [YES] | [YES] |
| Archive download (zip) | [YES] | [YES] | [YES] | [YES] | [YES] |
| Archive download (tar.gz) | [YES] | [YES] | [YES] | [YES] | [YES] |
| Archive download (tar.bz2) | [YES] | [YES] | [YES] | [YES] | [YES] |
| Web code editor | [YES] | [YES] | [YES] | [YES] | [YES] |
| Commit graph visualization | [YES] | [YES] | [YES] | [YES] | [YES] |
| Visual image diff | [YES] | [YES] | [YES] | [YES] | [YES] |
| Color-coded blame | [YES] | [YES] | [YES] | [YES] | [YES] |
| Repository size display | [YES] | [YES] | [YES] | [YES] | [YES] |
| Language detection/stats | [YES] | [YES] | [YES] | [YES] | [YES] |
| README rendering at repo root | [YES] | [YES] | [YES] | [YES] | [YES] |
| Mermaid diagrams in markdown | [YES] | [YES] | [YES] | [YES] | [YES] |
| Math (LaTeX/KaTeX) in markdown | [YES] | [YES] | [YES] | [YES] | [YES] |
| CSV rendering | [YES] | [YES] | [YES] | [YES] | [YES] |
| 3D model viewer (.stl) | [YES] | [NO] | [NO] | [NO] | [NO] |
| Jupyter notebook rendering | [YES] | [YES] | [NO] | [NO] | [NO] |
| PDF rendering in browser | [YES] | [NO] | [NO] | [NO] | [NO] |

---

## 2. Issues

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Issue tracker | [YES] | [YES] | [YES] | [YES] | [YES] |
| Issue templates | [YES] | [YES] | [YES] | [YES] | [YES] |
| Labels (color-coded) | [YES] | [YES] | [YES] | [YES] | [YES] |
| Milestones | [YES] | [YES] | [YES] | [YES] | [YES] |
| Group/org milestones | [NO] | [YES] | [NO] | [NO] | [NO] |
| Assignees (single/multiple) | [YES] | [YES] (multiple, EE) | [YES] (multiple) | [YES] (multiple) | [YES] (multiple) |
| Emoji reactions | [YES] | [YES] | [YES] | [YES] | [YES] |
| File attachments | [YES] | [YES] | [YES] | [YES] | [YES] |
| Cross-references (auto-link) | [YES] | [YES] | [YES] | [YES] | [YES] |
| Issue import/export | [YES] | [YES] (import) | [YES] (migration) | [YES] (migration) | [YES] (migration) |
| Time tracking | [YES] | [YES] | [YES] | [YES] | [YES] |
| Issue dependencies/blocking | [NO] | [YES] (related issues) | [YES] (blocking/blocked by) | [YES] (blocking/blocked by) | [YES] (blocking/blocked by) |
| Confidential issues | [NO] | [YES] (EE only) | [NO] | [NO] | [NO] |
| Issue analytics / boards | [YES] (Projects v2) | [YES] (issue analytics, EE) | [PARTIAL] (basic projects) | [PARTIAL] (basic projects) | [PARTIAL] (basic projects) |
| Kanban boards | [YES] (Projects v2) | [YES] (boards) | [PARTIAL] (basic projects) | [PARTIAL] (basic projects) | [PARTIAL] (basic projects) |
| Issue pinning | [YES] | [YES] | [YES] | [YES] | [YES] |
| Lock discussion | [YES] | [YES] | [YES] | [YES] | [YES] |
| Batch issue handling | [YES] | [YES] | [YES] | [YES] | [YES] |
| Convert comment to issue | [YES] | [YES] | [YES] | [YES] | [YES] |
| Issue search (repo) | [YES] | [YES] | [YES] | [YES] | [YES] |
| Issue search (global) | [YES] | [YES] | [PARTIAL] | [PARTIAL] | [PARTIAL] |
| Create branch from issue | [YES] | [YES] | [NO] | [NO] | [NO] |
| Issue via email | [NO] | [YES] (EE) | [NO] | [PARTIAL] (incoming email) | [NO] |
| Service desk (external tickets) | [NO] | [YES] (EE) | [NO] | [NO] | [NO] |
| Scoped labels (group::label) | [NO] | [YES] | [NO] | [NO] | [NO] |
| Weight / estimate | [YES] (Projects v2) | [YES] | [NO] | [NO] | [NO] |
| Sub-epics / hierarchy | [YES] | [YES] (EE) | [NO] | [NO] | [NO] |
| Issue due dates | [YES] | [YES] | [YES] | [YES] | [YES] |

---

## 3. Pull/Merge Requests

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| PR/MR creation | [YES] | [YES] | [YES] | [YES] | [YES] |
| PR/MR templates | [YES] | [YES] | [YES] | [YES] | [YES] |
| Inline comments | [YES] | [YES] | [YES] | [YES] | [YES] |
| Suggested edits (one-click) | [YES] | [YES] | [NO] | [NO] | [NO] |
| CODEOWNERS enforcement | [YES] | [YES] | [YES] | [YES] | [YES] |
| Status checks | [YES] (Actions) | [YES] (CI pipelines) | [YES] (Actions) | [YES] (Actions) | [YES] (Actions) |
| Merge commit | [YES] | [YES] | [YES] | [YES] | [YES] |
| Squash merge | [YES] | [YES] | [YES] | [YES] | [YES] |
| Rebase merge | [YES] | [YES] | [YES] | [YES] | [YES] |
| Fast-forward merge | [PARTIAL] (via rebase) | [YES] | [YES] | [YES] | [YES] |
| Draft PRs | [YES] | [YES] | [YES] | [YES] | [YES] |
| Conflict detection | [YES] | [YES] | [PARTIAL] (detection only) | [PARTIAL] (detection only) | [PARTIAL] (detection only) |
| In-browser conflict resolution | [YES] | [YES] | [NO] | [NO] | [NO] |
| Review requests | [YES] | [YES] | [YES] | [YES] | [YES] |
| Required reviews | [YES] | [YES] (approval rules) | [YES] | [YES] | [YES] |
| Auto-merge | [YES] | [YES] | [YES] | [YES] | [YES] |
| Merge queue | [YES] | [YES] (EE) | [YES] | [YES] | [YES] |
| Revert commit | [YES] | [YES] | [YES] | [YES] | [YES] |
| Linked issues (auto-close) | [YES] | [YES] | [YES] | [YES] | [YES] |
| PR approval workflow | [YES] | [YES] (approval rules) | [YES] | [YES] | [YES] |
| Cherry-pick changes | [NO] | [YES] | [YES] | [YES] | [YES] |
| Download patch | [YES] | [YES] | [YES] | [YES] | [YES] |
| Multiple reviewers | [YES] | [YES] | [YES] | [YES] | [YES] |
| Review threading | [YES] | [YES] | [YES] | [YES] | [YES] |
| Push to existing PR | [YES] | [YES] | [YES] | [YES] | [YES] |
| Merge message templates | [YES] | [YES] | [YES] | [YES] | [YES] |
| Restrict push/merge to users | [YES] | [YES] | [YES] | [YES] | [YES] |
| AGit / email-based PRs | [NO] | [NO] | [YES] (AGit) | [YES] (AGit) | [YES] (AGit) |
| Merge checklist | [YES] | [YES] | [NO] | [NO] | [NO] |
| Merge request deployments | [NO] | [YES] | [NO] | [NO] | [NO] |
| PR/MR size indicator | [NO] | [YES] (changes tab) | [NO] | [NO] | [NO] |

---

## 4. CI/CD

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Built-in CI/CD | [YES] (Actions) | [YES] (GitLab CI) | [YES] (Gitea Actions) | [YES] (Forgejo Actions) | [YES] (Forgejo Actions) |
| Pipeline YAML config | [YES] (workflow syntax) | [YES] (.gitlab-ci.yml) | [YES] (compatible syntax) | [YES] (compatible syntax) | [YES] (compatible syntax) |
| Artifacts | [YES] | [YES] | [YES] | [YES] | [YES] |
| Caches | [YES] | [YES] | [YES] (runner-side) | [YES] (runner-side) | [YES] (runner-side) |
| Matrix builds | [YES] | [YES] | [YES] | [YES] | [YES] |
| Parallelism | [YES] | [YES] | [YES] | [YES] | [YES] |
| Secrets management | [YES] | [YES] | [YES] | [YES] | [YES] |
| Environments | [YES] | [YES] | [PARTIAL] (basic) | [PARTIAL] (basic) | [PARTIAL] (basic) |
| Deployments | [YES] | [YES] | [PARTIAL] (basic) | [PARTIAL] (basic) | [PARTIAL] (basic) |
| Scheduled runs (cron) | [YES] | [YES] | [YES] | [YES] | [YES] |
| Manual triggers | [YES] (workflow_dispatch) | [YES] (when: manual) | [YES] | [YES] | [YES] |
| Status badges | [YES] | [YES] | [YES] | [YES] | [YES] |
| Container registry (built-in) | [YES] (GHCR) | [YES] | [YES] | [YES] | [YES] |
| Runner management | [YES] (GH-hosted + self) | [YES] (runners mgmt) | [YES] (Gitea Runner) | [YES] (Forgejo Runner) | [YES] (Forgejo Runner) |
| Multi-runner support | [YES] | [YES] | [YES] | [YES] | [YES] |
| Shared runners | [YES] (GH-hosted) | [YES] | [PARTIAL] (instance-level) | [PARTIAL] (instance-level) | [PARTIAL] (instance-level) |
| Protected environments | [YES] | [YES] (EE) | [NO] | [NO] | [NO] |
| Pipeline visualization | [YES] | [YES] | [PARTIAL] (basic) | [PARTIAL] (basic) | [PARTIAL] (basic) |
| Code Quality reports | [NO] | [YES] | [NO] | [NO] | [NO] |
| Auto-cancel redundant runs | [YES] | [YES] | [YES] | [YES] | [YES] |
| Container build in CI | [YES] | [YES] | [PARTIAL] (requires Docker access) | [PARTIAL] (requires Docker access) | [PARTIAL] (requires Docker access) |
| DAG pipelines | [YES] (needs) | [YES] (needs) | [YES] (needs) | [YES] (needs) | [YES] (needs) |
| Workflow artifacts cleanup | [YES] | [YES] | [YES] (configurable retention) | [YES] (configurable retention) | [YES] (configurable retention) |
| OIDC for workload identity | [YES] | [YES] | [YES] | [YES] | [YES] |
| OpenID Connect tokens | [YES] | [YES] | [YES] | [YES] | [YES] |

---

## 5. Wiki / Documentation

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|
| Built-in wiki | [YES] | [YES] | [YES] | [YES] | [YES] |
| Wiki stored as git repo | [YES] | [YES] | [YES] | [YES] | [YES] |
| Sidebar navigation | [YES] | [YES] | [YES] | [YES] | [YES] |
| Page history/revisions | [YES] | [YES] | [YES] | [YES] | [YES] |
| Markdown rendering | [YES] (GFM) | [YES] (GitLab Flavored) | [YES] (GFM-compatible) | [YES] (GFM-compatible) | [YES] (GFM-compatible) |
| TOC auto-generation | [YES] | [YES] | [YES] | [YES] | [YES] |
| Image uploads | [YES] | [YES] | [YES] | [YES] | [YES] |
| Wiki search | [PARTIAL] (repo search) | [YES] | [PARTIAL] | [PARTIAL] | [PARTIAL] |
| Page redirects | [NO] | [NO] | [NO] | [NO] | [NO] |
| Wiki clone | [YES] | [YES] | [YES] | [YES] | [YES] |
| Wiki edit via web | [YES] | [YES] | [YES] | [YES] | [YES] |
| Multiple wikis per repo | [NO] | [NO] | [NO] | [NO] | [NO] |

---

## 6. Authentication

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Username/password | [YES] | [YES] | [YES] | [YES] | [YES] |
| 2FA / TOTP | [YES] | [YES] | [YES] | [YES] | [YES] |
| WebAuthn / FIDO2 | [YES] | [YES] | [YES] | [YES] | [YES] |
| Passkeys | [YES] | [YES] | [PARTIAL] | [PARTIAL] | [PARTIAL] |
| SAML SSO | [PAID] (Enterprise) | [YES] (EE) | [NO] | [NO] | [NO] |
| OpenID Connect (login) | [YES] | [YES] | [YES] | [YES] | [YES] |
| LDAP / AD | [PAID] (Enterprise) | [YES] | [YES] | [YES] | [NO] (public instance) |
| Multiple LDAP sources | [PAID] | [YES] (EE) | [YES] | [YES] | [NO] |
| LDAP user sync | [PAID] | [YES] | [YES] | [YES] | [NO] |
| PAM authentication | [NO] | [NO] | [YES] (build flag) | [YES] (build flag) | [NO] |
| FreeIPA support | [NO] | [NO] | [YES] | [YES] | [NO] |
| Email verification | [YES] | [YES] | [YES] | [YES] | [YES] |
| Account lockout | [YES] | [YES] | [YES] | [YES] | [YES] |
| Password policies | [YES] (Enterprise) | [YES] | [YES] | [YES] | [YES] |
| Org-level 2FA enforcement | [YES] | [YES] (EE) | [PARTIAL] | [PARTIAL] | [PARTIAL] |
| OAuth2 provider | [YES] | [YES] | [YES] | [YES] | [YES] |
| SCIM provisioning | [YES] (Enterprise) | [YES] (EE) | [NO] | [NO] | [NO] |

---

## 7. Repository Management

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Topics/tags | [YES] | [YES] | [YES] | [YES] | [YES] |
| Transfer ownership | [YES] | [YES] | [YES] | [YES] | [YES] |
| Archive repo | [YES] | [YES] | [YES] | [YES] | [YES] |
| Rename repo | [YES] | [YES] | [YES] | [YES] | [YES] |
| Push mirror | [YES] | [YES] | [YES] | [YES] | [YES] |
| Pull mirror | [NO] | [YES] (EE) | [YES] | [YES] | [YES] |
| Default branch setting | [YES] | [YES] | [YES] | [YES] | [YES] |
| Branch protection rules | [YES] | [YES] | [YES] | [YES] | [YES] |
| Tag protection rules | [YES] | [NO] | [YES] | [YES] | [YES] |
| Required reviews | [YES] | [YES] (approval rules) | [YES] | [YES] | [YES] |
| Signed commit verification (GPG) | [YES] | [YES] | [YES] | [YES] | [YES] |
| Signed commit verification (SSH) | [YES] | [NO] | [YES] | [YES] | [YES] |
| Reject unsigned commits | [YES] | [YES] | [YES] | [YES] | [YES] |
| Verified committer badge | [YES] | [YES] | [PARTIAL] | [PARTIAL] | [PARTIAL] |
| Push rules | [NO] | [YES] (EE) | [PARTIAL] (branch protection) | [PARTIAL] (branch protection) | [PARTIAL] (branch protection) |
| Repository fork | [YES] | [YES] | [YES] | [YES] | [YES] |
| Template repositories | [YES] | [YES] | [YES] | [YES] | [YES] |
| Issue/PRL transfer between repos | [YES] | [YES] | [NO] | [NO] | [NO] |
| Repo activity page | [YES] | [YES] | [YES] | [YES] | [YES] |
| Soft quota (repo size limits) | [NO] | [YES] | [PARTIAL] | [YES] | [YES] |

---

## 8. Organizations / Teams

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Organization creation | [YES] | [YES] | [YES] | [YES] | [YES] |
| Team management | [YES] | [YES] | [YES] | [YES] | [YES] |
| Team-level permissions | [YES] (fine-grained) | [YES] | [YES] | [YES] | [YES] |
| Nested teams | [YES] | [YES] (EE) | [NO] | [NO] | [NO] |
| Org-level 2FA | [YES] | [YES] (EE) | [PARTIAL] | [PARTIAL] | [PARTIAL] |
| Audit log | [PAID] (Enterprise) | [YES] (EE) | [PARTIAL] (basic) | [PARTIAL] (basic) | [PARTIAL] (basic) |
| Billing / paid plans | [YES] | [YES] | [NO] (self-hosted) | [NO] (self-hosted) | [NO] (non-profit) |
| Org-level project boards | [YES] | [YES] | [PARTIAL] | [PARTIAL] | [PARTIAL] |
| Org visibility (public/private) | [YES] | [YES] | [YES] | [YES] | [YES] |
| Group/subgroup support | [NO] | [YES] | [NO] | [NO] | [NO] |
| Org membership requests | [YES] | [YES] | [YES] | [YES] | [YES] |
| External collaborators | [YES] | [YES] | [YES] | [YES] | [YES] |
| Organization profile page | [YES] | [YES] | [YES] | [YES] | [YES] |

---

## 9. Federation

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| ActivityPub support | [NO] | [NO] | [NO] | [PARTIAL] (in development) | [PARTIAL] (in development via Forgejo) |
| ForgeFed / NodeInfo | [NO] | [NO] | [NO] | [PARTIAL] (in development) | [PARTIAL] (in development via Forgejo) |
| Remote follow (federated) | [NO] | [NO] | [NO] | [PARTIAL] (planned) | [PARTIAL] (planned) |
| Cross-instance PRs | [NO] | [NO] | [NO] | [PARTIAL] (planned) | [PARTIAL] (planned) |
| Inter-instance interop | [NO] | [NO] | [NO] | [PARTIAL] (roadmap) | [PARTIAL] (roadmap) |
| Federated identity | [NO] | [NO] | [NO] | [PARTIAL] (remote login WIP) | [PARTIAL] (remote login WIP) |

---

## 10. Package Registry

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Container (OCI) | [YES] (GHCR) | [YES] | [YES] | [YES] | [YES] |
| npm | [YES] | [YES] | [YES] | [YES] | [YES] |
| PyPI | [YES] | [YES] | [YES] | [YES] | [YES] |
| Maven | [YES] | [YES] | [YES] | [YES] | [YES] |
| RPM | [NO] | [YES] (EE) | [YES] | [YES] | [YES] |
| Debian | [NO] | [YES] (EE) | [YES] | [YES] | [YES] |
| Go | [YES] | [YES] | [YES] | [YES] | [YES] |
| Composer | [NO] | [YES] | [YES] | [YES] | [YES] |
| NuGet | [YES] | [YES] | [YES] | [YES] | [YES] |
| Generic packages | [NO] | [YES] | [YES] | [YES] | [YES] |
| Alpine | [NO] | [NO] | [YES] | [YES] | [YES] |
| Arch | [NO] | [NO] | [YES] | [YES] | [YES] |
| Cargo (Rust) | [NO] | [NO] | [YES] | [YES] | [YES] |
| Chef | [NO] | [NO] | [YES] | [YES] | [YES] |
| Conan (C++) | [NO] | [NO] | [YES] | [YES] | [YES] |
| Conda | [NO] | [NO] | [YES] | [YES] | [YES] |
| CRAN (R) | [NO] | [NO] | [YES] | [YES] | [YES] |
| Helm Charts | [YES] | [YES] | [YES] | [YES] | [YES] |
| Pub (Dart) | [NO] | [NO] | [YES] | [YES] | [YES] |
| RubyGems | [NO] | [YES] | [YES] | [YES] | [YES] |
| Swift | [NO] | [NO] | [YES] | [YES] | [YES] |
| Vagrant | [NO] | [NO] | [YES] | [YES] | [YES] |
| Terraform State | [NO] | [NO] | [YES] | [NO] | [NO] |
| Package cleanup rules | [NO] | [PARTIAL] | [YES] | [YES] | [YES] |
| Package deduplication | [YES] | [YES] | [YES] | [YES] | [YES] |
| Package-link to repo | [YES] | [YES] | [YES] | [YES] | [YES] |

---

## 11. Security

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Secret scanning | [YES] (Advanced Security) | [YES] (secret detection) | [NO] | [NO] | [NO] |
| Dependency scanning | [YES] (Dependabot alerts) | [YES] (dependency scanning) | [NO] | [NO] | [NO] |
| Dependabot version updates | [YES] | [NO] | [NO] | [NO] | [NO] |
| SAST (Static Analysis) | [YES] (code scanning) | [YES] (SAST) | [NO] | [NO] | [NO] |
| CODEOWNERS | [YES] | [YES] | [YES] | [YES] | [YES] |
| Branch protection | [YES] | [YES] | [YES] | [YES] | [YES] |
| GPG/SSH key management | [YES] | [YES] | [YES] | [YES] | [YES] |
| SBOM generation | [PARTIAL] (Dependabot) | [YES] (SBOM) | [NO] | [NO] | [NO] |
| SLSA provenance | [YES] | [YES] | [NO] | [NO] | [NO] |
| Token scanning partnerships | [YES] | [YES] | [NO] | [NO] | [NO] |
| Push rules (commit restrictions) | [NO] | [YES] (EE) | [PARTIAL] (branch protection) | [PARTIAL] (branch protection) | [PARTIAL] (branch protection) |
| DAST | [YES] (Advanced Security) | [YES] (EE) | [NO] | [NO] | [NO] |
| Container scanning | [YES] | [YES] | [NO] | [NO] | [NO] |
| License compliance | [YES] (Advanced Security) | [YES] (EE) | [NO] | [NO] | [NO] |
| Security policy files | [YES] (SECURITY.md) | [YES] | [YES] | [YES] | [YES] |
| Vulnerability reporting | [YES] | [YES] | [YES] | [YES] | [YES] |
| Private vulnerability reports | [YES] | [YES] (EE) | [YES] | [YES] | [YES] |

---

## 12. Collaboration

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Notifications (email/in-app) | [YES] | [YES] | [YES] | [YES] | [YES] |
| Watch/star/fork | [YES] | [YES] | [YES] | [YES] | [YES] |
| @mentions | [YES] | [YES] | [YES] | [YES] | [YES] |
| Emoji reactions | [YES] | [YES] | [YES] | [YES] | [YES] |
| Task lists in markdown | [YES] (- [ ]) | [YES] | [YES] | [YES] | [YES] |
| Collaborative / real-time editing | [NO] | [PARTIAL] (real-time in Web IDE) | [NO] | [NO] | [NO] |
| Concurrent editing | [NO] | [NO] | [NO] | [NO] | [NO] |
| RSS feeds | [YES] | [NO] | [YES] | [YES] | [YES] |
| User blocking | [YES] | [YES] | [YES] | [YES] | [YES] |
| Auto-linked references | [YES] | [YES] | [YES] | [YES] | [YES] |
| Pin issues/PRs | [YES] | [YES] | [YES] | [YES] | [YES] |
| Issue assignments | [YES] | [YES] | [YES] | [YES] | [YES] |
| Label system | [YES] | [YES] | [YES] | [YES] | [YES] |

---

## 13. API

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| REST API | [YES] (comprehensive) | [YES] (comprehensive) | [YES] (comprehensive) | [YES] (comprehensive) | [YES] (comprehensive) |
| GraphQL | [YES] | [YES] | [NO] | [NO] | [NO] |
| SSH over HTTP port | [YES] (443) | [YES] | [YES] | [YES] | [YES] |
| Git protocol (git://) | [YES] | [YES] | [YES] | [YES] | [YES] |
| Smart HTTP (HTTPS git) | [YES] | [YES] | [YES] | [YES] | [YES] |
| Webhooks | [YES] | [YES] | [YES] | [YES] | [YES] |
| OAuth2 applications | [YES] | [YES] | [YES] | [YES] | [YES] |
| Personal access tokens | [YES] (fine-grained) | [YES] (scopes) | [YES] (scoped) | [YES] (scoped) | [YES] (scoped) |
| Machine users / bots | [YES] (GitHub App) | [YES] (bot users) | [PARTIAL] (access tokens) | [PARTIAL] (access tokens) | [PARTIAL] (access tokens) |
| Rate limiting | [YES] | [YES] | [YES] | [YES] | [YES] |
| API docs / Swagger | [YES] (OpenAPI) | [YES] (OpenAPI) | [YES] (Swagger) | [YES] (Swagger) | [YES] (Swagger) |
| Git hooks (pre-receive) | [YES] | [YES] | [YES] | [YES] | [YES] |
| API versioning | [YES] | [YES] | [YES] | [YES] | [YES] |

---

## 14. Search

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Global search | [YES] | [YES] | [YES] | [YES] | [YES] |
| Code search (repo) | [YES] | [YES] | [YES] (basic) | [YES] (basic) | [YES] (basic) |
| Code search (global) | [YES] | [YES] (EE) | [YES] (with indexer) | [YES] (with indexer) | [PARTIAL] (instance-dependent) |
| Repository search | [YES] | [YES] | [YES] | [YES] | [YES] |
| User search | [YES] | [YES] | [YES] | [YES] | [YES] |
| Issue search | [YES] | [YES] | [YES] | [YES] | [YES] |
| Advanced filters / query syntax | [YES] (very powerful) | [YES] | [PARTIAL] (basic filters) | [PARTIAL] (basic filters) | [PARTIAL] (basic filters) |
| Regex code search | [YES] | [YES] | [PARTIAL] (depends on indexer) | [PARTIAL] (depends on indexer) | [PARTIAL] |
| Semantic code search | [YES] (code search v2) | [NO] | [NO] | [NO] | [NO] |
| Search across organizations | [YES] | [YES] | [YES] | [YES] | [YES] |

---

## 15. UI/UX

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Dark mode | [YES] | [YES] | [YES] | [YES] | [YES] |
| Light mode | [YES] | [YES] | [YES] | [YES] | [YES] |
| Responsive / mobile-friendly | [YES] | [YES] | [YES] | [YES] | [YES] |
| Keyboard shortcuts | [YES] (extensive) | [YES] | [YES] | [YES] | [YES] |
| Accessibility (WCAG) | [PARTIAL] (ongoing) | [PARTIAL] (ongoing) | [PARTIAL] | [PARTIAL] | [PARTIAL] |
| Custom themes | [NO] (limited) | [PARTIAL] (branding) | [YES] (custom CSS) | [YES] (custom CSS) | [PARTIAL] (Forgejo default) |
| Notifications UI | [YES] (inbox) | [YES] (inbox) | [YES] (dashboard) | [YES] (dashboard) | [YES] (dashboard) |
| Activity timeline | [YES] | [YES] | [YES] | [YES] | [YES] |
| Custom CSS injection | [NO] | [PARTIAL] (admin) | [YES] | [YES] | [NO] (public instance) |
| Custom JS injection | [NO] | [PARTIAL] (admin) | [YES] | [YES] | [NO] (public instance) |
| Custom footer/logo | [NO] | [YES] | [YES] | [YES] | [NO] (public instance) |
| User profile customization | [YES] | [YES] | [YES] | [YES] | [YES] |
| Multi-language UI | [YES] | [YES] | [YES] (i18n) | [YES] (i18n) | [YES] (i18n) |
| Dashboard / feed | [YES] | [YES] | [YES] | [YES] | [YES] |

---

## 16. Desktop / Apps

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Native desktop app | [YES] (GitHub Desktop) | [NO] (third-party) | [NO] | [NO] | [NO] |
| Mobile apps (iOS/Android) | [NO] | [NO] | [NO] | [NO] | [NO] |
| CLI tool | [YES] (gh CLI) | [YES] (glab CLI) | [YES] (tea CLI) | [YES] (forgejo-cli / tea) | [NO] (use tea) |
| VS Code extension | [YES] (GitHub Pull Requests) | [YES] (GitLab Workflow) | [PARTIAL] (community) | [PARTIAL] (community) | [PARTIAL] (community) |
| JetBrains plugin | [YES] | [YES] | [PARTIAL] (community) | [PARTIAL] (community) | [PARTIAL] (community) |
| Mobile-responsive web | [YES] | [YES] | [YES] | [YES] | [YES] |
| Codespaces / Workspaces | [YES] (GitHub Codespaces) | [YES] (GitLab Workspaces) | [NO] | [NO] | [NO] |

---

## 17. AI Features

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| AI code suggestions (Copilot-like) | [YES] (GitHub Copilot) | [YES] (GitLab Duo) | [NO] | [NO] | [NO] |
| AI PR review | [YES] (Copilot) | [YES] (Duo Review) | [NO] | [NO] | [NO] |
| AI code search | [YES] (Copilot) | [YES] (Duo Chat) | [NO] | [NO] | [NO] |
| AI chat assistant | [YES] (Copilot Chat) | [YES] (Duo Chat) | [NO] | [NO] | [NO] |
| AI code generation | [YES] (Copilot) | [YES] (Duo) | [NO] | [NO] | [NO] |
| Vulnerability explanation (AI) | [YES] | [YES] (Duo) | [NO] | [NO] | [NO] |
| Root cause analysis (AI) | [NO] | [YES] (Duo) | [NO] | [NO] | [NO] |
| AI impact analytics | [NO] | [YES] (Duo SDLC) | [NO] | [NO] | [NO] |

---

## 18. Other

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Activity feed (global) | [YES] | [YES] | [YES] | [YES] | [YES] |
| Dashboard | [YES] | [YES] | [YES] | [YES] | [YES] |
| Grafana integration | [NO] | [YES] | [NO] | [NO] | [NO] |
| Labels system | [YES] | [YES] | [YES] | [YES] | [YES] |
| Issue templates | [YES] | [YES] | [YES] | [YES] | [YES] |
| Release management | [YES] | [YES] | [YES] | [YES] | [YES] |
| Changelog generation | [NO] (via Actions) | [YES] (EE) | [NO] | [NO] | [NO] |
| GPG signing (instance) | [YES] | [YES] | [YES] | [YES] | [YES] |
| SSH key management | [YES] | [YES] | [YES] | [YES] | [YES] |
| Deploy keys | [YES] | [YES] | [YES] | [YES] | [YES] |
| Deploy tokens | [YES] | [YES] | [YES] | [YES] | [YES] |
| Repository tokens (write) | [YES] | [YES] | [YES] | [YES] | [YES] |
| Project boards / Kanban | [YES] (Projects v2) | [YES] (boards) | [PARTIAL] (basic) | [PARTIAL] (basic) | [PARTIAL] (basic) |
| Pages (static site hosting) | [YES] (GitHub Pages) | [YES] (GitLab Pages) | [BE] (third-party) | [BE] (Codeberg Pages) | [YES] (Codeberg Pages) |
| Snippets / Gists | [YES] | [YES] (Snippets) | [BE] (OpenGist) | [BE] (OpenGist) | [BE] (OpenGist) |
| Moderation tools | [YES] | [YES] | [YES] | [YES] | [YES] |
| Import from GitHub/GitLab/etc | [YES] | [YES] | [YES] | [YES] | [YES] |
| Free/open source | [NO] (proprietary) | [NO] (Open Core) | [PARTIAL] (CE free, EE paid) | [YES] (AGPL) | [YES] (via Forgejo) |
| Low resource usage | [NO] | [NO] | [YES] | [YES] | [YES] |
| Self-hosted | [YES] (Enterprise Server) | [YES] | [YES] | [YES] | [NO] (public instance) |
| Public hosted instance | [YES] (github.com) | [YES] (gitlab.com) | [YES] (gitea.com) | [YES] (codeberg.org) | [YES] (codeberg.org) |

---

## Summary: Commonly Expected Features That Are Missing

### GitHub
- No subgroups / nested org structure
- No package registry for some formats (Alpine, Arch, Cargo, RPM, Debian)
- No in-browser merge conflict resolution on mobile
- No native SAML on free tier
- No free LDAP on free tier
- No built-in time tracking (in Issues natively — removed; now needs Projects)
- No free real-time collaborative editing
- No federation

### GitLab
- No GitHub-compatible CI syntax (uses `.gitlab-ci.yml`)
- No passkey-specific support (uses WebAuthn)
- Free tier lacks: multiple issue assignees (CE), approval rules (CE), push mirrors (CE), SAML
- No RSS feeds
- No federation
- Higher resource requirements

### Gitea
- No SAML SSO
- No nested teams / subgroups
- No group milestones
- No confidential issues
- No in-browser conflict resolution
- No suggested edits in PRs
- No code search (advanced/regex) without indexer
- No GraphQL API
- No native AI features
- No security scanning (SAST, dependency scanning, secret scanning)
- No federation
- Limited project management (basic kanban only)
- No merge checklist
- Pages requires third-party server
- **Open Core** (some cloud features not in CE)

### Forgejo
- Same gaps as Gitea (forked from it)
- Federation is in active development but not yet stable
- No SAML SSO
- No nested teams
- No confidential issues
- No in-browser conflict resolution
- No GraphQL API
- No security scanning tooling
- No AI features
- 100% Free Software (AGPL v3)

### Codeberg
- Same feature gaps as Forgejo (runs Forgejo)
- No LDAP (public instance limitation)
- No SAML
- No custom themes/CSS (public instance)
- No deploy tokens to external systems
- No third-party CI integration (Woodpecker available but separate)
- Limited resources (donation-funded)
- Federation still in development
