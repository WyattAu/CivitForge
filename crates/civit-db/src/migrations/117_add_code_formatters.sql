CREATE TABLE IF NOT EXISTS code_formatters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    language TEXT NOT NULL,
    formatter TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, language)
);

CREATE INDEX idx_code_formatters_repo_id ON code_formatters(repo_id);