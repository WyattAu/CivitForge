CREATE TABLE IF NOT EXISTS api_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    version TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'active',
    release_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deprecation_date TIMESTAMPTZ,
    sunset_date TIMESTAMPTZ,
    changelog TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_api_versions_version ON api_versions(version);
CREATE INDEX IF NOT EXISTS idx_api_versions_status ON api_versions(status);
