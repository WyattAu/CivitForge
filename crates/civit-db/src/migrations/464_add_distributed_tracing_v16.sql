-- Migration 464: Add trace_sampling_rules_v15 and trace_service_dependencies_v12

CREATE TABLE IF NOT EXISTS trace_sampling_rules_v15 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    sample_rate DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    max_traces_per_second INTEGER NOT NULL DEFAULT 100,
    priority INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_trace_sampling_rules_v15_service ON trace_sampling_rules_v15(service_name);
CREATE INDEX IF NOT EXISTS idx_trace_sampling_rules_v15_endpoint ON trace_sampling_rules_v15(endpoint);
CREATE INDEX IF NOT EXISTS idx_trace_sampling_rules_v15_enabled ON trace_sampling_rules_v15(enabled);
CREATE INDEX IF NOT EXISTS idx_trace_sampling_rules_v15_priority ON trace_sampling_rules_v15(priority DESC);

CREATE TABLE IF NOT EXISTS trace_service_dependencies_v12 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name TEXT NOT NULL,
    depends_on_service TEXT NOT NULL,
    call_count INTEGER NOT NULL DEFAULT 0,
    avg_duration_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    error_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    last_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(service_name, depends_on_service)
);

CREATE INDEX IF NOT EXISTS idx_trace_service_dependencies_v12_service ON trace_service_dependencies_v12(service_name);
CREATE INDEX IF NOT EXISTS idx_trace_service_dependencies_v12_depends ON trace_service_dependencies_v12(depends_on_service);
CREATE INDEX IF NOT EXISTS idx_trace_service_dependencies_v12_calls ON trace_service_dependencies_v12(call_count DESC);
