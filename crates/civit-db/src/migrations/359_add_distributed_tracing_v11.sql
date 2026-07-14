CREATE TABLE IF NOT EXISTS trace_sampling_rules_v10 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    sample_rate DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    max_traces_per_second INTEGER NOT NULL DEFAULT 100,
    priority INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_trace_sampling_rules_v10_service ON trace_sampling_rules_v10(service_name);
CREATE INDEX IF NOT EXISTS idx_trace_sampling_rules_v10_endpoint ON trace_sampling_rules_v10(endpoint);

CREATE TABLE IF NOT EXISTS trace_service_dependencies_v7 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name TEXT NOT NULL,
    depends_on_service TEXT NOT NULL,
    call_count INTEGER NOT NULL DEFAULT 0,
    avg_duration_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    error_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    last_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_trace_service_deps_v7_service ON trace_service_dependencies_v7(service_name);
CREATE INDEX IF NOT EXISTS idx_trace_service_deps_v7_depends ON trace_service_dependencies_v7(depends_on_service);
