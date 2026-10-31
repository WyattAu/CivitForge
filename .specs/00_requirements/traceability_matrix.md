**Scope:** Phase 0 (Requirements Engineering); Initial Matrix
| REQ-VCS-001 | REQ-1.1 | CivitCore | civit-core (VFS gRPC) | Integration, E2E | T-VFS-001 | - | 1 | Critical |
| REQ-VCS-002 | REQ-1.1 | CivitCore | civit-vfs (FUSE daemon) | E2E, Performance | T-VFS-002 | - | 1 | High |
| REQ-VCS-003 | REQ-1.2 | CivitCore | civit-core (git engine) | Unit, Integration | T-GIT-001 | - | 1 | Critical |
| REQ-VCS-004 | REQ-1.2 | CivitCore | civit-core (pack engine) | Performance, Benchmark | T-GIT-002 | - | 1 | High |
| REQ-VCS-006 | REQ-1.3 | CivitCore | civit-core (VCS adapter) | Integration | T-VCS-003 | - | 4 | Low |
| REQ-VCS-007 | REQ-1.4 | CivitCore | civit-core (build graph) | Unit, Integration | T-BG-001 | - | 1 | High |
| REQ-VCS-008 | REQ-1.4 | CivitCore | civit-core (CI trigger) | Integration, E2E | T-BG-002 | - | 1 | High |
| REQ-LFS-001 | REQ-2.1 | CivitData | civit-data (chunk engine) | Unit, Property | T-LFS-001 | - | 2 | Critical |
| REQ-LFS-003 | REQ-2.1 | CivitData | civit-data (delta engine) | Unit, Integration | T-LFS-003 | - | 2 | Critical |
| REQ-LFS-004 | REQ-2.1 | CivitData | civit-data (chunk engine) | Performance | T-LFS-004 | - | 2 | High |
| REQ-LFS-007 | REQ-2.3 | CivitRunner | civit-runner (CSI driver) | Performance | T-CSI-002 | - | 2 | High |
| REQ-LFS-008 | REQ-2.1 | CivitData | civit-data (stream engine) | E2E, Performance | T-LFS-005 | - | 2 | High |
| REQ-CI-003 | REQ-3.2 | CivitRunner | civit-runner (K8s operator) | Integration, E2E | T-CI-003 | - | 2 | Critical |
| REQ-CI-004 | REQ-3.2 | CivitRunner | civit-runner (event consumer) | Integration, Performance | T-CI-004 | - | 2 | High |
| REQ-CI-009 | REQ-3.5 | CivitRunner | civit-runner (CDE operator) | E2E | T-CI-009 | - | 4 | Medium |
| REQ-CI-010 | REQ-3.5 | CivitRunner | civit-runner (CDE sync) | Integration | T-CI-010 | - | 4 | Medium |
| REQ-CI-011 | REQ-3.2 | CivitRunner | civit-runner (K8s scheduler) | Performance, Load | T-CI-011 | - | 2 | High |
| REQ-AI-003 | REQ-4.2 | CivitBrain | civit-brain (AST parser) | Unit, Integration | T-AI-003 | - | 3 | Critical |
| REQ-AI-004 | REQ-4.2 | CivitBrain | civit-brain (embedding worker) | Integration, Performance | T-AI-004 | - | 3 | High |
| REQ-AI-005 | REQ-4.2 | CivitBrain | civit-brain (RAG engine) | Performance, Benchmark | T-AI-005 | - | 3 | High |
| REQ-AI-006 | REQ-4.1 | CivitBrain | civit-brain (vLLM server) | Integration, E2E | T-AI-006 | - | 3 | Critical |
| REQ-AI-007 | REQ-4.2 | CivitBrain | civit-brain (review agent) | Performance, E2E | T-AI-007 | - | 3 | High |
| REQ-AI-008 | REQ-4.3 | CivitBrain | civit-brain (agent framework) | Integration, E2E | T-AI-008 | - | 3 | High |
| REQ-AI-009 | REQ-4.3 | CivitBrain | civit-brain (PR review agent) | E2E | T-AI-009 | - | 3 | High |
| REQ-FED-002 | REQ-5.2 | CivitData | CockroachDB | Performance | T-FED-002 | - | 4 | High |
| REQ-FED-003 | REQ-5.1 | CivitCore | civit-core (federation engine) | Performance, E2E | T-FED-003 | - | 4 | High |
| REQ-FED-005 | REQ-5.1 | CivitCore | civit-core (DAG sync) | Integration, E2E | T-FED-005 | - | 4 | Critical |
| REQ-FED-006 | REQ-5.1 | CivitCore | civit-core (DAG sync) | Performance, Load | T-FED-006 | - | 4 | High |
| REQ-FED-008 | REQ-5.2 | CivitData | civit-data (edge cache) | Integration, Performance | T-FED-008 | - | 4 | High |
| REQ-FED-009 | REQ-5.2 | CivitData | civit-data (edge cache) | Integration, E2E | T-FED-009 | - | 4 | Medium |
| REQ-FED-010 | REQ-5.2 | CivitData | civit-data (edge cache) | Performance | T-FED-010 | - | 4 | High |
| NFR-PERF-001 | CivitCore | Performance, Benchmark | T-NFR-007 | - | 1 | High |
| NFR-PERF-002 | CivitCore | Load, Chaos | T-NFR-008 | - | 4 | High |
| NFR-PERF-003 | CivitCore, CivitData | Performance, E2E | T-NFR-009 | - | 1 | High |
| NFR-PERF-004 | CivitCore | Load, Benchmark | T-NFR-010 | - | 2 | High |
| NFR-DEPLOY-001 | All | Integration, E2E | T-NFR-014 | - | 1 | High |
| NFR-DEPLOY-002 | All | Integration, E2E | T-NFR-015 | - | 1 | High |
| NFR-DEPLOY-003 | CivitBrain | Integration | T-NFR-016 | - | 3 | Medium |