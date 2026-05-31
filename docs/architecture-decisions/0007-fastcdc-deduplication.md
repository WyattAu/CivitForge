# ADR-0007: FastCDC Deduplication

## Status

Accepted

## Context

The VFS layer stores large volumes of repository data. Content-addressable storage with chunk-level deduplication reduces storage costs and bandwidth.

## Decision

Use FastCDC for content-defined chunking in the VFS layer.

## Considerations

- FastCDC provides content-defined chunking with rolling hash
- Chunk boundaries are deterministic based on content, not position
- Enables efficient deduplication: identical files produce identical chunks
- Pure Rust implementation available
- Suitable for git pack-like storage with deduplication on top
- Average chunk size configurable (32KB-256KB recommended)

## Alternatives Considered

- **Rabin fingerprinting**: Similar approach, less optimized
- **Fixed-size blocks**: Poor deduplication for shifted content
- **rsync rolling checksum**: Designed for delta transfer, not storage

## Consequences

- VFS stores content-addressed chunks identified by BLAKE3 hashes
- Upload deduplication: only transmit/store chunks not already present
- Clone optimization: clients can skip chunks they already have locally
- Metadata stored in PostgreSQL, chunk data on local/filesystem storage
