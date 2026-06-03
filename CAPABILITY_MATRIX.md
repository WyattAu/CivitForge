# Capability Matrix

Tooling requirements mapped against CivitForge's stack. Identifies gaps and procurement dependencies.

## Language and Compiler

| Tool | Required By | Version Used | Status |
|------|------------|-------------|--------|
| Rust (rustc/cargo) | All components | 1.88 | Satisfied |
| C Compiler (gcc/clang) | crun build dependency | System | Satisfied (build dependency only) |

## Frameworks and Libraries

| Tool | Required By | Version Used | Status |
|------|------------|-------------|--------|
| axum | civit-core (HTTP) | 0.8 | Satisfied |
| tokio | All async runtime | 1 | Satisfied |
| gitoxide (gix) | civit-core (Git engine) | 0.70 | Satisfied |
| kube-rs | civit-runner (K8s operator) | 0.98 | Satisfied |
| russh | civit-core (SSH server) | 0.61 | Satisfied |
| tree-sitter | civit-brain (AST parser) | 0.24 | Satisfied (feature-gated) |
| sqlx | civit-core (DB queries) | 0.8 | Satisfied |
| tonic/prost | civit-vfs (gRPC) | 0.12/0.13 | Satisfied |
| ring | civit-crypto (crypto) | 0.17 | Satisfied |

## Infrastructure

| Tool | Required By | Version Used | Status |
|------|------------|-------------|--------|
| Docker / Podman | Local development | System | Satisfied |
| kubectl | civit-runner testing | 1.28+ | Satisfied |
| Helm | Deployment | 3.x | Satisfied |

## External Services (Development)

| Tool | Required By | Version Used | Status |
|------|------------|-------------|--------|
| PostgreSQL | Data storage | 17 | Satisfied |
| Redis | Sessions, cache, pub/sub | 7 | Satisfied |
| Qdrant | civit-brain (vectors) | Latest | Optional |

## CI/CD

| Tool | Required By | Status |
|------|------------|--------|
| GitHub Actions | CI pipeline | Satisfied |
| husky | Pre-commit hooks | Satisfied |

## Linting and Quality

| Tool | Required By | Status |
|------|------------|--------|
| cargo clippy | All Rust code | Satisfied (0 warnings enforced) |
| cargo fmt | All Rust code | Satisfied (0 violations enforced) |
