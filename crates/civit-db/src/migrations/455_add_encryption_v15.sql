CREATE TABLE IF NOT EXISTS encryption_key_versions_v12 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_id UUID NOT NULL REFERENCES encryption_keys(id),
    version INTEGER NOT NULL,
    key_material BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(key_id, version)
);

CREATE TABLE IF NOT EXISTS encryption_compliance_checks_v12 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    check_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    findings JSONB NOT NULL DEFAULT '[]',
    score INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_encryption_key_versions_v12_key ON encryption_key_versions_v12(key_id);
CREATE INDEX IF NOT EXISTS idx_encryption_key_versions_v12_version ON encryption_key_versions_v12(version);
CREATE INDEX IF NOT EXISTS idx_encryption_compliance_checks_v12_type ON encryption_compliance_checks_v12(check_type);
CREATE INDEX IF NOT EXISTS idx_encryption_compliance_checks_v12_status ON encryption_compliance_checks_v12(status);
