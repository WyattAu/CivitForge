-- CDN configuration table
-- Migration 087

CREATE TABLE IF NOT EXISTS cdn_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider TEXT NOT NULL,
    api_key_encrypted TEXT NOT NULL,
    zone_id TEXT,
    enabled BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
