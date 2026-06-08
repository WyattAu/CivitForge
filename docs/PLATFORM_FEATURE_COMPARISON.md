# Platform Feature Comparison

GitHub vs GitLab vs Gitea vs Forgejo vs Codeberg

**Legend:** ✅ Supported | ❌ Not supported | ⚠️ Partial support | 🔧 Via plugin/3rd party | 💰 Requires paid tier

---

## 1. Source Code

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Code browsing | ✅ | ✅ | ✅ | ✅ | ✅ |
| Git blame view | ✅ | ✅ | ✅ | ✅ | ✅ |
| Commit history per file | ✅ | ✅ | ✅ | ✅ | ✅ |
| Inline diff view | ✅ | ✅ | ✅ | ✅ | ✅ |
| Side-by-side diff | ✅ | ✅ | ✅ | ✅ | ✅ |
| Syntax highlighting | ✅ (rich, 100+ langs) | ✅ (rich, Rouge) | ✅ (Chroma, many langs) | ✅ (Chroma) | ✅ (Chroma) |
| Last commit column | ✅ | ✅ | ✅ | ✅ | ✅ |
| File finder (`.` or `t`) | ✅ | ✅ | ⚠️ (basic tree nav) | ⚠️ (basic tree nav) | ⚠️ (basic tree nav) |
| Search within repo | ✅ | ✅ | ✅ | ✅ | ✅ |
| Branch switching in UI | ✅ | ✅ | ✅ | ✅ | ✅ |
| Submodules support | ✅ | ✅ | ✅ | ✅ | ✅ |
| LFS 2.0 | ✅ | ✅ | ✅ | ✅ | ✅ |
| Raw file download | ✅ | ✅ | ✅ | ✅ | ✅ |
| Archive download (zip) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Archive download (tar.gz) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Archive download (tar.bz2) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Web code editor | ✅ | ✅ | ✅ | ✅ | ✅ |
| Commit graph visualization | ✅ | ✅ | ✅ | ✅ | ✅ |
| Visual image diff | ✅ | ✅ | ✅ | ✅ | ✅ |
| Color-coded blame | ✅ | ✅ | ✅ | ✅ | ✅ |
| Repository size display | ✅ | ✅ | ✅ | ✅ | ✅ |
| Language detection/stats | ✅ | ✅ | ✅ | ✅ | ✅ |
| README rendering at repo root | ✅ | ✅ | ✅ | ✅ | ✅ |
| Mermaid diagrams in markdown | ✅ | ✅ | ✅ | ✅ | ✅ |
| Math (LaTeX/KaTeX) in markdown | ✅ | ✅ | ✅ | ✅ | ✅ |
| CSV rendering | ✅ | ✅ | ✅ | ✅ | ✅ |
| 3D model viewer (.stl) | ✅ | ❌ | ❌ | ❌ | ❌ |
| Jupyter notebook rendering | ✅ | ✅ | ❌ | ❌ | ❌ |
| PDF rendering in browser | ✅ | ❌ | ❌ | ❌ | ❌ |

---

