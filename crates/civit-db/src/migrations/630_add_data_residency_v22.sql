CREATE TABLE IF NOT EXISTS data_residency_transfer_requests_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    data_category TEXT NOT NULL,
    source_region TEXT NOT NULL,
    target_region TEXT NOT NULL,
    data_identifiers JSONB NOT NULL DEFAULT '[]',
    status TEXT NOT NULL DEFAULT 'pending',
    requested_by UUID NOT NULL REFERENCES users(id),
    approved_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS data_residency_compliance_checks_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    data_category TEXT NOT NULL,
    region TEXT NOT NULL,
    check_type TEXT NOT NULL,
    result TEXT NOT NULL DEFAULT 'pending',
    details JSONB NOT NULL DEFAULT '{}',
    checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_data_residency_transfer_requests_v21_status ON data_residency_transfer_requests_v21(status);
CREATE INDEX IF NOT EXISTS idx_data_residency_transfer_requests_v21_category ON data_residency_transfer_requests_v21(data_category);
CREATE INDEX IF NOT EXISTS idx_data_residency_transfer_requests_v21_source ON data_residency_transfer_requests_v21(source_region);
CREATE INDEX IF NOT EXISTS idx_data_residency_transfer_requests_v21_target ON data_residency_transfer_requests_v21(target_region);
CREATE INDEX IF NOT EXISTS idx_data_residency_compliance_checks_v21_category ON data_residency_compliance_checks_v21(data_category);
CREATE INDEX IF NOT EXISTS idx_data_residency_compliance_checks_v21_region ON data_residency_compliance_checks_v21(region);
CREATE INDEX IF NOT EXISTS idx_data_residency_compliance_checks_v21_result ON data_residency_compliance_checks_v21(result);
