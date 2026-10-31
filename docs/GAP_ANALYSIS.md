## 1. CRITICAL Gaps: All 5 competitors have it
| 13 | Raw file download | [YES] |; |
| 14 | Code browsing | [YES] |; |
| 15 | Search within repo | [YES] |; |
| 28 | Issue tracker | [YES] |; |
| 29 | Labels | [YES] |; |
| 30 | Milestones | [YES] |; |
| 31 | Emoji reactions | [YES] |; |
| 46 | PR/MR creation | [YES] |; |
| 47 | Inline comments | [YES] |; |
| 48 | Status checks | [YES] |; |
| 49 | Merge commit | [YES] |; |
| 50 | Review requests | [YES] |; |
| 51 | Cherry-pick | [YES] (via Git API) |; |
| 61 | Pipeline YAML config | [YES] |; |
| 62 | Artifacts | [YES] |; |
| 63 | Manual triggers | [YES] |; |
| 64 | Container registry | [YES] (OCI) |; |
| 70 | Wiki edit via web | [YES] |; |
| 71 | Built-in wiki | [YES] |; |
| 72 | Page history | [YES] |; |
| 73 | Markdown rendering | [YES] |; |
| 78 | Username/password | [YES] |; |
| 79 | 2FA/TOTP | [YES] |; |
| 80 | WebAuthn | [YES] |; |
| 81 | OIDC login | [YES] |; |
| 94 | Repo activity page | [YES] |; |
| 95 | Fork | [YES] |; |
| 96 | Star/watch | [YES] |; |
| 102 | Org creation | [YES] |; |
| 109 | Watch/star/fork | [YES] |; |
| 110 | Emoji reactions | [YES] |; |
| 111 | Label system | [YES] |; |
| 116 | REST API | [YES] |; |
| 124 | Dashboard | [YES] |; |
| 125 | Activity feed | [YES] |; |
| 126 | SSH key management | [YES] |; |
## 2. HIGH Gaps: 4 of 5 competitors have it
| 159 | SCIM provisioning | 2/5 (GitHub/GitLab Enterprise only) | [NO] | Low priority; only SaaS |
## 3. MEDIUM Gaps: 3 of 5 competitors have it
| 174 | DAST scanning | 2/5 (GitHub/GitLab Advanced) | [NO] | Low priority; security scanner |
| 175 | Container scanning | 2/5 | [NO] | Low priority; security scanner |
| 184 | Snippets / Gists | 1/5 (GitLab only native) | [NO] | Low priority; consider plugin |
## 4. LOW Gaps: 2 or fewer competitors have it
| 197 | SAML SSO | CivitForge [YES] |; | Already implemented |
| 200 | Mobile apps | 0/5 | [NO] | Not in scope; responsive web instead |
| 201 | Codespaces / Workspaces | 2/5 | [NO] | Not in scope; use VFS instead |
| 204 | Grafana integration | 1/5 | [NO] | Not in scope; use metrics export |
| 205 | CLA tooling |; | [NO] | Very low priority |
| 207 | Secret scanning | 2/5 (GitHub/GitLab) | [NO] | Medium priority; could leverage AI AST |
## 5. CivitForge Competitive Advantages: No competitor has these
## 8. Priority Matrix: Recommended Implementation Order
Implement these first; they are features all users expect from any forge:
1. **Blame view** (#1); fundamental to code browsing
2. **Syntax highlighting** (#5); non-negotiable for any code viewer
3. **Branch switching** (#7); basic navigation
4. **Assignees** (#17); core issue workflow
5. **Cross-references** (#19); issue/PR linking
6. **@mentions** (#104); collaboration
7. **Notifications** (#103); user engagement
8. **Email verification** (#74); security baseline
9. **Account lockout** (#75); security baseline
10. **Archive download** (#S1); complete the stub
11. **Collaborator management** (#S2); complete the stub
12. **Dark mode** (#127); UX baseline
13. **Responsive/mobile** (#128); UX baseline
14. **Deploy keys/tokens** (#118-119); CI/CD prerequisite
15. **Webhooks** (#112); ecosystem integration
16. **Personal access tokens** (#113); API usage
17. **Rate limiting** (#114); security baseline
18. **Branch/tag protection** (#88-89); repo security
19. **Release management** (#117); software distribution
20. **Pipeline detail UI** (#U1); CI/CD usability
21. **Org profile page** (#U1); org visibility
22. **Runner management UI** (#U5); CI/CD operations
1. **Squash/rebase/ff merge** (#34-36); merge strategies
2. **Draft PRs** (#37); common workflow
3. **CODEOWNERS** (#33); team workflow
4. **Issue templates** (#16); issue quality
5. **File attachments** (#18); issue/PR context
6. **Task lists in markdown** (#105); issue tracking
7. **Secrets management** (#54); CI/CD security
8. **Matrix/parallel builds** (#52-53); CI/CD power
9. **DAG pipelines** (#60); CI/CD power
10. **Scheduled runs** (#57); CI/CD automation
11. **Status badges** (#58); repo README integration
12. **Topics** (#82); repo discoverability
13. **Team management** (#97-98); org workflow
14. **Import from GitHub/GitLab** (#121); migration path
15. **LFS 2.0** (#8); large file support
16. **Commit graph** (#10); repo visualization
17. **README rendering** (#12); repo landing
18. **Web code editor** (#9); quick edits
19. **Language stats** (#11); repo overview
20. **Merge queue** (#40); branch protection enforcement
21. **Linked issues auto-close** (#41); workflow automation
22. **Revert commit** (#172); common operation
1. **Federation polish**; stabilize cross-instance PRs, follow UX
2. **AI PR review**; ship the RAG + AST pipeline in UI
3. **AI code search**; expose Tantivy + HNSW via natural language
4. **VFS launch**; market FUSE mount as killer feature
5. **SLSA provenance**; dashboard for supply chain transparency
6. **Edge caching + pre-signed URLs**; performance marketing
7. **K8s operator GA**; enterprise deployment story
8. **mTLS + Cosign**; harden CI/CD supply chain