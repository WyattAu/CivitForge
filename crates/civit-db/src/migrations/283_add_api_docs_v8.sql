CREATE TABLE IF NOT EXISTS api_docs_v8 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint TEXT NOT NULL,
    method TEXT NOT NULL,
    version TEXT NOT NULL DEFAULT 'v7',
    summary TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    parameters JSONB NOT NULL DEFAULT '[]',
    request_body JSONB,
    responses JSONB NOT NULL DEFAULT '{}',
    examples JSONB NOT NULL DEFAULT '{}',
    tags TEXT[] NOT NULL DEFAULT '{}',
    deprecated BOOLEAN NOT NULL DEFAULT false,
    changelog TEXT NOT NULL DEFAULT '',
    security_schemes JSONB NOT NULL DEFAULT '[]',
    rate_limits JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(endpoint, method, version)
);

CREATE INDEX IF NOT EXISTS idx_api_docs_v8_endpoint ON api_docs_v8(endpoint);
CREATE INDEX IF NOT EXISTS idx_api_docs_v8_method ON api_docs_v8(method);
CREATE INDEX IF NOT EXISTS idx_api_docs_v8_version ON api_docs_v8(version);
CREATE INDEX IF NOT EXISTS idx_api_docs_v8_tags ON api_docs_v8 USING GIN(tags);
CREATE INDEX IF NOT EXISTS idx_api_docs_v8_deprecated ON api_docs_v8(deprecated);
