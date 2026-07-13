-- Cache entries table for persistent caching
-- Migration 086

CREATE TABLE IF NOT EXISTS cache_entries (
    key TEXT PRIMARY KEY,
    value JSONB NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_cache_entries_expires ON cache_entries(expires_at);
