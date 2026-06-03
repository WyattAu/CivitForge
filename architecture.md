# CivitForge Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the canonical architecture documentation.

## System Overview

```mermaid
flowchart TD
    subgraph Client_Layer ["Client Layer"]
        UI["Web UI (Leptos WASM + Tailwind)"]
        CLI["Git CLI (HTTP/SSH)"]
        FUSE["VFS (gRPC)"]
        ExtNode["External Federated Node"]
    end

    subgraph Core_Layer ["civit-core (Axum)"]
        Gateway["API Gateway + Auth"]
        GitEngine["Git Engine (gitoxide)"]
        VFS_RPC["VFS gRPC Server"]
        FedEngine["ForgeFed Sync"]
    end

    subgraph Brain_Layer ["civit-brain"]
        AST["AST Parser (tree-sitter)"]
        Embedder["Embedding Worker"]
        LLM["Inference Server (vLLM)"]
        Agent["PR Review Agent"]
    end

    subgraph Runner_Layer ["civit-runner"]
        Operator["K8s Runner Operator"]
        Podman["Rootless Podman"]
        Crypto["SBOM + Cosign"]
    end

    subgraph Data_Layer ["Storage"]
        DB[(PostgreSQL 17)]
        S3[(S3 / MinIO)]
        Redis["Redis 7"]
        Qdrant[(Qdrant)]
    end

    UI -->|REST| Gateway
    CLI -->|Git HTTP / russh| Gateway
    FUSE ==>|gRPC| VFS_RPC
    ExtNode <-->|ActivityPub / mTLS| FedEngine
    Gateway -->|CRUD| DB
    Gateway -->|Events| Redis
    Redis -.->|Trigger CI| Operator
    Redis -.->|Trigger Review| Agent
```

## Four Logical Domains

1. **CivitCore** -- HTTP/gRPC API, auth (JWT, RBAC, TOTP), gitoxide Git engine, SSH daemon, ForgeFed federation, events
2. **CivitRunner** -- CI/CD pipeline execution, K8s operator (kube-rs), rootless Podman sandbox, SLSA provenance
3. **CivitBrain** -- AST parsing (19 languages, 3-tier), RAG pipeline, vector DB (Qdrant), LLM inference
4. **CivitData** -- PostgreSQL 17 (relational), S3/MinIO (blob/LFS), Redis 7 (cache/pub-sub), Qdrant (vectors)
