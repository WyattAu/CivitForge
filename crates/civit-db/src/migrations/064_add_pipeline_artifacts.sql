-- CivitForge Phase 64: Pipeline Artifacts
-- Migration 064
-- Adds artifact storage for CI/CD pipelines.

CREATE TABLE IF NOT EXISTS pipeline_artifacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pipeline_run_id UUID NOT NULL REFERENCES pipeline_runs(id) ON DELETE CASCADE,
    job_id UUID NOT NULL REFERENCES pipeline_run_jobs(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    size_bytes BIGINT NOT NULL DEFAULT 0,
    content_type TEXT NOT NULL DEFAULT 'application/octet-stream',
    storage_key TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pipeline_artifacts_run_id ON pipeline_artifacts(pipeline_run_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_artifacts_job_id ON pipeline_artifacts(job_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_artifacts_storage_key ON pipeline_artifacts(storage_key);
