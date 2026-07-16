CREATE TABLE IF NOT EXISTS scheduled_task_performance_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES scheduled_tasks(id) ON DELETE CASCADE,
    metric_name TEXT NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    measured_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS scheduled_task_resource_usage_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES scheduled_tasks(id) ON DELETE CASCADE,
    cpu_usage_percent DOUBLE PRECISION NOT NULL DEFAULT 0,
    memory_usage_bytes BIGINT NOT NULL DEFAULT 0,
    disk_usage_bytes BIGINT NOT NULL DEFAULT 0,
    network_bytes_sent BIGINT NOT NULL DEFAULT 0,
    network_bytes_received BIGINT NOT NULL DEFAULT 0,
    measured_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_scheduled_task_performance_v21_task_id ON scheduled_task_performance_v21(task_id);
CREATE INDEX IF NOT EXISTS idx_scheduled_task_performance_v21_metric_name ON scheduled_task_performance_v21(metric_name);
CREATE INDEX IF NOT EXISTS idx_scheduled_task_resource_usage_v21_task_id ON scheduled_task_resource_usage_v21(task_id);
CREATE INDEX IF NOT EXISTS idx_scheduled_task_resource_usage_v21_measured_at ON scheduled_task_resource_usage_v21(measured_at);
