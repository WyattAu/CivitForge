-- CivitForge: Pipeline scheduled runs
-- Migration 039
-- Adds a table to track cron-scheduled pipeline triggers.

CREATE TABLE IF NOT EXISTS pipeline_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    -- Cron expression (5-field: min hour dom month dow)
    cron VARCHAR(64) NOT NULL,
    -- Human-readable schedule name
    name VARCHAR(255),
    -- Branch to run on (NULL = default branch)
    ref_name VARCHAR(255),
    -- YAML path
    yaml_path VARCHAR(512) NOT NULL DEFAULT '.civit/pipeline.yaml',
    -- Whether this schedule is enabled
    enabled BOOLEAN NOT NULL DEFAULT true,
    -- Last time this schedule was triggered
    last_run_at TIMESTAMPTZ,
    -- Next scheduled run time (computed)
    next_run_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pipeline_schedules_repo ON pipeline_schedules(repo_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_schedules_next_run ON pipeline_schedules(next_run_at) WHERE enabled = true;
