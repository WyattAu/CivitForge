-- CivitForge Phase 120: Multi-project Pipelines
-- Migration 120
-- Adds cross-project pipeline orchestration with dependency management.

CREATE TABLE IF NOT EXISTS multi_project_pipelines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    project_ids UUID[] NOT NULL DEFAULT '{}',
    trigger_rules JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS multi_project_pipeline_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pipeline_id UUID NOT NULL REFERENCES multi_project_pipelines(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_multi_project_pipelines_enabled ON multi_project_pipelines(enabled);
CREATE INDEX IF NOT EXISTS idx_multi_project_pipeline_runs_pipeline ON multi_project_pipeline_runs(pipeline_id);
CREATE INDEX IF NOT EXISTS idx_multi_project_pipeline_runs_status ON multi_project_pipeline_runs(status);
