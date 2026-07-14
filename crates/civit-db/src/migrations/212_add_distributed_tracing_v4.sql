CREATE TABLE IF NOT EXISTS trace_sampling_rules_v3 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    sample_rate DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    max_traces_per_second INTEGER NOT NULL DEFAULT 100,
    priority INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS trace_service_map (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    call_count INTEGER NOT NULL DEFAULT 0,
    avg_duration_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    error_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    last_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_trace_sampling_rules_v3_service ON trace_sampling_rules_v3(service_name);
CREATE INDEX IF NOT EXISTS idx_trace_sampling_rules_v3_endpoint ON trace_sampling_rules_v3(endpoint);
CREATE INDEX IF NOT EXISTS idx_trace_sampling_rules_v3_priority ON trace_sampling_rules_v3(priority);
CREATE INDEX IF NOT EXISTS idx_trace_service_map_service ON trace_service_map(service_name);
CREATE INDEX IF NOT EXISTS idx_trace_service_map_endpoint ON trace_service_map(endpoint);
