| **CRITICAL gaps (5/5 have it)** | 18 |; |; |; |; | N/A |
| **HIGH gaps (4/5 have it)** | 12 |; |; |; |; | N/A |
| 19 | **VFS (FUSE mount)** | Mount repos as filesystems; unique to CivitForge |
### Phase 0: Foundation (Weeks 1-2): Unblock Everything
| F0.1 | **Archive download** (git archive zip/tar); complete NOT_IMPLEMENTED stub | 2d | HIGH | git binary available |
| F0.2 | **Collaborator add/remove**; complete NOT_IMPLEMENTED stubs | 1d | HIGH | DB tables exist |
| F0.8 | **Rate limiting** verify UI (already in middleware) | 0.5d | DONE |; |
| F1.1 | **Syntax highlighting**; integrate highlight.js via CDN (50+ langs) | 2d | CRITICAL |
| F1.2 | **README rendering**; fetch README.md on repo detail, render markdown | 2d | CRITICAL |
| F1.3 | **Language stats bar**; detect via extension, render colored breakdown | 2d | HIGH |
| F1.4 | **Commit history per file**; `git log -- <path>` endpoint + UI | 3d | HIGH |
| F1.5 | **Git blame view**; `git blame` endpoint + line-by-line UI | 4d | HIGH |
| F1.6 | **Submodules**; detect and render submodule links | 1d | MEDIUM |
| F1.7 | **Repository size display**; `du -sh` on repo path | 0.5d | LOW |
| F1.8 | **File finder** (fuzzy search); debounce search input, filter tree | 2d | MEDIUM |
| F1.9 | **Responsive / mobile-friendly**; CSS breakpoints, mobile layouts | 5d | HIGH |
| F1.10 | **Keyboard shortcuts**; global keybindings for nav/actions | 2d | MEDIUM |
| F2.1 | **Assignees**; DB migration + API + picker UI | 2d | CRITICAL |
| F2.2 | **Cross-references**; parse #123 in markdown, render links | 2d | CRITICAL |
| F2.3 | **@mentions**; parse @user in markdown, trigger notifications | 2d | CRITICAL |
| F2.4 | **Task lists**; render `- [ ]` / `- [x]` as checkboxes | 1d | HIGH |
| F2.5 | **Issue templates**; parse .github/ISSUE_TEMPLATE/ | 2d | MEDIUM |
| F2.6 | **File attachments**; multipart upload endpoint | 3d | HIGH |
| F2.7 | **Issue pinning**; pin/unpin + sort in list | 1d | MEDIUM |
| F2.8 | **Lock discussion**; lock/unlock endpoint + UI toggle | 1d | MEDIUM |
| F2.9 | **Squash merge**; merge strategy in git receive-pack | 2d | CRITICAL |
| F2.10 | **Rebase merge**; merge strategy | 2d | HIGH |
| F2.11 | **Fast-forward merge**; merge strategy | 1d | HIGH |
| F2.12 | **Draft PRs**; draft flag + WIP prefix | 1d | HIGH |
| F2.13 | **Merge message templates**; custom format strings | 1d | MEDIUM |
| F2.14 | **Linked issues auto-close**; parse "Fixes #123" | 2d | HIGH |
| F2.15 | **Push to existing PR**; auto-update on branch push | 2d | HIGH |
| F2.16 | **PR templates**; render template in creation form | 1d | MEDIUM |
| F2.17 | **Download patch**; .patch format from PR diff | 1d | MEDIUM |
| F2.18 | **Release management**; CRUD + tag association + assets | 3d | HIGH |
| F2.19 | **Branch/tag protection rules**; UI for existing backend | 3d | HIGH |
| F3.1 | **Matrix builds**; YAML `matrix` + `include` expansion | 3d | HIGH |
| F3.2 | **Parallelism**; `parallel` directive + job fan-out | 2d | HIGH |
| F3.3 | **Secrets management**; encrypted store + inject at runtime | 2d | CRITICAL |
| F3.4 | **Caches**; cache API + mount in runner | 2d | HIGH |
| F3.5 | **Environments + deployments**; env model + deployment records | 3d | MEDIUM |
| F3.6 | **Scheduled runs (cron)**; cron trigger parsing + scheduler | 2d | MEDIUM |
| F3.7 | **Status badges**; SVG endpoint for pipeline status | 1d | MEDIUM |
| F3.8 | **Auto-cancel redundant runs**; cancel superseded on push | 1d | MEDIUM |
| F3.9 | **Runner management UI**; runner list, status, job logs | 3d | HIGH |
| F3.10 | **Pipeline visualization**; DAG graph rendering | 2d | MEDIUM |
| F3.11 | **GitHub Actions compat**; workflow syntax alignment | 5d | HIGH |
| F3.12 | **OIDC workload identity**; issue OIDC tokens to pipelines | 3d | MEDIUM |
| F4.1 | **Team management**; CRUD teams within org + permissions | 3d | HIGH |
| F4.4 | **Org profile page**; landing with repos, members, teams | 2d | MEDIUM |
| F4.11 | **Moderation tools**; report, hide, ban | 3d | MEDIUM |
| F4.12 | **Import from GitHub/GitLab**; migration adapters | 5d | HIGH |
| F4.13 | **Custom footer/logo**; admin settings | 1d | LOW |
| F5.1 | **AI PR review in UI**; expose RAG + AST pipeline as inline comments | 5d | [NEW] DIFFERENTIATOR |
| F5.2 | **AI code search**; natural language → code results via RAG | 3d | [NEW] DIFFERENTIATOR |
| F5.3 | **VFS launch**; FUSE mount UI, mount manager page | 5d | [NEW] DIFFERENTIATOR |
| F5.4 | **Federation polish**; stabilize cross-instance PRs, follow UX | 5d | [NEW] DIFFERENTIATOR |
| F5.5 | **SLSA dashboard**; provenance transparency UI | 3d | [NEW] DIFFERENTIATOR |
| F5.6 | **Edge caching UI**; pre-signed URL management | 2d | [NEW] DIFFERENTIATOR |
| F5.7 | **Compliance dashboard**; ISO 27001, audit trail viewer | 3d | [NEW] DIFFERENTIATOR |
| F5.8 | **NodeInfo endpoint**; for federation discovery | 0.5d | FEDERATION |
| F5.9 | **Secret scanning**; AI-powered credential detection in code | 5d | [NEW] DIFFERENTIATOR |
| F5.11 | **Commit graph visualization**; SVG branch topology | 3d | HIGH |
| F5.13 | **Push/pull mirrors**; mirror config + sync worker | 3d | HIGH |
| F5.14 | **Git LFS 2.0**; batch API + transfer adapter | 5d | HIGH |
| F6.9 | **Kanban boards**; drag-and-drop issue boards | 5d | HIGH |
| F6.10 | **Web code editor**; CodeMirror integration | 5d | HIGH |
| Phase 0: Foundation | 2 weeks | 10 tasks | CRITICAL; unblocks all other work |
| Phase 1: Code Browser | 2 weeks | 10 tasks | HIGH; daily user experience |
| Phase 2: Issues & PRs | 3 weeks | 19 tasks | HIGH; core workflow |
| Phase 3: CI/CD | 3 weeks | 12 tasks | HIGH; developer experience |
| Phase 4: Org & Admin | 2 weeks | 13 tasks | HIGH; team collaboration |
| Phase 5: Differentiation | 4 weeks | 14 tasks | STRATEGIC; moat building |
| Phase 6: Ecosystem | 4 weeks | 11 tasks | MEDIUM; breadth |