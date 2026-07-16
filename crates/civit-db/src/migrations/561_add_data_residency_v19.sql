CREATE TABLE IF NOT EXISTS data_residency_reports_v17 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_type TEXT NOT NULL,
    findings JSONB NOT NULL DEFAULT '[]',
    score INTEGER NOT NULL DEFAULT 0,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS data_residency_compliance_v17 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES data_residency_rules(id),
    compliance_status TEXT NOT NULL DEFAULT 'compliant',
    last_checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_data_residency_reports_v17_type ON data_residency_reports_v17(report_type);
CREATE INDEX IF NOT EXISTS idx_data_residency_compliance_v17_rule ON data_residency_compliance_v17(rule_id);
CREATE INDEX IF NOT EXISTS idx_data_residency_compliance_v17_status ON data_residency_compliance_v17(compliance_status);
