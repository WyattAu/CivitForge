CREATE TABLE IF NOT EXISTS saml_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    entity_id TEXT NOT NULL,
    sso_url TEXT NOT NULL,
    certificate TEXT NOT NULL,
    metadata_url TEXT,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
