CREATE TABLE IF NOT EXISTS distributed_traces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    trace_id TEXT NOT NULL,
    span_id TEXT NOT NULL,
    parent_span_id TEXT,
    operation_name TEXT NOT NULL,
    service_name TEXT NOT NULL DEFAULT 'civitforge',
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ,
    duration_ms INTEGER,
    status TEXT NOT NULL DEFAULT 'ok',
    attributes JSONB NOT NULL DEFAULT '{}',
    events JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_distributed_traces_trace_id ON distributed_traces(trace_id);
CREATE INDEX IF NOT EXISTS idx_distributed_traces_operation_name ON distributed_traces(operation_name);
CREATE INDEX IF NOT EXISTS idx_distributed_traces_start_time ON distributed_traces(start_time);
CREATE INDEX IF NOT EXISTS idx_distributed_traces_service_name ON distributed_traces(service_name);
CREATE INDEX IF NOT EXISTS idx_distributed_traces_status ON distributed_traces(status);
