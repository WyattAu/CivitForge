-- API Documentation v15
CREATE TABLE IF NOT EXISTS api_docs_v15 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint TEXT NOT NULL,
    method TEXT NOT NULL,
    version TEXT NOT NULL DEFAULT 'v14',
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

CREATE INDEX IF NOT EXISTS idx_api_docs_v15_endpoint ON api_docs_v15(endpoint);
CREATE INDEX IF NOT EXISTS idx_api_docs_v15_version ON api_docs_v15(version);
CREATE INDEX IF NOT EXISTS idx_api_docs_v15_tags ON api_docs_v15 USING GIN(tags);
CREATE INDEX IF NOT EXISTS idx_api_docs_v15_deprecated ON api_docs_v15(deprecated);
