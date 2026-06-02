# CivitForge Version Tracker

Phase: Post-Release Maintenance
Version: 1.1.0
Status: In Development
Last Updated: 2026-06-02
Tests: 2,644 passing
Clippy: 0 warnings

## Artifact Summary
- Rust source files: 264+
- Rust lines of code: ~97,000
- Cargo workspace crates: 8 (civit-shared, civit-pipeline, civit-core, civit-runner, civit-brain, civit-vfs, civit-crypto, civit-ui)
- Unit tests passing: 2,644
- Unit tests ignored: 0
- Clippy warnings: 0
- Format violations: 0
- `#![forbid(unsafe_code)]`: Enforced across all crates
- CI/CD: Green (all checks passing, --locked enforced)
- Pre-commit hooks: fmt + clippy -D warnings + test --locked
- API endpoints: ~60 routes
- Migrations: 11 (001-021, odd-numbered)

## v1.1.0 Changes
- Fixed hardcoded "todo" owner in runners — now joins users table for username
- Real unified diff via LCS algorithm for wiki (unified_diff, DiffHunk, DiffLine, lcs_lines)
- Wiki content_snapshot column (migration 019/020) — stores page content in revision
- AES-256-GCM encryption for pipeline variables using ring::aead (encrypt_value, decrypt_value)
- Runner secret fetching decrypts AES-256-GCM encrypted variables
- Checkout/cache/artifact action handlers in executor
- Service container lifecycle (start_services, stop_all_services, ServiceGuard RAII)
- Real CEL expression evaluator (==, !=, contains, startsWith, endsWith, matches, &&, ||, !, ${{ }} expansion)
- PostgreSQL full-text search (tsvector/tsquery, GIN indexes, triggers) — migration 021/022
- **Token refresh endpoint** (POST /api/v1/auth/refresh — validates existing JWT, issues new token)
- **RSA-SHA256 signing** in federation (via rsa + pkcs8 crates, PKCS1v15 padding)
- **ECDSA-P256 signing** in federation (via ring EcdsaKeyPair)
- **Retriever trait** for RAG pipeline — RagOrchestrator now takes Box<dyn Retriever>
- **KeywordRetriever** replaces MockRetriever (thread-safe via RwLock)
- **Podman log errors** return proper errors instead of fake "log line N" text
- **OCI dedup** get_layer() fixed — uses put_direct(digest, data) for correct storage/retrieval
- **Pipeline YAML reader** ref-aware — tries git archive first, falls back to filesystem
- **Test stubs gated** behind #[cfg(test)] — StubLlmProvider, StubReviewAgent, StubVulnScanner, MockRemoteProvider
- **CEL doc comment** updated to reflect actual implementation (9 expression kinds)
- **Encryption key warning** logged when CIVIT_ENCRYPTION_KEY not set

## Completed Phases (v1.0.0)
- [x] Phase -1 through Phase 26: All 26 phases complete
- [x] v1.0.0 tagged at 6c625cc

## Next: v1.2.0
- FUSE kernel mount
- Full SAML XML-DSig, WebAuthn ES-256/RS256, real HSM PKCS#11
- Project boards/Kanban, merge queue, dep graph, multi-region
- Real-time WebSocket log streaming
- Tantivy-based code search upgrade
- Git-backed wiki storage (.wiki.git)
- Per-repo encryption keys
