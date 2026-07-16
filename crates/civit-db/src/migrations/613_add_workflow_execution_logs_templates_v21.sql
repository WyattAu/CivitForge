CREATE TABLE IF NOT EXISTS workflow_execution_logs_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    execution_id UUID NOT NULL,
    step_name TEXT NOT NULL,
    step_status TEXT NOT NULL,
    input_data JSONB NOT NULL DEFAULT '{}',
    output_data JSONB NOT NULL DEFAULT '{}',
    error_message TEXT,
    duration_ms INTEGER,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS workflow_templates_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT 'general',
    definition JSONB NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    usage_count INTEGER NOT NULL DEFAULT 0,
    rating DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workflow_execution_logs_v21_workflow_id ON workflow_execution_logs_v21(workflow_id);
CREATE INDEX IF NOT EXISTS idx_workflow_execution_logs_v21_execution_id ON workflow_execution_logs_v21(execution_id);
CREATE INDEX IF NOT EXISTS idx_workflow_execution_logs_v21_step_status ON workflow_execution_logs_v21(step_status);
CREATE INDEX IF NOT EXISTS idx_workflow_execution_logs_v21_started_at ON workflow_execution_logs_v21(started_at);
CREATE INDEX IF NOT EXISTS idx_workflow_templates_v20_name ON workflow_templates_v20(name);
CREATE INDEX IF NOT EXISTS idx_workflow_templates_v20_category ON workflow_templates_v20(category);
