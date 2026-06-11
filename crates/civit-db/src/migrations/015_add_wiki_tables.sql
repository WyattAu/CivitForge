-- Wiki pages metadata (actual content stored in .wiki.git bare repo via gitoxide)
CREATE TABLE IF NOT EXISTS wiki_pages (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    slug            TEXT NOT NULL,
    title           TEXT NOT NULL,
    format          TEXT NOT NULL DEFAULT 'markdown',
    content         TEXT NOT NULL DEFAULT '',
    latest_commit   TEXT NOT NULL DEFAULT 'pending',
    created_by      UUID NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, slug)
);

-- Wiki page edit history (tracked via git log, but indexed here for fast queries)
CREATE TABLE IF NOT EXISTS wiki_revisions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    page_id         UUID NOT NULL REFERENCES wiki_pages(id) ON DELETE CASCADE,
    commit_sha      TEXT NOT NULL,
    author_id       UUID NOT NULL REFERENCES users(id),
    edit_message    TEXT DEFAULT '',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_wiki_pages_repo ON wiki_pages(repo_id);
CREATE INDEX IF NOT EXISTS idx_wiki_pages_slug ON wiki_pages(repo_id, slug);
CREATE INDEX IF NOT EXISTS idx_wiki_revisions_page ON wiki_revisions(page_id);
