-- Migration 422: Distributed Tracing v14
-- Adds trace_sampling_rules_v13 and trace_service_dependencies_v10 tables

CREATE TABLE IF NOT EXISTS trace_sampling_rules_v13 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    sample_rate DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    max_traces_per_second INTEGER NOT NULL DEFAULT 100,
    priority INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS trace_service_dependencies_v10 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name TEXT NOT NULL,
    depends_on_service TEXT NOT NULL,
    call_count INTEGER NOT NULL DEFAULT 0,
    avg_duration_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    error_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    last_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_trace_sampling_rules_v13_service ON trace_sampling_rules_v13(service_name);
CREATE INDEX IF NOT EXISTS idx_trace_sampling_rules_v13_endpoint ON trace_sampling_rules_v13(endpoint);
CREATE INDEX IF NOT EXISTS idx_trace_sampling_rules_v13_enabled ON trace_sampling_rules_v13(enabled);
CREATE INDEX IF NOT EXISTS idx_trace_service_dependencies_v10_service ON trace_service_dependencies_v10(service_name);
CREATE INDEX IF NOT EXISTS idx_trace_service_dependencies_v10_depends ON trace_service_dependencies_v10(depends_on_service);