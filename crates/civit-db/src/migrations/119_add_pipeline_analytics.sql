-- CivitForge Phase 119: Pipeline Analytics
-- Migration 119
-- Adds pipeline run statistics and duration analysis.

CREATE TABLE IF NOT EXISTS pipeline_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    total_runs INTEGER NOT NULL DEFAULT 0,
    successful_runs INTEGER NOT NULL DEFAULT 0,
    failed_runs INTEGER NOT NULL DEFAULT 0,
    avg_duration_ms INTEGER NOT NULL DEFAULT 0,
    total_duration_ms BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pipeline_analytics_repo ON pipeline_analytics(repo_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_analytics_period ON pipeline_analytics(period_start, period_end);
