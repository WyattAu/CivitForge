CREATE TABLE IF NOT EXISTS api_docs_v5 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint TEXT NOT NULL,
    method TEXT NOT NULL,
    version TEXT NOT NULL DEFAULT 'v4',
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
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(endpoint, method, version)
);

CREATE INDEX IF NOT EXISTS idx_api_docs_v5_endpoint ON api_docs_v5(endpoint);
CREATE INDEX IF NOT EXISTS idx_api_docs_v5_method ON api_docs_v5(method);
CREATE INDEX IF NOT EXISTS idx_api_docs_v5_version ON api_docs_v5(version);
CREATE INDEX IF NOT EXISTS idx_api_docs_v5_tags ON api_docs_v5 USING GIN(tags);
CREATE INDEX IF NOT EXISTS idx_api_docs_v5_deprecated ON api_docs_v5(deprecated);
