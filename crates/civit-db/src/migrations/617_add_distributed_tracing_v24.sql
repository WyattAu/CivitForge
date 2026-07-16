CREATE TABLE IF NOT EXISTS trace_service_health_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name TEXT NOT NULL,
    health_score DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    error_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    avg_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    throughput_rps DOUBLE PRECISION NOT NULL DEFAULT 0,
    checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS trace_cascade_failure_detection_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_service TEXT NOT NULL,
    affected_service TEXT NOT NULL,
    failure_type TEXT NOT NULL,
    cascade_depth INTEGER NOT NULL DEFAULT 1,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_trace_service_health_v20_service ON trace_service_health_v20(service_name);
CREATE INDEX IF NOT EXISTS idx_trace_service_health_v20_checked_at ON trace_service_health_v20(checked_at);
CREATE INDEX IF NOT EXISTS idx_trace_cascade_failure_v20_source ON trace_cascade_failure_detection_v20(source_service);
CREATE INDEX IF NOT EXISTS idx_trace_cascade_failure_v20_affected ON trace_cascade_failure_detection_v20(affected_service);
CREATE INDEX IF NOT EXISTS idx_trace_cascade_failure_v20_detected_at ON trace_cascade_failure_detection_v20(detected_at);
