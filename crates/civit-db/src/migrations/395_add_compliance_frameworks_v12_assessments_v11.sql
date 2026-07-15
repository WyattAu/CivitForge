-- Migration 395: Compliance Frameworks v12 and Assessments v11

CREATE TABLE IF NOT EXISTS compliance_frameworks_v12 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    version TEXT NOT NULL DEFAULT '11.0',
    description TEXT NOT NULL DEFAULT '',
    requirements JSONB NOT NULL DEFAULT '[]',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_assessments_v11 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    framework_id UUID NOT NULL REFERENCES compliance_frameworks_v12(id),
    repo_id UUID REFERENCES repositories(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    findings JSONB NOT NULL DEFAULT '[]',
    score INTEGER NOT NULL DEFAULT 0,
    assessor_id UUID REFERENCES users(id),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_compliance_frameworks_v12_name ON compliance_frameworks_v12(name);
CREATE INDEX IF NOT EXISTS idx_compliance_frameworks_v12_enabled ON compliance_frameworks_v12(enabled);
CREATE INDEX IF NOT EXISTS idx_compliance_assessments_v11_framework ON compliance_assessments_v11(framework_id);
CREATE INDEX IF NOT EXISTS idx_compliance_assessments_v11_repo ON compliance_assessments_v11(repo_id);
CREATE INDEX IF NOT EXISTS idx_compliance_assessments_v11_status ON compliance_assessments_v11(status);
CREATE INDEX IF NOT EXISTS idx_compliance_assessments_v11_assessor ON compliance_assessments_v11(assessor_id);
