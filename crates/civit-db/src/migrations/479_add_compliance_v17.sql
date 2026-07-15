-- CivitForge Phase 479: Compliance Frameworks V17
-- Migration 479
-- Enhances compliance frameworks with framework versioning v17, assessment history v15, finding tracking v17, and compliance scoring v17.

CREATE TABLE IF NOT EXISTS compliance_frameworks_v16 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    version TEXT NOT NULL DEFAULT '15.0',
    description TEXT NOT NULL DEFAULT '',
    requirements JSONB NOT NULL DEFAULT '[]',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_assessments_v15 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    framework_id UUID NOT NULL REFERENCES compliance_frameworks_v16(id),
    repo_id UUID REFERENCES repositories(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    findings JSONB NOT NULL DEFAULT '[]',
    score INTEGER NOT NULL DEFAULT 0,
    assessor_id UUID REFERENCES users(id),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_compliance_frameworks_v16_name ON compliance_frameworks_v16(name);
CREATE INDEX idx_compliance_frameworks_v16_enabled ON compliance_frameworks_v16(enabled);
CREATE INDEX idx_compliance_assessments_v15_framework_id ON compliance_assessments_v15(framework_id);
CREATE INDEX idx_compliance_assessments_v15_repo_id ON compliance_assessments_v15(repo_id);
CREATE INDEX idx_compliance_assessments_v15_status ON compliance_assessments_v15(status);
CREATE INDEX idx_compliance_assessments_v15_assessor_id ON compliance_assessments_v15(assessor_id);
