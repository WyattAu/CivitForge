CREATE TABLE IF NOT EXISTS trace_sampling_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    sample_rate DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_trace_sampling_rules_service ON trace_sampling_rules(service_name);
CREATE INDEX IF NOT EXISTS idx_trace_sampling_rules_service_endpoint ON trace_sampling_rules(service_name, endpoint);
