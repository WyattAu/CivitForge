-- Migration 149: Advanced Pipeline Runners v2
-- Adds runner metrics collection, enhanced runner management, and job assignment tracking.

CREATE TABLE IF NOT EXISTS pipeline_runners_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    tags TEXT[] NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'online',
    last_heartbeat TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    current_job UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS runner_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    runner_id UUID NOT NULL REFERENCES pipeline_runners_v2(id) ON DELETE CASCADE,
    cpu_usage DOUBLE PRECISION NOT NULL DEFAULT 0,
    memory_usage DOUBLE PRECISION NOT NULL DEFAULT 0,
    disk_usage DOUBLE PRECISION NOT NULL DEFAULT 0,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pipeline_runners_v2_status ON pipeline_runners_v2(status);
CREATE INDEX IF NOT EXISTS idx_pipeline_runners_v2_tags ON pipeline_runners_v2 USING GIN(tags);
CREATE INDEX IF NOT EXISTS idx_runner_metrics_runner ON runner_metrics(runner_id);
CREATE INDEX IF NOT EXISTS idx_runner_metrics_recorded ON runner_metrics(recorded_at);
