### The CivitForge Architecture Diagram

```mermaid
flowchart TD
    classDef edge fill:#2D3748,stroke:#4A5568,stroke-width:2px,color:#fff
    classDef core fill:#2B6CB0,stroke:#2C5282,stroke-width:2px,color:#fff
    classDef runner fill:#805AD5,stroke:#553C9A,stroke-width:2px,color:#fff
    classDef brain fill:#38A169,stroke:#276749,stroke-width:2px,color:#fff
    classDef data fill:#C53030,stroke:#9B2C2C,stroke-width:2px,color:#fff
    classDef external fill:#1A202C,stroke:#718096,stroke-width:2px,color:#fff,stroke-dasharray: 5 5

    subgraph Client_Layer ["Edge and Client Layer"]
        UI["Web UI (Wasm/React)"]:::edge
        CLI["Git CLI (HTTP/SSH)"]:::edge
        FUSE["Civit VFS (Local FUSE)"]:::edge
        ExtNode["External Federated Node"]:::external
    end

    subgraph Core_Layer ["CivitCore (Rust Application Layer)"]
        Gateway["Axum API Gateway and Auth"]:::core
        GitEngine["Git Engine (gitoxide)"]:::core
        VFS_RPC["VFS gRPC Server"]:::core
        FedEngine["ForgeFed Sync Engine (DAG)"]:::core
        
        Gateway <--> GitEngine
        Gateway <--> VFS_RPC
        Gateway <--> FedEngine
    end

    subgraph Brain_Layer ["CivitBrain (Local AI Layer)"]
        AST["AST Parser (tree-sitter)"]:::brain
        Embedder["Embedding Worker"]:::brain
        LLM["Inference Server (vLLM)"]:::brain
        Agent["AI PR Review Agent"]:::brain
        
        AST --> Embedder
        Agent <--> LLM
    end

    subgraph Runner_Layer ["CivitRunner (K8s Orchestration)"]
        Operator["K8s Runner Operator (Rust)"]:::runner
        Podman["Rootless Podman Sandboxes"]:::runner
        Crypto["SBOM and Cosign Signer"]:::runner
        
        Operator --> Podman
        Podman --> Crypto
    end

    subgraph Data_Layer ["CivitData (Distributed Storage)"]
        DB[(CockroachDB <br/> Relational Meta)]:::data
        S3[(MinIO / S3 <br/> LFS+ and Git Blobs)]:::data
        Redis["Redis / Dragonfly <br/> Event PubSub"]:::data
        Qdrant[(Qdrant <br/> Vector Database)]:::data
    end

    UI -->|GraphQL / REST| Gateway
    CLI -->|Git HTTP / russh| Gateway
    FUSE ==>|gRPC On-Demand Fetch| VFS_RPC
    ExtNode <-->|ActivityPub / mTLS| FedEngine

    Gateway -->|Read/Write State| DB
    GitEngine ==>|Packfiles / Chunks| S3
    FedEngine -->|Sync State| DB

    GitEngine -.->|Push Event| Redis
    Redis -.->|Trigger CI Pipeline| Operator
    Redis -.->|Trigger AST Parse| AST
    Redis -.->|Trigger Code Review| Agent

    Podman ==>|CSI Native Mount| S3
    Podman -->|Write Build Artifacts| S3
    Crypto -->|Store SBOM/Signatures| DB

    Embedder ==>|Store Code Vectors| Qdrant
    LLM <-->|RAG Context Retrieval| Qdrant
    Agent -->|Post Comments/PRs| Gateway
    Agent <-->|Request Sandbox Execution| Operator

```

### Architecture Flow Description

The following describes the data flow through each subsystem during standard engineering workflows.

#### 1. Push to Monorepo

1.  A developer pushes via the Git CLI over SSH or HTTP.
2.  The request reaches the Axum API Gateway (CivitCore), which authenticates the caller via OIDC.
3.  The Git Engine (`gitoxide`) receives the payload and parallelizes object unpacking.
4.  Standard Git objects and deduplicated LFS+ chunks are streamed to S3/MinIO. Metadata (commit hashes, PR state transitions) is written to CockroachDB.
5.  On successful persistence, CivitCore emits a `CodePushed` event to Redis.

#### 2. AI RAG and Review (CivitBrain)

1.  Redis broadcasts the `CodePushed` event.
2.  The AST Parser consumes the event, fetches the diff, and decomposes the changed source into an abstract syntax tree via `tree-sitter`.
3.  The Embedding Worker segments the AST into semantic vectors and persists them in Qdrant.
4.  Concurrently, the AI PR Review Agent is triggered. It invokes `vLLM` to analyze the diff, querying Qdrant for repository-wide context (e.g., error-handling patterns, API contract conformance).
5.  The Agent posts its findings to the PR via the Core API.

#### 3. Secure CI/CD (CivitRunner)

1.  The K8s Operator consumes pipeline-trigger events from Redis.
2.  It schedules ephemeral Kubernetes Pods. CI tasks execute inside these pods via rootless Podman, using user namespaces to prevent container-escape attacks.
3.  When a pipeline requires large ML datasets, K8s CSI mounts the S3 buckets directly into the Podman container, bypassing network-based data transfer.
4.  On build completion, the Crypto Worker generates an SBOM, signs the image with Cosign/Sigstore, and pushes the signed artifact to S3.

#### 4. Federation

1.  When a PR is created, the ForgeFed Sync Engine checks whether the repository is shared with other geographic nodes (e.g., Tokyo, London) or external organizations.
2.  It propagates metadata and negotiates missing Git objects via an asynchronous DAG protocol over mTLS, achieving eventual consistency across all nodes without blocking the local writer.
