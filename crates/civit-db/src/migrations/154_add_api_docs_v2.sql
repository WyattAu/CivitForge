CREATE TABLE IF NOT EXISTS api_docs_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint TEXT NOT NULL,
    method TEXT NOT NULL,
    version TEXT NOT NULL DEFAULT 'v1',
    summary TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    parameters JSONB NOT NULL DEFAULT '[]',
    request_body JSONB,
    responses JSONB NOT NULL DEFAULT '{}',
    examples JSONB NOT NULL DEFAULT '{}',
    tags TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(endpoint, method, version)
);

CREATE INDEX IF NOT EXISTS idx_api_docs_v2_endpoint ON api_docs_v2(endpoint);
CREATE INDEX IF NOT EXISTS idx_api_docs_v2_method ON api_docs_v2(method);
CREATE INDEX IF NOT EXISTS idx_api_docs_v2_version ON api_docs_v2(version);
CREATE INDEX IF NOT EXISTS idx_api_docs_v2_tags ON api_docs_v2 USING GIN(tags);
