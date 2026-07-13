-- CivitForge: Container Registry Improvements
-- Migration 083
-- Adds repository-level policies for immutable tags, tag limits, retention,
-- vulnerability scanning config, signature verification, and pull-through cache.

CREATE TABLE IF NOT EXISTS container_repository_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    immutable_tags BOOLEAN NOT NULL DEFAULT false,
    max_tags INTEGER NOT NULL DEFAULT 100,
    retention_days INTEGER NOT NULL DEFAULT 90,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS container_vulnerability_scans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    manifest_digest TEXT NOT NULL,
    scanner TEXT NOT NULL DEFAULT 'osv',
    scan_status TEXT NOT NULL DEFAULT 'pending',
    total_vulns INTEGER NOT NULL DEFAULT 0,
    critical_count INTEGER NOT NULL DEFAULT 0,
    high_count INTEGER NOT NULL DEFAULT 0,
    medium_count INTEGER NOT NULL DEFAULT 0,
    low_count INTEGER NOT NULL DEFAULT 0,
    raw_results JSONB NOT NULL DEFAULT '{}',
    scanned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS container_image_signatures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    manifest_digest TEXT NOT NULL,
    signature_payload BYTEA NOT NULL,
    signer_key_id TEXT NOT NULL,
    signed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS container_pull_through_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    upstream_url TEXT NOT NULL,
    upstream_ref TEXT NOT NULL,
    local_digest TEXT NOT NULL,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    UNIQUE(repo_id, upstream_ref)
);

CREATE INDEX IF NOT EXISTS idx_container_repo_policies_repo ON container_repository_policies(repo_id);
CREATE INDEX IF NOT EXISTS idx_container_vuln_scans_repo_digest ON container_vulnerability_scans(repo_id, manifest_digest);
CREATE INDEX IF NOT EXISTS idx_container_vuln_scans_status ON container_vulnerability_scans(scan_status);
CREATE INDEX IF NOT EXISTS idx_container_image_signatures_repo ON container_image_signatures(repo_id, manifest_digest);
CREATE INDEX IF NOT EXISTS idx_container_pull_through_repo_ref ON container_pull_through_cache(repo_id, upstream_ref);
CREATE INDEX IF NOT EXISTS idx_container_pull_through_expires ON container_pull_through_cache(expires_at);
