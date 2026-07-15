-- Log Aggregation v11: Enhanced log entries and alert rules v8

CREATE TABLE IF NOT EXISTS log_entries_v11 (
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

CREATE TABLE IF NOT EXISTS log_alert_rules_v8 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    level TEXT NOT NULL,
    pattern TEXT NOT NULL,
    threshold INTEGER NOT NULL DEFAULT 10,
    window_seconds INTEGER NOT NULL DEFAULT 300,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_log_entries_v11_level ON log_entries_v11(level);
CREATE INDEX IF NOT EXISTS idx_log_entries_v11_service ON log_entries_v11(service);
CREATE INDEX IF NOT EXISTS idx_log_entries_v11_trace_id ON log_entries_v11(trace_id);
CREATE INDEX IF NOT EXISTS idx_log_entries_v11_created_at ON log_entries_v11(created_at);
CREATE INDEX IF NOT EXISTS idx_log_entries_v11_indexed ON log_entries_v11(indexed);
CREATE INDEX IF NOT EXISTS idx_log_entries_v11_message_gin ON log_entries_v11 USING gin(to_tsvector('english', message));
