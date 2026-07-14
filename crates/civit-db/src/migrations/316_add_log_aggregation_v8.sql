-- Migration 316: Log Aggregation v8
-- Adds log_entries_v8 and log_alert_rules_v5 tables

CREATE TABLE IF NOT EXISTS log_entries_v8 (
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

CREATE INDEX IF NOT EXISTS idx_log_entries_v8_level ON log_entries_v8(level);
CREATE INDEX IF NOT EXISTS idx_log_entries_v8_source ON log_entries_v8(source);
CREATE INDEX IF NOT EXISTS idx_log_entries_v8_service ON log_entries_v8(service);
CREATE INDEX IF NOT EXISTS idx_log_entries_v8_trace_id ON log_entries_v8(trace_id);
CREATE INDEX IF NOT EXISTS idx_log_entries_v8_created_at ON log_entries_v8(created_at);
CREATE INDEX IF NOT EXISTS idx_log_entries_v8_indexed ON log_entries_v8(indexed);

CREATE TABLE IF NOT EXISTS log_alert_rules_v5 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    level TEXT NOT NULL,
    pattern TEXT NOT NULL,
    threshold INTEGER NOT NULL DEFAULT 10,
    window_seconds INTEGER NOT NULL DEFAULT 300,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_log_alert_rules_v5_enabled ON log_alert_rules_v5(enabled);
