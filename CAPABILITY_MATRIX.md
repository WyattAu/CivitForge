# Capability Matrix

Map available tooling against CivitForge's required stack. Identifies gaps and procurement dependencies.

---

## Language and Compiler

| Tool | Required By | Version Available | Version Required | Status |
|------|------------|-------------------|------------------|--------|
| Rust (rustc/cargo) | All components | 1.95.0 | 1.78+ | Satisfied |
| Lean 4 | Formal verification (future) | 4.30.0 | -- | Available (not yet required) |
| C Compiler (gcc/clang) | crun build dependency | System | -- | Satisfied (build dependency only) |
| LLVM | Code generation backend | System | -- | Satisfied (via rustc) |

## WebAssembly

| Tool | Required By | Version Available | Version Required | Status |
|------|------------|-------------------|------------------|--------|
| wasm-pack | Web UI (Wasm) | Not verified | Latest | **Required** |
| wasm-bindgen | Wasm FFI | Not verified | Latest | **Required** |

## Frameworks and Libraries

| Tool | Required By | Version Available | Version Required | Status |
|------|------------|-------------------|------------------|--------|
| axum | CivitCore (HTTP/gRPC) | crates.io (latest) | 0.7+ | Satisfied (crate available) |
| tokio | All async runtime | crates.io (latest) | 1.x | Satisfied |
| gitoxide | CivitCore (Git engine) | crates.io (latest) | latest | Satisfied |
| rayon | Parallel processing | crates.io (latest) | 1.x | Satisfied |
| kube-rs | CivitRunner (K8s operator) | crates.io (latest) | 0.9+ | Satisfied (crate available) |
| russh | CivitCore (SSH server) | crates.io (latest) | latest | Satisfied |
| tree-sitter | CivitBrain (AST parser) | crates.io (latest) | latest | Satisfied |
| sqlx | CivitCore (DB queries) | crates.io (latest) | 0.7+ | Satisfied |
| fuser | CivitVFS (FUSE daemon) | crates.io (latest) | latest | Satisfied |

## Cryptography

| Tool | Required By | Version Available | Version Required | Status |
|------|------------|-------------------|------------------|--------|
| ring | CivitCore (default crypto) | crates.io (latest) | latest | Satisfied (non-FIPS) |
| boring | CivitCore (FIPS crypto) | crates.io (latest) | latest | Satisfied (FIPS mode) |
| cosign | CivitRunner (artifact signing) | CLI tool | Latest | **Required** (external binary) |
| sbom-rs / syft | CivitRunner (SBOM generation) | Not verified | Latest | **Required** |

## Infrastructure

| Tool | Required By | Version Available | Version Required | Status |
|------|------------|-------------------|------------------|--------|
| Docker / Podman | Local development | System | Latest | Satisfied |
| kubectl | CivitRunner testing | Not verified | 1.28+ | **Required** |
| kind / minikube | Local K8s testing | Not verified | Latest | **Required** |
| Helm | Deployment | Not verified | 3.x | **Required** |
| Terraform | IaC (optional) | Not verified | 1.x | **Optional** |

## External Services (Development)

| Tool | Required By | Version Available | Version Required | Status |
|------|------------|-------------------|------------------|--------|
| CockroachDB | CivitData | Not verified | 23.1+ | **Required** |
| MinIO | CivitData (S3 compat) | Not verified | Latest | **Required** |
| Redis / DragonflyDB | CivitData (cache/queue) | Not verified | 7.x | **Required** |
| Qdrant | CivitBrain (vectors) | Not verified | Latest | **Required** (Phase 3) |
| vLLM | CivitBrain (inference) | Not verified | Latest | **Required** (Phase 3) |

## Linting and Quality

| Tool | Required By | Version Available | Version Required | Status |
|------|------------|-------------------|------------------|--------|
| cargo clippy | All Rust code | Bundled with Rust | Latest | Satisfied |
| cargo fmt | All Rust code | Bundled with Rust | Latest | Satisfied |
| cargo deny | Dependency audit | crates.io (latest) | Latest | **Required** |
| cargo geiger | Unsafe code audit | Not verified | Latest | **Required** |
| cargo audit | Vulnerability scanning | Not verified | Latest | **Required** |

## CI/CD (Self-Hosting)

| Tool | Required By | Version Available | Version Required | Status |
|------|------------|-------------------|------------------|--------|
| GitHub Actions | CivitForge CI | GitHub | -- | Satisfied (current) |
| Self-hosted runner | Self-hosted CI migration | GitHub Actions | -- | **Future** (Phase 2) |

---

## Gap Summary

| Gap | Severity | Phase Needed | Action |
|-----|----------|-------------|--------|
| wasm-pack / wasm-bindgen not verified | Medium | 1 | Install and verify Wasm build toolchain |
| cosign CLI not available | High | 2 | Install cosign for SBOM signing development |
| SBOM tool (sbom-rs / syft) not verified | High | 2 | Evaluate and install SBOM generation tooling |
| kubectl / kind not verified | High | 2 | Install K8s tooling for CivitRunner development |
| Helm not verified | Medium | 1 | Install Helm for deployment chart development |
| CockroachDB not installed | High | 1 | Deploy CockroachDB for local development |
| MinIO not installed | High | 1 | Deploy MinIO for S3-compatible local storage |
| Redis not installed | Medium | 1 | Deploy Redis for event bus development |
| cargo deny / cargo geiger / cargo audit not installed | Medium | 1 | Install security auditing toolchain |
| Qdrant / vLLM not available | Low | 3 | Deferred to Phase 3 |
