# ADR-001: Scoped Unsafe Code in Feature-Gated Crates

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Created** | 2026-06-01 |
| **Author** | Nexus (Principal Systems Architect) |
| **Decision Makers** | Nexus, Project Maintainer |

## Context

The CivitForge monorepo enforces `#![forbid(unsafe_code)]` at the crate root of all 5 workspace crates. This has been a deliberate security and auditability decision. However, several ROADMAP phases require functionality that has no pure-Rust alternative:

| Phase | Requirement | Why No Pure-Rust |
|-------|-------------|-------------------|
| 2.1 | tree-sitter AST parsing for 12+ languages | tree-sitter runtime is written in C; all grammar crates embed C source compiled via `cc` at build time |
| 2.2 | ML embeddings (candle-rs) | `tch` wraps PyTorch C++ bindings; `burn` is pure-Rust alternative (preferred) |
| 4.2 | FUSE kernel filesystem mount | `fuser` wraps libfuse3 C library; no Rust-native FUSE implementation exists |
| 5.4 | Full PKCS#11 HSM support | `cryptoki` crate wraps PKCS#11 C library; no pure-Rust PKCS#11 exists |

Additionally, Phase 2.1 (AST parsing) has a **hybrid strategy**: pure-Rust native parsers exist for some languages and are preferred, while tree-sitter C FFI fills the gaps:

| Language | Parser | Unsafe Required |
|----------|--------|-----------------|
| Rust | `syn` v2.0 | No |
| TypeScript/JavaScript | `swc_ecma_parser` | No (transitive via `swc_common` — audited, minimal) |
| SQL | `sqlparser` v0.62 | No |
| JSON/TOML/Markdown | `serde_json`/`toml`/`pulldown-cmark` | No |
| Python, Go, C, C++, Java, Kotlin, Bash, Ruby, PHP, Swift, Haskell, Scala | `tree-sitter-*` grammars | Yes (build-time C compilation) |

## Decision

**Accept scoped `unsafe` code within designated feature-gated modules only.**

### Rules

1. **Feature-gated isolation**: All code requiring `unsafe` (direct or transitive) is placed behind explicit Cargo feature flags that are **off by default**.

2. **Module-level `#![forbid(unsafe_code)]` remains**: The crate root retains `#![forbid(unsafe_code)]`. Only specific modules behind feature gates remove this restriction:
   ```rust
   // In modules requiring tree-sitter C FFI:
   #![cfg_attr(feature = "treesitter", allow(unsafe_code))]
   ```

3. **No `unsafe` in default build**: `cargo build` without any features produces the exact same codebase as today — zero `unsafe`, zero new dependencies.

4. **Feature flag naming convention**:
   - `treesitter` — enables tree-sitter C FFI AST parsing for all supported grammars
   - `fuse-mount` — enables FUSE kernel filesystem mount via `fuser`
   - `pkcs11-hsm` — enables real PKCS#11 HSM via `cryptoki`
   - `ml-embeddings` — enables ML embeddings via `burn` (preferred) or `candle-rs`

5. **SAFETY annotations mandatory**: Every `unsafe` block must have a `// SAFETY:` comment explaining the invariant being maintained.

6. **CI enforcement**: A dedicated CI job runs `cargo clippy --all-features -- -D warnings` to audit all unsafe paths. The default CI job runs without features.

### Implementation: Phase 2.1 AST Parser Tiers

```
Tier 1 (Native, no unsafe):  syn (Rust), swc (JS/TS), sqlparser (SQL), serde_json/toml/pulldown-cmark
Tier 2 (Tree-sitter, unsafe): tree-sitter-python, tree-sitter-go, tree-sitter-c, tree-sitter-cpp,
                                tree-sitter-java, tree-sitter-kotlin, tree-sitter-bash, tree-sitter-ruby,
                                tree-sitter-php, tree-sitter-swift, tree-sitter-haskell, tree-sitter-scala
Tier 3 (Regex fallback):      existing RegexAstParser — always available, no features needed
```

The unified dispatcher routes: Tier 1 > Tier 2 (if `treesitter` feature enabled) > Tier 3.

## Consequences

### Positive

- Preserves `#![forbid(unsafe_code)]` guarantee for default builds
- Enables production-quality AST parsing for Rust, JS/TS, and SQL without any unsafe
- Tree-sitter support unlocks 12+ additional languages when explicitly opted-in
- Machine-enforceable boundary via feature flags
- No new dependencies in default build

### Negative

- Tree-sitter grammars require a C compiler at build time when the feature is enabled
- Two parser tiers means slightly different AST fidelity across languages
- Feature flag combinations need testing matrix in CI

### Risks

- **Risk**: `swc_common` transitive dependencies may introduce unsafe in future versions.
  **Mitigation**: Pin `swc` version, audit with `cargo unsafe` periodically.

- **Risk**: tree-sitter grammar maintenance burden.
  **Mitigation**: Use tree-sitter org grammars (highest maintenance quality), pin versions.

## Alternatives Considered

| Alternative | Description | Why Rejected |
|-------------|-------------|--------------|
| Pure-Rust parsers for all languages | Write recursive-descent parsers for each of 20+ languages | ~3,500-4,500 hours engineering effort; fragile; zero ecosystem reuse |
| Lift `#![forbid(unsafe_code)]` entirely | Replace `forbid` with `warn` or remove | Loses compile-time guarantee; unsafe creeps into safe crates |
| WASM-based tree-sitter | Compile tree-sitter to WASM, use wasmtime | Still requires unsafe in wasmtime runtime; added complexity for no safety gain |
| Restrict to Tier 1 only | Only support Rust/JS/TS/SQL AST parsing | Excludes 12+ languages; insufficient for code review / RAG coverage |

## Related Standards

- ISO/IEC 12207:2017 §6.4 (Software Construction)
- IEC 61508 SIL considerations for C FFI boundary
- NIST SP 800-53 SA-11 (Developer Security Testing)

## Related ADRs

None (first ADR).

## Feature Flag Dependency Matrix

| Feature | New Dependencies (transitive) | Build Requires | Unsafe |
|---------|------------------------------|----------------|--------|
| `syn-parser` | `syn`, `quote` (no transitive unsafe) | Rust compiler only | No |
| `swc-parser` | `swc_ecma_parser`, `swc_common`, `swc_atoms` | Rust compiler only | Minimal (audited) |
| `sql-parser` | `sqlparser` (no transitive unsafe) | Rust compiler only | No |
| `treesitter` | `tree-sitter`, `tree-sitter-python`, `tree-sitter-go`, ... (12 grammar crates) | C compiler (`cc`) | Yes (build-time C) |
| `fuse-mount` | `fuser` | C compiler + FUSE dev headers | Yes (runtime FFI) |
| `pkcs11-hsm` | `cryptoki` | C compiler + PKCS#11 headers | Yes (runtime FFI) |
| `ml-embeddings` | `burn` (preferred) or `candle-core` | Rust compiler only (burn) / C++ (candle) | burn: No / candle: Yes |
