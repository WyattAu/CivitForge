-- CivitForge Phase 137: Data Archival
-- Migration 137
-- Adds tables for tracking data archives with retention policies.

CREATE TABLE IF NOT EXISTS data_archives (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    archive_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    file_path TEXT,
    file_size_bytes BIGINT NOT NULL DEFAULT 0,
    retention_days INTEGER NOT NULL DEFAULT 365,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_data_archives_repo_id ON data_archives(repo_id);
CREATE INDEX IF NOT EXISTS idx_data_archives_status ON data_archives(status);
CREATE INDEX IF NOT EXISTS idx_data_archives_type ON data_archives(archive_type);
CREATE INDEX IF NOT EXISTS idx_data_archives_expires ON data_archives(expires_at);
CREATE INDEX IF NOT EXISTS idx_data_archives_repo_status ON data_archives(repo_id, status);
