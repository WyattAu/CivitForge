CREATE TABLE IF NOT EXISTS scim_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_scim_tokens_provider_id ON scim_tokens (provider_id);
CREATE INDEX IF NOT EXISTS idx_scim_tokens_token_hash ON scim_tokens (token_hash);
