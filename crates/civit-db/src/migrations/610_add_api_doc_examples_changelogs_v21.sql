CREATE TABLE IF NOT EXISTS api_doc_examples_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint_id UUID NOT NULL REFERENCES api_endpoints(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    language TEXT NOT NULL DEFAULT 'curl',
    request_example TEXT NOT NULL,
    response_example TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS api_doc_changelogs_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint_id UUID NOT NULL REFERENCES api_endpoints(id) ON DELETE CASCADE,
    version TEXT NOT NULL,
    change_type TEXT NOT NULL,
    description TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_api_doc_examples_v21_endpoint ON api_doc_examples_v21(endpoint_id);
CREATE INDEX IF NOT EXISTS idx_api_doc_examples_v21_language ON api_doc_examples_v21(language);
CREATE INDEX IF NOT EXISTS idx_api_doc_changelogs_v21_endpoint ON api_doc_changelogs_v21(endpoint_id);
CREATE INDEX IF NOT EXISTS idx_api_doc_changelogs_v21_version ON api_doc_changelogs_v21(version);
