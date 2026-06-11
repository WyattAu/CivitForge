-- CivitForge: Repo Collaborators
-- Migration 044

CREATE TABLE IF NOT EXISTS repo_collaborators (
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission TEXT NOT NULL DEFAULT 'read',
    PRIMARY KEY (repo_id, user_id),
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_repo_collaborators_repo ON repo_collaborators(repo_id);
CREATE INDEX idx_repo_collaborators_user ON repo_collaborators(user_id);
