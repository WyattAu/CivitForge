CREATE TABLE IF NOT EXISTS trace_sampling_rules_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    sample_rate DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    max_traces_per_second INTEGER NOT NULL DEFAULT 100,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS trace_dependencies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_trace_id TEXT NOT NULL,
    child_trace_id TEXT NOT NULL,
    dependency_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_trace_sampling_rules_v2_service ON trace_sampling_rules_v2(service_name);
CREATE INDEX IF NOT EXISTS idx_trace_sampling_rules_v2_service_endpoint ON trace_sampling_rules_v2(service_name, endpoint);
CREATE INDEX IF NOT EXISTS idx_trace_dependencies_parent ON trace_dependencies(parent_trace_id);
CREATE INDEX IF NOT EXISTS idx_trace_dependencies_child ON trace_dependencies(child_trace_id);
