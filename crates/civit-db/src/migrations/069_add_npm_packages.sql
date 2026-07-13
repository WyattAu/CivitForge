CREATE TABLE IF NOT EXISTS npm_packages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    dist_tags JSONB NOT NULL DEFAULT '{}',
    readme TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, name, version)
);

CREATE TABLE IF NOT EXISTS npm_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    package_id UUID NOT NULL REFERENCES npm_packages(id) ON DELETE CASCADE,
    version TEXT NOT NULL,
    tarball_url TEXT NOT NULL,
    shasum TEXT NOT NULL,
    integrity TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
