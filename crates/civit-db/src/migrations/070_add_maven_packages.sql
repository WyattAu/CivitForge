CREATE TABLE IF NOT EXISTS maven_packages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    group_id TEXT NOT NULL,
    artifact_id TEXT NOT NULL,
    version TEXT NOT NULL,
    packaging TEXT NOT NULL DEFAULT 'jar',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, group_id, artifact_id, version)
);
