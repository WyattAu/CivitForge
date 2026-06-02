-- Code Search: file index metadata (content indexed via PostgreSQL full-text search for v1.0)
-- tantivy integration deferred to v1.1

-- File index entries (updated on git push)
CREATE TABLE IF NOT EXISTS code_search_index (
    id              BIGSERIAL PRIMARY KEY,
    repo_id         BIGINT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    file_path       TEXT NOT NULL,               -- relative path from repo root
    language        TEXT,                        -- programming language
    content         TEXT,                        -- file content (for small files, truncated at 100KB)
    line_count      INT NOT NULL DEFAULT 0,
    byte_size       BIGINT NOT NULL DEFAULT 0,
    commit_sha      TEXT NOT NULL,               -- last indexed commit
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Search tokens (trigram index for substring search)
CREATE TABLE IF NOT EXISTS code_search_tokens (
    id              BIGSERIAL PRIMARY KEY,
    index_id        BIGINT NOT NULL REFERENCES code_search_index(id) ON DELETE CASCADE,
    token           TEXT NOT NULL,               -- word/token
    line_number     INT NOT NULL,
    line_content    TEXT NOT NULL                -- the actual line
);

CREATE INDEX IF NOT EXISTS idx_code_search_repo ON code_search_index(repo_id);
CREATE INDEX IF NOT EXISTS idx_code_search_path ON code_search_index(repo_id, file_path);
CREATE INDEX IF NOT EXISTS idx_code_search_language ON code_search_index(repo_id, language);
CREATE INDEX IF NOT EXISTS idx_code_tokens_token ON code_search_tokens(token);
CREATE INDEX IF NOT EXISTS idx_code_tokens_index ON code_search_tokens(index_id);