## 2. Issues

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Issue tracker | ✅ | ✅ | ✅ | ✅ | ✅ |
| Issue templates | ✅ | ✅ | ✅ | ✅ | ✅ |
| Labels (color-coded) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Milestones | ✅ | ✅ | ✅ | ✅ | ✅ |
| Group/org milestones | ❌ | ✅ | ❌ | ❌ | ❌ |
| Assignees (single/multiple) | ✅ | ✅ (multiple, EE) | ✅ (multiple) | ✅ (multiple) | ✅ (multiple) |
| Emoji reactions | ✅ | ✅ | ✅ | ✅ | ✅ |
| File attachments | ✅ | ✅ | ✅ | ✅ | ✅ |
| Cross-references (auto-link) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Issue import/export | ✅ | ✅ (import) | ✅ (migration) | ✅ (migration) | ✅ (migration) |
| Time tracking | ✅ | ✅ | ✅ | ✅ | ✅ |
| Issue dependencies/blocking | ❌ | ✅ (related issues) | ✅ (blocking/blocked by) | ✅ (blocking/blocked by) | ✅ (blocking/blocked by) |
| Confidential issues | ❌ | ✅ (EE only) | ❌ | ❌ | ❌ |
| Issue analytics / boards | ✅ (Projects v2) | ✅ (issue analytics, EE) | ⚠️ (basic projects) | ⚠️ (basic projects) | ⚠️ (basic projects) |
| Kanban boards | ✅ (Projects v2) | ✅ (boards) | ⚠️ (basic projects) | ⚠️ (basic projects) | ⚠️ (basic projects) |
| Issue pinning | ✅ | ✅ | ✅ | ✅ | ✅ |
| Lock discussion | ✅ | ✅ | ✅ | ✅ | ✅ |
| Batch issue handling | ✅ | ✅ | ✅ | ✅ | ✅ |
| Convert comment to issue | ✅ | ✅ | ✅ | ✅ | ✅ |
| Issue search (repo) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Issue search (global) | ✅ | ✅ | ⚠️ | ⚠️ | ⚠️ |
| Create branch from issue | ✅ | ✅ | ❌ | ❌ | ❌ |
| Issue via email | ❌ | ✅ (EE) | ❌ | ⚠️ (incoming email) | ❌ |
| Service desk (external tickets) | ❌ | ✅ (EE) | ❌ | ❌ | ❌ |
| Scoped labels (group::label) | ❌ | ✅ | ❌ | ❌ | ❌ |
| Weight / estimate | ✅ (Projects v2) | ✅ | ❌ | ❌ | ❌ |
| Sub-epics / hierarchy | ✅ | ✅ (EE) | ❌ | ❌ | ❌ |
| Issue due dates | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## 3. Pull/Merge Requests

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| PR/MR creation | ✅ | ✅ | ✅ | ✅ | ✅ |
| PR/MR templates | ✅ | ✅ | ✅ | ✅ | ✅ |
| Inline comments | ✅ | ✅ | ✅ | ✅ | ✅ |
| Suggested edits (one-click) | ✅ | ✅ | ❌ | ❌ | ❌ |
| CODEOWNERS enforcement | ✅ | ✅ | ✅ | ✅ | ✅ |
| Status checks | ✅ (Actions) | ✅ (CI pipelines) | ✅ (Actions) | ✅ (Actions) | ✅ (Actions) |
| Merge commit | ✅ | ✅ | ✅ | ✅ | ✅ |
| Squash merge | ✅ | ✅ | ✅ | ✅ | ✅ |
| Rebase merge | ✅ | ✅ | ✅ | ✅ | ✅ |
| Fast-forward merge | ⚠️ (via rebase) | ✅ | ✅ | ✅ | ✅ |
| Draft PRs | ✅ | ✅ | ✅ | ✅ | ✅ |
| Conflict detection | ✅ | ✅ | ⚠️ (detection only) | ⚠️ (detection only) | ⚠️ (detection only) |
| In-browser conflict resolution | ✅ | ✅ | ❌ | ❌ | ❌ |
| Review requests | ✅ | ✅ | ✅ | ✅ | ✅ |
| Required reviews | ✅ | ✅ (approval rules) | ✅ | ✅ | ✅ |
| Auto-merge | ✅ | ✅ | ✅ | ✅ | ✅ |
| Merge queue | ✅ | ✅ (EE) | ✅ | ✅ | ✅ |
| Revert commit | ✅ | ✅ | ✅ | ✅ | ✅ |
| Linked issues (auto-close) | ✅ | ✅ | ✅ | ✅ | ✅ |
| PR approval workflow | ✅ | ✅ (approval rules) | ✅ | ✅ | ✅ |
| Cherry-pick changes | ❌ | ✅ | ✅ | ✅ | ✅ |
| Download patch | ✅ | ✅ | ✅ | ✅ | ✅ |
| Multiple reviewers | ✅ | ✅ | ✅ | ✅ | ✅ |
| Review threading | ✅ | ✅ | ✅ | ✅ | ✅ |
| Push to existing PR | ✅ | ✅ | ✅ | ✅ | ✅ |
| Merge message templates | ✅ | ✅ | ✅ | ✅ | ✅ |
| Restrict push/merge to users | ✅ | ✅ | ✅ | ✅ | ✅ |
| AGit / email-based PRs | ❌ | ❌ | ✅ (AGit) | ✅ (AGit) | ✅ (AGit) |
| Merge checklist | ✅ | ✅ | ❌ | ❌ | ❌ |
| Merge request deployments | ❌ | ✅ | ❌ | ❌ | ❌ |
| PR/MR size indicator | ❌ | ✅ (changes tab) | ❌ | ❌ | ❌ |

