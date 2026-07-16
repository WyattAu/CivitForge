CREATE TABLE IF NOT EXISTS data_residency_audit_logs_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    data_category TEXT NOT NULL,
    source_region TEXT NOT NULL,
    target_region TEXT NOT NULL,
    action TEXT NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS data_residency_policies_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    data_category TEXT NOT NULL,
    allowed_regions TEXT[] NOT NULL DEFAULT '{}',
    encryption_required BOOLEAN NOT NULL DEFAULT true,
    retention_days INTEGER,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(data_category)
);

CREATE INDEX IF NOT EXISTS idx_data_residency_audit_logs_v20_category ON data_residency_audit_logs_v20(data_category);
CREATE INDEX IF NOT EXISTS idx_data_residency_audit_logs_v20_source ON data_residency_audit_logs_v20(source_region);
CREATE INDEX IF NOT EXISTS idx_data_residency_audit_logs_v20_target ON data_residency_audit_logs_v20(target_region);
CREATE INDEX IF NOT EXISTS idx_data_residency_audit_logs_v20_user ON data_residency_audit_logs_v20(user_id);
CREATE INDEX IF NOT EXISTS idx_data_residency_audit_logs_v20_created ON data_residency_audit_logs_v20(created_at);
CREATE INDEX IF NOT EXISTS idx_data_residency_policies_v20_category ON data_residency_policies_v20(data_category);
