-- Migration 317: Distributed Tracing v9
-- Adds trace_sampling_rules_v8 and trace_service_dependencies_v5 tables

CREATE TABLE IF NOT EXISTS trace_sampling_rules_v8 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    sample_rate DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    max_traces_per_second INTEGER NOT NULL DEFAULT 100,
    priority INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_trace_sampling_rules_v8_service_endpoint ON trace_sampling_rules_v8(service_name, endpoint);
CREATE INDEX IF NOT EXISTS idx_trace_sampling_rules_v8_enabled ON trace_sampling_rules_v8(enabled);
CREATE INDEX IF NOT EXISTS idx_trace_sampling_rules_v8_priority ON trace_sampling_rules_v8(priority DESC);

CREATE TABLE IF NOT EXISTS trace_service_dependencies_v5 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name TEXT NOT NULL,
    depends_on_service TEXT NOT NULL,
    call_count INTEGER NOT NULL DEFAULT 0,
    avg_duration_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    error_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    last_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_trace_service_dependencies_v5_service ON trace_service_dependencies_v5(service_name);
CREATE INDEX IF NOT EXISTS idx_trace_service_dependencies_v5_depends_on ON trace_service_dependencies_v5(depends_on_service);
