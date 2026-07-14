CREATE TABLE IF NOT EXISTS pipeline_action_installations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action_id UUID NOT NULL REFERENCES pipeline_actions(id) ON DELETE CASCADE,
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    installed_by UUID NOT NULL REFERENCES users(id),
    version TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}',
    installed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_action_installations_repo ON pipeline_action_installations(repo_id);
CREATE INDEX idx_action_installations_action ON pipeline_action_installations(action_id);
