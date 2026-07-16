CREATE TABLE IF NOT EXISTS environment_drift_detection_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id UUID NOT NULL REFERENCES pipeline_environments(id) ON DELETE CASCADE,
    drift_type TEXT NOT NULL,
    expected_state JSONB NOT NULL,
    actual_state JSONB NOT NULL,
    severity TEXT NOT NULL DEFAULT 'warning',
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS environment_snapshot_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id UUID NOT NULL REFERENCES pipeline_environments(id) ON DELETE CASCADE,
    snapshot_data JSONB NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    description TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_environment_drift_detection_v20_env ON environment_drift_detection_v20(environment_id);
CREATE INDEX IF NOT EXISTS idx_environment_drift_detection_v20_severity ON environment_drift_detection_v20(severity);
CREATE INDEX IF NOT EXISTS idx_environment_drift_detection_v20_drift_type ON environment_drift_detection_v20(drift_type);
CREATE INDEX IF NOT EXISTS idx_environment_snapshot_v20_env ON environment_snapshot_v20(environment_id);
CREATE INDEX IF NOT EXISTS idx_environment_snapshot_v20_created_by ON environment_snapshot_v20(created_by);
