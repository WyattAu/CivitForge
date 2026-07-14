CREATE TABLE IF NOT EXISTS log_entries_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    level TEXT NOT NULL,
    message TEXT NOT NULL,
    source TEXT NOT NULL,
    service TEXT NOT NULL DEFAULT 'civitforge',
    trace_id TEXT,
    span_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS log_retention_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service TEXT NOT NULL,
    level TEXT NOT NULL,
    retention_days INTEGER NOT NULL DEFAULT 30,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_log_entries_v2_level ON log_entries_v2(level);
CREATE INDEX IF NOT EXISTS idx_log_entries_v2_source ON log_entries_v2(source);
CREATE INDEX IF NOT EXISTS idx_log_entries_v2_service ON log_entries_v2(service);
CREATE INDEX IF NOT EXISTS idx_log_entries_v2_trace_id ON log_entries_v2(trace_id);
CREATE INDEX IF NOT EXISTS idx_log_entries_v2_created_at ON log_entries_v2(created_at);
CREATE INDEX IF NOT EXISTS idx_log_entries_v2_level_created ON log_entries_v2(level, created_at);
CREATE INDEX IF NOT EXISTS idx_log_entries_v2_service_level ON log_entries_v2(service, level);
CREATE INDEX IF NOT EXISTS idx_log_retention_policies_service ON log_retention_policies(service);
