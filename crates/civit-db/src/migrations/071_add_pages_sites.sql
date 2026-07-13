CREATE TABLE IF NOT EXISTS pages_sites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    branch TEXT NOT NULL DEFAULT 'main',
    path TEXT NOT NULL DEFAULT '/',
    public BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id)
);

CREATE INDEX IF NOT EXISTS idx_pages_sites_repo_id ON pages_sites(repo_id);
