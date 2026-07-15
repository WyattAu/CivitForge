CREATE TABLE IF NOT EXISTS compliance_frameworks_v17 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    version TEXT NOT NULL DEFAULT '16.0',
    description TEXT NOT NULL DEFAULT '',
    requirements JSONB NOT NULL DEFAULT '[]',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_assessments_v16 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    framework_id UUID NOT NULL REFERENCES compliance_frameworks_v17(id),
    repo_id UUID REFERENCES repositories(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    findings JSONB NOT NULL DEFAULT '[]',
    score INTEGER NOT NULL DEFAULT 0,
    assessor_id UUID REFERENCES users(id),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_compliance_frameworks_v17_name ON compliance_frameworks_v17(name);
CREATE INDEX IF NOT EXISTS idx_compliance_frameworks_v17_enabled ON compliance_frameworks_v17(enabled);
CREATE INDEX IF NOT EXISTS idx_compliance_assessments_v16_framework_id ON compliance_assessments_v16(framework_id);
CREATE INDEX IF NOT EXISTS idx_compliance_assessments_v16_repo_id ON compliance_assessments_v16(repo_id);
CREATE INDEX IF NOT EXISTS idx_compliance_assessments_v16_status ON compliance_assessments_v16(status);
