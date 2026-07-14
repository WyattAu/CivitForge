CREATE TABLE IF NOT EXISTS api_gateway_routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    path TEXT NOT NULL,
    method TEXT NOT NULL,
    backend_url TEXT NOT NULL,
    rate_limit INTEGER NOT NULL DEFAULT 100,
    timeout_ms INTEGER NOT NULL DEFAULT 30000,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS api_gateway_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_hash TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    scopes TEXT[] NOT NULL DEFAULT '{}',
    rate_limit INTEGER NOT NULL DEFAULT 100,
    enabled BOOLEAN NOT NULL DEFAULT true,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_api_gateway_routes_path ON api_gateway_routes(path);
CREATE INDEX IF NOT EXISTS idx_api_gateway_routes_enabled ON api_gateway_routes(enabled);
CREATE INDEX IF NOT EXISTS idx_api_gateway_keys_key_hash ON api_gateway_keys(key_hash);
CREATE INDEX IF NOT EXISTS idx_api_gateway_keys_enabled ON api_gateway_keys(enabled);
