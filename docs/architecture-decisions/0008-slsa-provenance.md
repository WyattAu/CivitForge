# ADR-0008: SLSA Provenance

## Status

Accepted

## Context

CI/CD pipeline outputs (binaries, containers, artifacts) need verifiable provenance to establish supply chain integrity.

## Decision

Generate SLSA (Supply-chain Levels for Software Artifacts) provenance attestation for all pipeline outputs.

## Considerations

- SLSA provides a framework for supply chain security levels (L1-L4)
- Target: SLSA Level 4 (hermetic builds, reproducible builds, provenance attestation, non-falsifiable provenance)
- In-toto attestation format for provenance metadata
- Provenance includes: builder identity, source commit, build configuration, dependencies
- Public verification via Transparency Log or Sigstore

## Implementation

- `civit-runner` generates provenance during pipeline execution
- Attestation includes: commit SHA, builder identity, build timestamp, build config digest
- Signed using `civit-crypto` with the instance's signing key
- Published alongside artifacts in the VFS storage

## Consequences

- All pipeline artifacts have cryptographic provenance
- Builds are hermetic and reproducible (isolated sandbox environment, deterministic inputs)
- Non-falsifiable provenance via SLSA Level 4 requirements (hardened builder, isolated build environment)
- Consumers can verify artifact authenticity independently via Sigstore transparency log
- Provenance generation adds minimal latency to pipeline completion
