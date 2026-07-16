CREATE TABLE IF NOT EXISTS pipeline_action_security_scans_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action_id UUID NOT NULL REFERENCES pipeline_actions(id) ON DELETE CASCADE,
    scan_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    findings JSONB NOT NULL DEFAULT '[]',
    score DOUBLE PRECISION,
    scanned_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS pipeline_action_compatibility_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action_id UUID NOT NULL REFERENCES pipeline_actions(id) ON DELETE CASCADE,
    platform TEXT NOT NULL,
    runtime_version TEXT NOT NULL,
    compatible BOOLEAN NOT NULL DEFAULT true,
    tested_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pipeline_action_security_scans_v20_action ON pipeline_action_security_scans_v20(action_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_action_security_scans_v20_status ON pipeline_action_security_scans_v20(status);
CREATE INDEX IF NOT EXISTS idx_pipeline_action_security_scans_v20_scan_type ON pipeline_action_security_scans_v20(scan_type);
CREATE INDEX IF NOT EXISTS idx_pipeline_action_compatibility_v20_action ON pipeline_action_compatibility_v20(action_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_action_compatibility_v20_platform ON pipeline_action_compatibility_v20(platform);
CREATE INDEX IF NOT EXISTS idx_pipeline_action_compatibility_v20_compatible ON pipeline_action_compatibility_v20(compatible);
