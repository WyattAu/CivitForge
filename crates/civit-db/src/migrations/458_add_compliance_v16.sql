-- CivitForge Phase 458: Compliance Frameworks V16
-- Migration 458
-- Enhances compliance frameworks with framework versioning v16, assessment history v14, finding tracking v16, and compliance scoring v16.

CREATE TABLE IF NOT EXISTS compliance_frameworks_v15 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    version TEXT NOT NULL DEFAULT '14.0',
    description TEXT NOT NULL DEFAULT '',
    requirements JSONB NOT NULL DEFAULT '[]',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_assessments_v14 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    framework_id UUID NOT NULL REFERENCES compliance_frameworks_v15(id),
    repo_id UUID REFERENCES repositories(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    findings JSONB NOT NULL DEFAULT '[]',
    score INTEGER NOT NULL DEFAULT 0,
    assessor_id UUID REFERENCES users(id),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_compliance_frameworks_v15_name ON compliance_frameworks_v15(name);
CREATE INDEX idx_compliance_frameworks_v15_enabled ON compliance_frameworks_v15(enabled);
CREATE INDEX idx_compliance_assessments_v14_framework_id ON compliance_assessments_v14(framework_id);
CREATE INDEX idx_compliance_assessments_v14_repo_id ON compliance_assessments_v14(repo_id);
CREATE INDEX idx_compliance_assessments_v14_status ON compliance_assessments_v14(status);
CREATE INDEX idx_compliance_assessments_v14_assessor_id ON compliance_assessments_v14(assessor_id);
