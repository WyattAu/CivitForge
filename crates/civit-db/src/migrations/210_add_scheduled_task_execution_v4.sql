-- CivitForge Phase 210: Scheduled Tasks V4 - Execution Tracking
-- Migration 210
-- Adds execution tracking with input/output logging, error handling, and performance analytics.

CREATE TABLE IF NOT EXISTS scheduled_task_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES scheduled_tasks(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    input JSONB NOT NULL DEFAULT '{}',
    output JSONB NOT NULL DEFAULT '{}',
    error TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_scheduled_task_executions_task_id ON scheduled_task_executions(task_id);
CREATE INDEX idx_scheduled_task_executions_status ON scheduled_task_executions(status);
CREATE INDEX idx_scheduled_task_executions_started_at ON scheduled_task_executions(started_at);
CREATE INDEX idx_scheduled_task_executions_completed_at ON scheduled_task_executions(completed_at);