---

## 4. CI/CD

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Built-in CI/CD | ✅ (Actions) | ✅ (GitLab CI) | ✅ (Gitea Actions) | ✅ (Forgejo Actions) | ✅ (Forgejo Actions) |
| Pipeline YAML config | ✅ (workflow syntax) | ✅ (.gitlab-ci.yml) | ✅ (compatible syntax) | ✅ (compatible syntax) | ✅ (compatible syntax) |
| Artifacts | ✅ | ✅ | ✅ | ✅ | ✅ |
| Caches | ✅ | ✅ | ✅ (runner-side) | ✅ (runner-side) | ✅ (runner-side) |
| Matrix builds | ✅ | ✅ | ✅ | ✅ | ✅ |
| Parallelism | ✅ | ✅ | ✅ | ✅ | ✅ |
| Secrets management | ✅ | ✅ | ✅ | ✅ | ✅ |
| Environments | ✅ | ✅ | ⚠️ (basic) | ⚠️ (basic) | ⚠️ (basic) |
| Deployments | ✅ | ✅ | ⚠️ (basic) | ⚠️ (basic) | ⚠️ (basic) |
| Scheduled runs (cron) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Manual triggers | ✅ (workflow_dispatch) | ✅ (when: manual) | ✅ | ✅ | ✅ |
| Status badges | ✅ | ✅ | ✅ | ✅ | ✅ |
| Container registry (built-in) | ✅ (GHCR) | ✅ | ✅ | ✅ | ✅ |
| Runner management | ✅ (GH-hosted + self) | ✅ (runners mgmt) | ✅ (Gitea Runner) | ✅ (Forgejo Runner) | ✅ (Forgejo Runner) |
| Multi-runner support | ✅ | ✅ | ✅ | ✅ | ✅ |
| Shared runners | ✅ (GH-hosted) | ✅ | ⚠️ (instance-level) | ⚠️ (instance-level) | ⚠️ (instance-level) |
| Protected environments | ✅ | ✅ (EE) | ❌ | ❌ | ❌ |
| Pipeline visualization | ✅ | ✅ | ⚠️ (basic) | ⚠️ (basic) | ⚠️ (basic) |
| Code Quality reports | ❌ | ✅ | ❌ | ❌ | ❌ |
| Auto-cancel redundant runs | ✅ | ✅ | ✅ | ✅ | ✅ |
| Container build in CI | ✅ | ✅ | ⚠️ (requires Docker access) | ⚠️ (requires Docker access) | ⚠️ (requires Docker access) |
| DAG pipelines | ✅ (needs) | ✅ (needs) | ✅ (needs) | ✅ (needs) | ✅ (needs) |
| Workflow artifacts cleanup | ✅ | ✅ | ✅ (configurable retention) | ✅ (configurable retention) | ✅ (configurable retention) |
| OIDC for workload identity | ✅ | ✅ | ✅ | ✅ | ✅ |
| OpenID Connect tokens | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## 5. Wiki / Documentation

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|
| Built-in wiki | ✅ | ✅ | ✅ | ✅ | ✅ |
| Wiki stored as git repo | ✅ | ✅ | ✅ | ✅ | ✅ |
| Sidebar navigation | ✅ | ✅ | ✅ | ✅ | ✅ |
| Page history/revisions | ✅ | ✅ | ✅ | ✅ | ✅ |
| Markdown rendering | ✅ (GFM) | ✅ (GitLab Flavored) | ✅ (GFM-compatible) | ✅ (GFM-compatible) | ✅ (GFM-compatible) |
| TOC auto-generation | ✅ | ✅ | ✅ | ✅ | ✅ |
| Image uploads | ✅ | ✅ | ✅ | ✅ | ✅ |
| Wiki search | ⚠️ (repo search) | ✅ | ⚠️ | ⚠️ | ⚠️ |
| Page redirects | ❌ | ❌ | ❌ | ❌ | ❌ |
| Wiki clone | ✅ | ✅ | ✅ | ✅ | ✅ |
| Wiki edit via web | ✅ | ✅ | ✅ | ✅ | ✅ |
| Multiple wikis per repo | ❌ | ❌ | ❌ | ❌ | ❌ |

