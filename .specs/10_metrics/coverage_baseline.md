# Coverage Baseline Report

**Date:** 2025-06-20
**Tool:** `cargo-llvm-cov` v0.8.7 (LLVM 22.1.6)
**Rust Toolchain:** 1.88

## Overall Workspace Coverage

| Metric | Total | Covered | Percentage |
|--------|------:|--------:|-----------|
| Lines | 343,138 | 105,581 | **30.8%** |
| Functions | 20,603 | 6,206 | **30.1%** |

## Per-Crate Coverage

| Crate | Lines | Covered | Line % | Functions | F-Covered | Func % |
|-------|------:|--------:|-------:|----------:|----------:|-------:|
| civit-brain | 35,086 | 17,504 | **49.9%** | 1,456 | 1,289 | **88.5%** |
| civit-crypto | 27,578 | 13,787 | **50.0%** | 865 | 780 | **90.2%** |
| civit-git | 6,989 | 3,261 | **46.7%** | 145 | 138 | **95.2%** |
| civit-core | 166,774 | 61,437 | **36.8%** | 8,369 | 3,360 | **40.1%** |
| civit-ci | 4,562 | 2,159 | **47.3%** | 280 | 112 | **40.0%** |
| civit-pipeline | 5,029 | 2,279 | **45.3%** | 196 | 116 | **59.2%** |
| civit-auth | 6,237 | 2,878 | **46.1%** | 396 | 251 | **63.4%** |
| civit-db | 8,559 | 2,205 | **25.8%** | 1,047 | 151 | **14.4%** |
| civit-shared | 828 | 71 | **8.6%** | 227 | 9 | **4.0%** |
| civit-runner | 22,912 | 0 | **0.0%** | 819 | 0 | **0.0%** |
| civit-shard | 3,944 | 0 | **0.0%** | 160 | 0 | **0.0%** |
| civit-storage | 3,680 | 0 | **0.0%** | 205 | 0 | **0.0%** |
| civit-ui | 46,140 | 0 | **0.0%** | 6,223 | 0 | **0.0%** |
| civit-vfs | 4,820 | 0 | **0.0%** | 215 | 0 | **0.0%** |

## Test Summary

| Metric | Count |
|--------|------:|
| Tests Passing | 3,883 |
| Tests Failing | 0 |
| Tests Ignored | 118 |
| **Pass Rate** | **100%** |

## Lowest Coverage Files (civit-core)

| File | Lines | Covered | Line % |
|------|------:|--------:|-------:|
| api/artifact_serving.rs | 253 | 0 | 0.0% |
| api/deploy_keys.rs | 187 | 0 | 0.0% |
| api/git_http.rs | 321 | 5 | 1.6% |
| api/lfs.rs | 360 | 0 | 0.0% |
| api/mirrors.rs | 383 | 0 | 0.0% |
| api/notifications.rs | 137 | 0 | 0.0% |
| api/oci.rs | 663 | 0 | 0.0% |
| api/pipeline_caches.rs | 147 | 0 | 0.0% |
| api/pipeline_schedules.rs | 549 | 0 | 0.0% |
| api/pipeline_secrets.rs | 212 | 0 | 0.0% |
| api/tokens.rs | 192 | 0 | 0.0% |
| api/webhooks.rs | 450 | 0 | 0.0% |
| scheduler.rs | 572 | 78 | 13.6% |
| api/repos.rs | 1,533 | 86 | 5.6% |
| api/pull_requests.rs | 1,738 | 345 | 19.9% |

## Recommendations

### Priority 1: Crates at 0% Coverage
- **civit-runner** (22,912 lines) - Entire crate untested. Add unit tests for core modules.
- **civit-ui** (46,140 lines) - No Rust-side tests. Consider adding component render tests.
- **civit-vfs** (4,820 lines) - Add tests for cache, store, and gRPC modules.
- **civit-storage** (3,680 lines) - Add tests for artifacts, lfs, mirrors, oci modules.
- **civit-shard** (3,944 lines) - Add tests for coordination, migration, ring, router.

### Priority 2: civit-core API Modules
- 14 API modules at 0% coverage (2,288 lines)
- `scheduler.rs` at 13.6% (572 lines) needs dedicated tests
- `repos.rs` at 5.6% (1,533 lines) is the largest low-coverage file

### Priority 3: civit-db
- 25.8% line coverage, 14.4% function coverage
- `repository.rs` (2,330 lines, 15.4% func coverage) is the largest gap
- `pool.rs` and `session.rs` also need attention

### Priority 4: civit-shared
- 8.6% line coverage on shared types
- Critical for type safety across the workspace
