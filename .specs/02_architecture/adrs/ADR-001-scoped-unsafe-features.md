| TypeScript/JavaScript | `swc_ecma_parser` | No (transitive via `swc_common`; audited, minimal) |
3. **No `unsafe` in default build**: `cargo build` without any features produces the exact same codebase as today; zero `unsafe`, zero new dependencies.
   - `treesitter`; enables tree-sitter C FFI AST parsing for all supported grammars
   - `fuse-mount`; enables FUSE kernel filesystem mount via `fuser`
   - `pkcs11-hsm`; enables real PKCS#11 HSM via `cryptoki`
   - `ml-embeddings`; enables ML embeddings via `burn` (preferred) or `candle-rs`
Tier 3 (Regex fallback):      existing RegexAstParser; always available, no features needed