---

## 6. Authentication

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Username/password | ✅ | ✅ | ✅ | ✅ | ✅ |
| 2FA / TOTP | ✅ | ✅ | ✅ | ✅ | ✅ |
| WebAuthn / FIDO2 | ✅ | ✅ | ✅ | ✅ | ✅ |
| Passkeys | ✅ | ✅ | ⚠️ | ⚠️ | ⚠️ |
| SAML SSO | 💰 (Enterprise) | ✅ (EE) | ❌ | ❌ | ❌ |
| OpenID Connect (login) | ✅ | ✅ | ✅ | ✅ | ✅ |
| LDAP / AD | 💰 (Enterprise) | ✅ | ✅ | ✅ | ❌ (public instance) |
| Multiple LDAP sources | 💰 | ✅ (EE) | ✅ | ✅ | ❌ |
| LDAP user sync | 💰 | ✅ | ✅ | ✅ | ❌ |
| PAM authentication | ❌ | ❌ | ✅ (build flag) | ✅ (build flag) | ❌ |
| FreeIPA support | ❌ | ❌ | ✅ | ✅ | ❌ |
| Email verification | ✅ | ✅ | ✅ | ✅ | ✅ |
| Account lockout | ✅ | ✅ | ✅ | ✅ | ✅ |
| Password policies | ✅ (Enterprise) | ✅ | ✅ | ✅ | ✅ |
| Org-level 2FA enforcement | ✅ | ✅ (EE) | ⚠️ | ⚠️ | ⚠️ |
| OAuth2 provider | ✅ | ✅ | ✅ | ✅ | ✅ |
| SCIM provisioning | ✅ (Enterprise) | ✅ (EE) | ❌ | ❌ | ❌ |

---

## 7. Repository Management

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Topics/tags | ✅ | ✅ | ✅ | ✅ | ✅ |
| Transfer ownership | ✅ | ✅ | ✅ | ✅ | ✅ |
| Archive repo | ✅ | ✅ | ✅ | ✅ | ✅ |
| Rename repo | ✅ | ✅ | ✅ | ✅ | ✅ |
| Push mirror | ✅ | ✅ | ✅ | ✅ | ✅ |
| Pull mirror | ❌ | ✅ (EE) | ✅ | ✅ | ✅ |
| Default branch setting | ✅ | ✅ | ✅ | ✅ | ✅ |
| Branch protection rules | ✅ | ✅ | ✅ | ✅ | ✅ |
| Tag protection rules | ✅ | ❌ | ✅ | ✅ | ✅ |
| Required reviews | ✅ | ✅ (approval rules) | ✅ | ✅ | ✅ |
| Signed commit verification (GPG) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Signed commit verification (SSH) | ✅ | ❌ | ✅ | ✅ | ✅ |
| Reject unsigned commits | ✅ | ✅ | ✅ | ✅ | ✅ |
| Verified committer badge | ✅ | ✅ | ⚠️ | ⚠️ | ⚠️ |
| Push rules | ❌ | ✅ (EE) | ⚠️ (branch protection) | ⚠️ (branch protection) | ⚠️ (branch protection) |
| Repository fork | ✅ | ✅ | ✅ | ✅ | ✅ |
| Template repositories | ✅ | ✅ | ✅ | ✅ | ✅ |
| Issue/PRL transfer between repos | ✅ | ✅ | ❌ | ❌ | ❌ |
| Repo activity page | ✅ | ✅ | ✅ | ✅ | ✅ |
| Soft quota (repo size limits) | ❌ | ✅ | ⚠️ | ✅ | ✅ |

