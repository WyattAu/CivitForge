-- Migration 421: Log Aggregation v13
-- Adds log_entries_v13 and log_alert_rules_v10 tables

CREATE TABLE IF NOT EXISTS log_entries_v13 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    level TEXT NOT NULL,
    message TEXT NOT NULL,
    source TEXT NOT NULL,
    service TEXT NOT NULL DEFAULT 'civitforge',
    trace_id TEXT,
    span_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    retention_days INTEGER NOT NULL DEFAULT 30,
    indexed BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS log_alert_rules_v10 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    level TEXT NOT NULL,
    pattern TEXT NOT NULL,
    threshold INTEGER NOT NULL DEFAULT 10,
    window_seconds INTEGER NOT NULL DEFAULT 300,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_log_entries_v13_level ON log_entries_v13(level);
CREATE INDEX IF NOT EXISTS idx_log_entries_v13_source ON log_entries_v13(source);
CREATE INDEX IF NOT EXISTS idx_log_entries_v13_service ON log_entries_v13(service);
CREATE INDEX IF NOT EXISTS idx_log_entries_v13_trace_id ON log_entries_v13(trace_id);
CREATE INDEX IF NOT EXISTS idx_log_entries_v13_created_at ON log_entries_v13(created_at);
CREATE INDEX IF NOT EXISTS idx_log_entries_v13_indexed ON log_entries_v13(indexed);
CREATE INDEX IF NOT EXISTS idx_log_alert_rules_v10_enabled ON log_alert_rules_v10(enabled);