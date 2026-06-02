-- CivitForge Phase 10: OCI Container Registry
-- Migration 011
-- Adds OCI Distribution Spec v1.1 tables: repositories, blobs, manifests, tags,
-- referrers, image_signatures (cosign), vulnerability scan results, and
-- per-image RBAC policies.

-- OCI Repositories (namespaces: org/image or user/image)
CREATE TABLE IF NOT EXISTS oci_repositories (
    id              BIGSERIAL PRIMARY KEY,
    name            TEXT NOT NULL,              -- e.g. "myorg/alpine"
    namespace_type  TEXT NOT NULL DEFAULT 'org', -- 'org' or 'user'
    namespace_id    TEXT NOT NULL,              -- org_id or user_id
    description     TEXT,
    visibility      TEXT NOT NULL DEFAULT 'private', -- 'public' | 'private'
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(name)
);

-- OCI Blobs (content-addressable storage metadata)
CREATE TABLE IF NOT EXISTS oci_blobs (
    id              BIGSERIAL PRIMARY KEY,
    repo_id         BIGINT NOT NULL REFERENCES oci_repositories(id) ON DELETE CASCADE,
    digest          TEXT NOT NULL,              -- e.g. "sha256:abc123..."
    media_type      TEXT NOT NULL,              -- e.g. "application/vnd.oci.image.layer.v1.tar+gzip"
    size_bytes      BIGINT NOT NULL,
    storage_path    TEXT NOT NULL,              -- filesystem path or object storage key
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, digest)
);

-- OCI Manifests
CREATE TABLE IF NOT EXISTS oci_manifests (
    id              BIGSERIAL PRIMARY KEY,
    repo_id         BIGINT NOT NULL REFERENCES oci_repositories(id) ON DELETE CASCADE,
    digest          TEXT NOT NULL,              -- e.g. "sha256:manifest123..."
    media_type      TEXT NOT NULL,              -- "application/vnd.oci.image.manifest.v1+json" or index
    raw_json        BYTEA NOT NULL,             -- full manifest JSON
    config_digest   TEXT,                       -- for image manifests, the config blob digest
    config_size     BIGINT,                     -- config blob size
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, digest)
);

-- OCI Tags (mutable pointers to manifests by digest)
CREATE TABLE IF NOT EXISTS oci_tags (
    id              BIGSERIAL PRIMARY KEY,
    repo_id         BIGINT NOT NULL REFERENCES oci_repositories(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    manifest_digest TEXT NOT NULL,              -- digest this tag currently points to
    immutable       BOOLEAN NOT NULL DEFAULT FALSE, -- if true, tag cannot be moved
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, name)
);

-- Manifest-to-blob relationship (layers + config)
CREATE TABLE IF NOT EXISTS oci_manifest_layers (
    id              BIGSERIAL PRIMARY KEY,
    manifest_id     BIGINT NOT NULL REFERENCES oci_manifests(id) ON DELETE CASCADE,
    blob_digest     TEXT NOT NULL,              -- references oci_blobs.digest
    blob_size       BIGINT NOT NULL,
    media_type      TEXT NOT NULL,
    annotation_key  TEXT,
    annotation_val  TEXT,
    sort_order      INT NOT NULL DEFAULT 0,     -- layer order
    UNIQUE(manifest_id, blob_digest)
);

-- Image Signatures (cosign)
CREATE TABLE IF NOT EXISTS oci_image_signatures (
    id              BIGSERIAL PRIMARY KEY,
    repo_id         BIGINT NOT NULL REFERENCES oci_repositories(id) ON DELETE CASCADE,
    manifest_digest TEXT NOT NULL,              -- the signed manifest
    signature_blob  BYTEA NOT NULL,             -- cosign signature payload
    signer_key_id   TEXT NOT NULL,              -- key fingerprint
    signed_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Vulnerability Scan Results (OSV-based)
CREATE TABLE IF NOT EXISTS oci_vuln_scans (
    id              BIGSERIAL PRIMARY KEY,
    repo_id         BIGINT NOT NULL REFERENCES oci_repositories(id) ON DELETE CASCADE,
    manifest_digest TEXT NOT NULL,
    scanner         TEXT NOT NULL DEFAULT 'osv', -- scanner identifier
    scan_status     TEXT NOT NULL DEFAULT 'pending', -- 'pending' | 'completed' | 'failed'
    total_vulns     INT NOT NULL DEFAULT 0,
    critical_count  INT NOT NULL DEFAULT 0,
    high_count      INT NOT NULL DEFAULT 0,
    medium_count    INT NOT NULL DEFAULT 0,
    low_count       INT NOT NULL DEFAULT 0,
    raw_results     BYTEA,                       -- JSON scan results
    scanned_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Per-image RBAC policies
CREATE TABLE IF NOT EXISTS oci_policies (
    id              BIGSERIAL PRIMARY KEY,
    repo_id         BIGINT NOT NULL REFERENCES oci_repositories(id) ON DELETE CASCADE,
    role            TEXT NOT NULL,              -- 'reader' | 'writer' | 'admin'
    entity_type     TEXT NOT NULL,              -- 'user' | 'org' | 'team' | 'public'
    entity_id       TEXT,                       -- user_id or org_id (NULL for 'public')
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, role, entity_type, entity_id)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_oci_repos_namespace ON oci_repositories(namespace_type, namespace_id);
CREATE INDEX IF NOT EXISTS idx_oci_repos_visibility ON oci_repositories(visibility);
CREATE INDEX IF NOT EXISTS idx_oci_blobs_repo ON oci_blobs(repo_id);
CREATE INDEX IF NOT EXISTS idx_oci_blobs_digest ON oci_blobs(digest);
CREATE INDEX IF NOT EXISTS idx_oci_manifests_repo ON oci_manifests(repo_id);
CREATE INDEX IF NOT EXISTS idx_oci_manifests_digest ON oci_manifests(digest);
CREATE INDEX IF NOT EXISTS idx_oci_tags_repo ON oci_tags(repo_id);
CREATE INDEX IF NOT EXISTS idx_oci_tags_name ON oci_tags(repo_id, name);
CREATE INDEX IF NOT EXISTS idx_oci_tags_digest ON oci_tags(manifest_digest);
CREATE INDEX IF NOT EXISTS idx_oci_layers_manifest ON oci_manifest_layers(manifest_id);
CREATE INDEX IF NOT EXISTS idx_oci_signatures_manifest ON oci_image_signatures(repo_id, manifest_digest);
CREATE INDEX IF NOT EXISTS idx_oci_vulns_manifest ON oci_vuln_scans(repo_id, manifest_digest);
CREATE INDEX IF NOT EXISTS idx_oci_policies_repo ON oci_policies(repo_id);