---

## 8. Organizations / Teams

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Organization creation | ✅ | ✅ | ✅ | ✅ | ✅ |
| Team management | ✅ | ✅ | ✅ | ✅ | ✅ |
| Team-level permissions | ✅ (fine-grained) | ✅ | ✅ | ✅ | ✅ |
| Nested teams | ✅ | ✅ (EE) | ❌ | ❌ | ❌ |
| Org-level 2FA | ✅ | ✅ (EE) | ⚠️ | ⚠️ | ⚠️ |
| Audit log | 💰 (Enterprise) | ✅ (EE) | ⚠️ (basic) | ⚠️ (basic) | ⚠️ (basic) |
| Billing / paid plans | ✅ | ✅ | ❌ (self-hosted) | ❌ (self-hosted) | ❌ (non-profit) |
| Org-level project boards | ✅ | ✅ | ⚠️ | ⚠️ | ⚠️ |
| Org visibility (public/private) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Group/subgroup support | ❌ | ✅ | ❌ | ❌ | ❌ |
| Org membership requests | ✅ | ✅ | ✅ | ✅ | ✅ |
| External collaborators | ✅ | ✅ | ✅ | ✅ | ✅ |
| Organization profile page | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## 9. Federation

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| ActivityPub support | ❌ | ❌ | ❌ | ⚠️ (in development) | ⚠️ (in development via Forgejo) |
| ForgeFed / NodeInfo | ❌ | ❌ | ❌ | ⚠️ (in development) | ⚠️ (in development via Forgejo) |
| Remote follow (federated) | ❌ | ❌ | ❌ | ⚠️ (planned) | ⚠️ (planned) |
| Cross-instance PRs | ❌ | ❌ | ❌ | ⚠️ (planned) | ⚠️ (planned) |
| Inter-instance interop | ❌ | ❌ | ❌ | ⚠️ (roadmap) | ⚠️ (roadmap) |
| Federated identity | ❌ | ❌ | ❌ | ⚠️ (remote login WIP) | ⚠️ (remote login WIP) |

---

