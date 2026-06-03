# CivitForge Version Tracker

Version: 1.1.0
Last Updated: 2026-06-02
Tests: 2,644 passing
Clippy: 0 warnings

## Artifact Summary

- Rust source files: 264+
- Rust lines of code: ~97,000
- Cargo workspace crates: 8 (civit-shared, civit-pipeline, civit-core, civit-runner, civit-brain, civit-vfs, civit-crypto, civit-ui)
- Unit tests passing: 2,644
- Clippy warnings: 0
- Format violations: 0
- `#![forbid(unsafe_code)]`: Enforced across all crates
- API endpoints: ~60 routes
- Migrations: 001-021 (odd-numbered)
- Rust edition: 2024
- MSRV: 1.88

## v1.1.0 Changes

- Token refresh endpoint (POST /api/v1/auth/refresh)
- RSA-SHA256 and ECDSA-P256 signing in federation
- Retriever trait + KeywordRetriever for RAG pipeline
- Unified diff for wiki via LCS algorithm
- Wiki content_snapshot column (migration 019/020)
- AES-256-GCM encryption for pipeline variables
- Checkout/cache/artifact action handlers in executor
- Service container lifecycle (ServiceGuard RAII)
- CEL expression evaluator (9 expression kinds, ${{ }} expansion)
- PostgreSQL full-text search (tsvector/tsquery, GIN indexes, migration 021/022)
- Git archive-based pipeline YAML reading
- Runner owner lookup fixed (joins users table)
- OCI dedup get_layer() fixed (put_direct by digest)
- Test stubs gated behind #[cfg(test)]

## Tags

- v1.0.0 (6c625cc)
- v1.1.0

## Next: v1.2.0

- FUSE kernel mount
- Full SAML XML-DSig, WebAuthn ES-256/RS256, real HSM PKCS#11
- Project boards, merge queue, dependency graph, multi-region
- Real-time WebSocket log streaming
- Git-backed wiki storage (.wiki.git)
- Per-repo encryption keys
