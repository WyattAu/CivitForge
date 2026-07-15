-- Migration 463: Add log_entries_v15 and log_alert_rules_v12

CREATE TABLE IF NOT EXISTS log_entries_v15 (
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

CREATE INDEX IF NOT EXISTS idx_log_entries_v15_level ON log_entries_v15(level);
CREATE INDEX IF NOT EXISTS idx_log_entries_v15_source ON log_entries_v15(source);
CREATE INDEX IF NOT EXISTS idx_log_entries_v15_service ON log_entries_v15(service);
CREATE INDEX IF NOT EXISTS idx_log_entries_v15_trace_id ON log_entries_v15(trace_id);
CREATE INDEX IF NOT EXISTS idx_log_entries_v15_created_at ON log_entries_v15(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_log_entries_v15_indexed ON log_entries_v15(indexed);

CREATE TABLE IF NOT EXISTS log_alert_rules_v12 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    level TEXT NOT NULL,
    pattern TEXT NOT NULL,
    threshold INTEGER NOT NULL DEFAULT 10,
    window_seconds INTEGER NOT NULL DEFAULT 300,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_log_alert_rules_v12_enabled ON log_alert_rules_v12(enabled);
CREATE INDEX IF NOT EXISTS idx_log_alert_rules_v12_level ON log_alert_rules_v12(level);