## 10. Package Registry

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Container (OCI) | ✅ (GHCR) | ✅ | ✅ | ✅ | ✅ |
| npm | ✅ | ✅ | ✅ | ✅ | ✅ |
| PyPI | ✅ | ✅ | ✅ | ✅ | ✅ |
| Maven | ✅ | ✅ | ✅ | ✅ | ✅ |
| RPM | ❌ | ✅ (EE) | ✅ | ✅ | ✅ |
| Debian | ❌ | ✅ (EE) | ✅ | ✅ | ✅ |
| Go | ✅ | ✅ | ✅ | ✅ | ✅ |
| Composer | ❌ | ✅ | ✅ | ✅ | ✅ |
| NuGet | ✅ | ✅ | ✅ | ✅ | ✅ |
| Generic packages | ❌ | ✅ | ✅ | ✅ | ✅ |
| Alpine | ❌ | ❌ | ✅ | ✅ | ✅ |
| Arch | ❌ | ❌ | ✅ | ✅ | ✅ |
| Cargo (Rust) | ❌ | ❌ | ✅ | ✅ | ✅ |
| Chef | ❌ | ❌ | ✅ | ✅ | ✅ |
| Conan (C++) | ❌ | ❌ | ✅ | ✅ | ✅ |
| Conda | ❌ | ❌ | ✅ | ✅ | ✅ |
| CRAN (R) | ❌ | ❌ | ✅ | ✅ | ✅ |
| Helm Charts | ✅ | ✅ | ✅ | ✅ | ✅ |
| Pub (Dart) | ❌ | ❌ | ✅ | ✅ | ✅ |
| RubyGems | ❌ | ✅ | ✅ | ✅ | ✅ |
| Swift | ❌ | ❌ | ✅ | ✅ | ✅ |
| Vagrant | ❌ | ❌ | ✅ | ✅ | ✅ |
| Terraform State | ❌ | ❌ | ✅ | ❌ | ❌ |
| Package cleanup rules | ❌ | ⚠️ | ✅ | ✅ | ✅ |
| Package deduplication | ✅ | ✅ | ✅ | ✅ | ✅ |
| Package-link to repo | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## 11. Security

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Secret scanning | ✅ (Advanced Security) | ✅ (secret detection) | ❌ | ❌ | ❌ |
| Dependency scanning | ✅ (Dependabot alerts) | ✅ (dependency scanning) | ❌ | ❌ | ❌ |
| Dependabot version updates | ✅ | ❌ | ❌ | ❌ | ❌ |
| SAST (Static Analysis) | ✅ (code scanning) | ✅ (SAST) | ❌ | ❌ | ❌ |
| CODEOWNERS | ✅ | ✅ | ✅ | ✅ | ✅ |
| Branch protection | ✅ | ✅ | ✅ | ✅ | ✅ |
| GPG/SSH key management | ✅ | ✅ | ✅ | ✅ | ✅ |
| SBOM generation | ⚠️ (Dependabot) | ✅ (SBOM) | ❌ | ❌ | ❌ |
| SLSA provenance | ✅ | ✅ | ❌ | ❌ | ❌ |
| Token scanning partnerships | ✅ | ✅ | ❌ | ❌ | ❌ |
| Push rules (commit restrictions) | ❌ | ✅ (EE) | ⚠️ (branch protection) | ⚠️ (branch protection) | ⚠️ (branch protection) |
| DAST | ✅ (Advanced Security) | ✅ (EE) | ❌ | ❌ | ❌ |
| Container scanning | ✅ | ✅ | ❌ | ❌ | ❌ |
| License compliance | ✅ (Advanced Security) | ✅ (EE) | ❌ | ❌ | ❌ |
| Security policy files | ✅ (SECURITY.md) | ✅ | ✅ | ✅ | ✅ |
| Vulnerability reporting | ✅ | ✅ | ✅ | ✅ | ✅ |
| Private vulnerability reports | ✅ | ✅ (EE) | ✅ | ✅ | ✅ |

---

