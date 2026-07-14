CREATE TABLE IF NOT EXISTS log_entries_v3 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    level TEXT NOT NULL,
    message TEXT NOT NULL,
    source TEXT NOT NULL,
    service TEXT NOT NULL DEFAULT 'civitforge',
    trace_id TEXT,
    span_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    retention_days INTEGER NOT NULL DEFAULT 30,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS log_search_index (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    log_id UUID NOT NULL REFERENCES log_entries_v3(id),
    search_vector TSVECTOR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_log_entries_v3_level ON log_entries_v3(level);
CREATE INDEX IF NOT EXISTS idx_log_entries_v3_source ON log_entries_v3(source);
CREATE INDEX IF NOT EXISTS idx_log_entries_v3_service ON log_entries_v3(service);
CREATE INDEX IF NOT EXISTS idx_log_entries_v3_trace_id ON log_entries_v3(trace_id);
CREATE INDEX IF NOT EXISTS idx_log_entries_v3_created_at ON log_entries_v3(created_at);
CREATE INDEX IF NOT EXISTS idx_log_search_index_log_id ON log_search_index(log_id);
CREATE INDEX IF NOT EXISTS idx_log_search_index_search_vector ON log_search_index USING GIN(search_vector);
