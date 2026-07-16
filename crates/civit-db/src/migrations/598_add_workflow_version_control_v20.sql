-- Workflow Version Control v20: version control, branches
CREATE TABLE IF NOT EXISTS workflow_version_control_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    definition JSONB NOT NULL,
    change_description TEXT NOT NULL DEFAULT '',
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(workflow_id, version)
);

CREATE INDEX IF NOT EXISTS idx_workflow_version_control_v20_workflow ON workflow_version_control_v20(workflow_id);
CREATE INDEX IF NOT EXISTS idx_workflow_version_control_v20_version ON workflow_version_control_v20(workflow_id, version DESC);

CREATE TABLE IF NOT EXISTS workflow_branches_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    parent_version INTEGER,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(workflow_id, name)
);

CREATE INDEX IF NOT EXISTS idx_workflow_branches_v20_workflow ON workflow_branches_v20(workflow_id);