## 12. Collaboration

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Notifications (email/in-app) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Watch/star/fork | ✅ | ✅ | ✅ | ✅ | ✅ |
| @mentions | ✅ | ✅ | ✅ | ✅ | ✅ |
| Emoji reactions | ✅ | ✅ | ✅ | ✅ | ✅ |
| Task lists in markdown | ✅ (- [ ]) | ✅ | ✅ | ✅ | ✅ |
| Collaborative / real-time editing | ❌ | ⚠️ (real-time in Web IDE) | ❌ | ❌ | ❌ |
| Concurrent editing | ❌ | ❌ | ❌ | ❌ | ❌ |
| RSS feeds | ✅ | ❌ | ✅ | ✅ | ✅ |
| User blocking | ✅ | ✅ | ✅ | ✅ | ✅ |
| Auto-linked references | ✅ | ✅ | ✅ | ✅ | ✅ |
| Pin issues/PRs | ✅ | ✅ | ✅ | ✅ | ✅ |
| Issue assignments | ✅ | ✅ | ✅ | ✅ | ✅ |
| Label system | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## 13. API

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| REST API | ✅ (comprehensive) | ✅ (comprehensive) | ✅ (comprehensive) | ✅ (comprehensive) | ✅ (comprehensive) |
| GraphQL | ✅ | ✅ | ❌ | ❌ | ❌ |
| SSH over HTTP port | ✅ (443) | ✅ | ✅ | ✅ | ✅ |
| Git protocol (git://) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Smart HTTP (HTTPS git) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Webhooks | ✅ | ✅ | ✅ | ✅ | ✅ |
| OAuth2 applications | ✅ | ✅ | ✅ | ✅ | ✅ |
| Personal access tokens | ✅ (fine-grained) | ✅ (scopes) | ✅ (scoped) | ✅ (scoped) | ✅ (scoped) |
| Machine users / bots | ✅ (GitHub App) | ✅ (bot users) | ⚠️ (access tokens) | ⚠️ (access tokens) | ⚠️ (access tokens) |
| Rate limiting | ✅ | ✅ | ✅ | ✅ | ✅ |
| API docs / Swagger | ✅ (OpenAPI) | ✅ (OpenAPI) | ✅ (Swagger) | ✅ (Swagger) | ✅ (Swagger) |
| Git hooks (pre-receive) | ✅ | ✅ | ✅ | ✅ | ✅ |
| API versioning | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## 14. Search

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Global search | ✅ | ✅ | ✅ | ✅ | ✅ |
| Code search (repo) | ✅ | ✅ | ✅ (basic) | ✅ (basic) | ✅ (basic) |
| Code search (global) | ✅ | ✅ (EE) | ✅ (with indexer) | ✅ (with indexer) | ⚠️ (instance-dependent) |
| Repository search | ✅ | ✅ | ✅ | ✅ | ✅ |
| User search | ✅ | ✅ | ✅ | ✅ | ✅ |
| Issue search | ✅ | ✅ | ✅ | ✅ | ✅ |
| Advanced filters / query syntax | ✅ (very powerful) | ✅ | ⚠️ (basic filters) | ⚠️ (basic filters) | ⚠️ (basic filters) |
| Regex code search | ✅ | ✅ | ⚠️ (depends on indexer) | ⚠️ (depends on indexer) | ⚠️ |
| Semantic code search | ✅ (code search v2) | ❌ | ❌ | ❌ | ❌ |
| Search across organizations | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## 15. UI/UX

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Dark mode | ✅ | ✅ | ✅ | ✅ | ✅ |
| Light mode | ✅ | ✅ | ✅ | ✅ | ✅ |
| Responsive / mobile-friendly | ✅ | ✅ | ✅ | ✅ | ✅ |
| Keyboard shortcuts | ✅ (extensive) | ✅ | ✅ | ✅ | ✅ |
| Accessibility (WCAG) | ⚠️ (ongoing) | ⚠️ (ongoing) | ⚠️ | ⚠️ | ⚠️ |
| Custom themes | ❌ (limited) | ⚠️ (branding) | ✅ (custom CSS) | ✅ (custom CSS) | ⚠️ (Forgejo default) |
| Notifications UI | ✅ (inbox) | ✅ (inbox) | ✅ (dashboard) | ✅ (dashboard) | ✅ (dashboard) |
| Activity timeline | ✅ | ✅ | ✅ | ✅ | ✅ |
| Custom CSS injection | ❌ | ⚠️ (admin) | ✅ | ✅ | ❌ (public instance) |
| Custom JS injection | ❌ | ⚠️ (admin) | ✅ | ✅ | ❌ (public instance) |
| Custom footer/logo | ❌ | ✅ | ✅ | ✅ | ❌ (public instance) |
| User profile customization | ✅ | ✅ | ✅ | ✅ | ✅ |
| Multi-language UI | ✅ | ✅ | ✅ (i18n) | ✅ (i18n) | ✅ (i18n) |
| Dashboard / feed | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## 16. Desktop / Apps

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Native desktop app | ✅ (GitHub Desktop) | ❌ (third-party) | ❌ | ❌ | ❌ |
| Mobile apps (iOS/Android) | ❌ | ❌ | ❌ | ❌ | ❌ |
| CLI tool | ✅ (gh CLI) | ✅ (glab CLI) | ✅ (tea CLI) | ✅ (forgejo-cli / tea) | ❌ (use tea) |
| VS Code extension | ✅ (GitHub Pull Requests) | ✅ (GitLab Workflow) | ⚠️ (community) | ⚠️ (community) | ⚠️ (community) |
| JetBrains plugin | ✅ | ✅ | ⚠️ (community) | ⚠️ (community) | ⚠️ (community) |
| Mobile-responsive web | ✅ | ✅ | ✅ | ✅ | ✅ |
| Codespaces / Workspaces | ✅ (GitHub Codespaces) | ✅ (GitLab Workspaces) | ❌ | ❌ | ❌ |

---

## 17. AI Features

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| AI code suggestions (Copilot-like) | ✅ (GitHub Copilot) | ✅ (GitLab Duo) | ❌ | ❌ | ❌ |
| AI PR review | ✅ (Copilot) | ✅ (Duo Review) | ❌ | ❌ | ❌ |
| AI code search | ✅ (Copilot) | ✅ (Duo Chat) | ❌ | ❌ | ❌ |
| AI chat assistant | ✅ (Copilot Chat) | ✅ (Duo Chat) | ❌ | ❌ | ❌ |
| AI code generation | ✅ (Copilot) | ✅ (Duo) | ❌ | ❌ | ❌ |
| Vulnerability explanation (AI) | ✅ | ✅ (Duo) | ❌ | ❌ | ❌ |
| Root cause analysis (AI) | ❌ | ✅ (Duo) | ❌ | ❌ | ❌ |
| AI impact analytics | ❌ | ✅ (Duo SDLC) | ❌ | ❌ | ❌ |

---

## 18. Other

| Feature | GitHub | GitLab | Gitea | Forgejo | Codeberg |
|---|---|---|---|---|---|
| Activity feed (global) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Dashboard | ✅ | ✅ | ✅ | ✅ | ✅ |
| Grafana integration | ❌ | ✅ | ❌ | ❌ | ❌ |
| Labels system | ✅ | ✅ | ✅ | ✅ | ✅ |
| Issue templates | ✅ | ✅ | ✅ | ✅ | ✅ |
| Release management | ✅ | ✅ | ✅ | ✅ | ✅ |
| Changelog generation | ❌ (via Actions) | ✅ (EE) | ❌ | ❌ | ❌ |
| GPG signing (instance) | ✅ | ✅ | ✅ | ✅ | ✅ |
| SSH key management | ✅ | ✅ | ✅ | ✅ | ✅ |
| Deploy keys | ✅ | ✅ | ✅ | ✅ | ✅ |
| Deploy tokens | ✅ | ✅ | ✅ | ✅ | ✅ |
| Repository tokens (write) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Project boards / Kanban | ✅ (Projects v2) | ✅ (boards) | ⚠️ (basic) | ⚠️ (basic) | ⚠️ (basic) |
| Pages (static site hosting) | ✅ (GitHub Pages) | ✅ (GitLab Pages) | 🔧 (third-party) | 🔧 (Codeberg Pages) | ✅ (Codeberg Pages) |
| Snippets / Gists | ✅ | ✅ (Snippets) | 🔧 (OpenGist) | 🔧 (OpenGist) | 🔧 (OpenGist) |
| Moderation tools | ✅ | ✅ | ✅ | ✅ | ✅ |
| Import from GitHub/GitLab/etc | ✅ | ✅ | ✅ | ✅ | ✅ |
| Free/open source | ❌ (proprietary) | ❌ (Open Core) | ⚠️ (CE free, EE paid) | ✅ (AGPL) | ✅ (via Forgejo) |
| Low resource usage | ❌ | ❌ | ✅ | ✅ | ✅ |
| Self-hosted | ✅ (Enterprise Server) | ✅ | ✅ | ✅ | ❌ (public instance) |
| Public hosted instance | ✅ (github.com) | ✅ (gitlab.com) | ✅ (gitea.com) | ✅ (codeberg.org) | ✅ (codeberg.org) |

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
