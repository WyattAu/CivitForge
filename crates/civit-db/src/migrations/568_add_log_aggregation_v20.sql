-- Log Aggregation v20: Enhanced log entries with advanced alert rules v17

CREATE TABLE IF NOT EXISTS log_entries_v20 (
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

CREATE TABLE IF NOT EXISTS log_alert_rules_v17 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    level TEXT NOT NULL,
    pattern TEXT NOT NULL,
    threshold INTEGER NOT NULL DEFAULT 10,
    window_seconds INTEGER NOT NULL DEFAULT 300,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_log_entries_v20_level ON log_entries_v20(level);
CREATE INDEX IF NOT EXISTS idx_log_entries_v20_service ON log_entries_v20(service);
CREATE INDEX IF NOT EXISTS idx_log_entries_v20_trace_id ON log_entries_v20(trace_id);
CREATE INDEX IF NOT EXISTS idx_log_entries_v20_created_at ON log_entries_v20(created_at);
CREATE INDEX IF NOT EXISTS idx_log_entries_v20_indexed ON log_entries_v20(indexed);
CREATE INDEX IF NOT EXISTS idx_log_alert_rules_v17_level ON log_alert_rules_v17(level);
CREATE INDEX IF NOT EXISTS idx_log_alert_rules_v17_enabled ON log_alert_rules_v17(enabled);
