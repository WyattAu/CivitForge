CREATE TABLE IF NOT EXISTS data_residency_audits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES data_residency_rules(id),
    audit_type TEXT NOT NULL,
    findings JSONB NOT NULL DEFAULT '[]',
    score INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS data_residency_migrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    violation_id UUID NOT NULL REFERENCES data_residency_violations(id),
    target_region TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_data_residency_audits_rule_id ON data_residency_audits(rule_id);
CREATE INDEX IF NOT EXISTS idx_data_residency_audits_audit_type ON data_residency_audits(audit_type);
CREATE INDEX IF NOT EXISTS idx_data_residency_audits_created_at ON data_residency_audits(created_at);
CREATE INDEX IF NOT EXISTS idx_data_residency_migrations_violation_id ON data_residency_migrations(violation_id);
CREATE INDEX IF NOT EXISTS idx_data_residency_migrations_status ON data_residency_migrations(status);
