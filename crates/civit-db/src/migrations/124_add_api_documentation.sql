CREATE TABLE IF NOT EXISTS api_documentation (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint TEXT NOT NULL,
    method TEXT NOT NULL,
    summary TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    parameters JSONB NOT NULL DEFAULT '[]',
    request_body JSONB,
    responses JSONB NOT NULL DEFAULT '{}',
    tags TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(endpoint, method)
);

CREATE INDEX IF NOT EXISTS idx_api_documentation_endpoint ON api_documentation(endpoint);
CREATE INDEX IF NOT EXISTS idx_api_documentation_method ON api_documentation(method);
CREATE INDEX IF NOT EXISTS idx_api_documentation_tags ON api_documentation USING GIN(tags);
