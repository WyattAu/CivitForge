# CivitForge Version Tracker

Version: 1.5.0
Last Updated: 2026-06-04
Tests: 2,888 passing
Clippy: 0 warnings

## Artifact Summary

- Rust source files: 290+
- Rust lines of code: ~105,000
- Cargo workspace crates: 8 (civit-shared, civit-pipeline, civit-core, civit-runner, civit-brain, civit-vfs, civit-crypto, civit-ui)
- Unit tests passing: 2,888
- Clippy warnings: 0
- Format violations: 0
- `#![forbid(unsafe_code)`: Enforced across all crates
- API endpoints: ~68 routes
- Migrations: 001-025 (odd-numbered)
- Rust edition: 2024
- MSRV: 1.88

## v1.5.0 Changes

- Read replica router (primary/replica pool split, failover, lag monitoring)
- Multi-region replication transport (channel-based, SHA-256 checksums, heartbeat)
- Vector clocks (conflict detection: happened-before, concurrent, merge)
- K8s operator (CivitForgeApp CRD, reconciler, health checker)
- CDN artifact pre-signed URLs (HMAC-SHA256, TTL, cache headers)
- Artifact serving API (download, pre-signed URL, HEAD/ETag, cache invalidation)
- Password change verifies current password
- Code browser uses gix EntryMode for file/directory detection
- Federation inbox validates HTTP signatures
- Federation actor generates real Ed25519 keypair
- Shared UI utilities extracted (8 pages deduplicated)
- Wiki page content fetch wired, repo settings wired, explore search wired

## Tags

- v1.0.0 (6c625cc)
- v1.1.0
- v1.1.1
- v1.2.0
- v1.3.0
- v1.4.0
- v1.5.0

## Next: v2.0.0

- Tauri desktop application
- PWA mobile
- Marketplace / extensions
- API stability guarantee (/api/v1/ frozen)
