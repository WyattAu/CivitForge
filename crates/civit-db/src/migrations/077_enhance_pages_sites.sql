-- Migration 077: Enhance Pages Sites
-- Adds custom domain, HTTPS certificate, build pipeline, and preview deployments.

ALTER TABLE pages_sites ADD COLUMN IF NOT EXISTS custom_domain TEXT;
ALTER TABLE pages_sites ADD COLUMN IF NOT EXISTS https_enabled BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE pages_sites ADD COLUMN IF NOT EXISTS last_built_at TIMESTAMPTZ;

CREATE TABLE IF NOT EXISTS pages_deployments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    site_id UUID NOT NULL REFERENCES pages_sites(id) ON DELETE CASCADE,
    sha TEXT NOT NULL,
    url TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pages_deployments_site_id ON pages_deployments(site_id);
CREATE INDEX IF NOT EXISTS idx_pages_deployments_status ON pages_deployments(status);
