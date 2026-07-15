-- CivitForge Phase 437: Compliance Frameworks V15
-- Migration 437
-- Adds versioned compliance frameworks, assessment history, finding tracking, and compliance scoring.

CREATE TABLE IF NOT EXISTS compliance_frameworks_v14 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    version TEXT NOT NULL DEFAULT '13.0',
    description TEXT NOT NULL DEFAULT '',
    requirements JSONB NOT NULL DEFAULT '[]',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_assessments_v13 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    framework_id UUID NOT NULL REFERENCES compliance_frameworks_v14(id),
    repo_id UUID REFERENCES repositories(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    findings JSONB NOT NULL DEFAULT '[]',
    score INTEGER NOT NULL DEFAULT 0,
    assessor_id UUID REFERENCES users(id),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_compliance_frameworks_v14_name ON compliance_frameworks_v14(name);
CREATE INDEX idx_compliance_frameworks_v14_enabled ON compliance_frameworks_v14(enabled);
CREATE INDEX idx_compliance_assessments_v13_framework_id ON compliance_assessments_v13(framework_id);
CREATE INDEX idx_compliance_assessments_v13_repo_id ON compliance_assessments_v13(repo_id);
CREATE INDEX idx_compliance_assessments_v13_status ON compliance_assessments_v13(status);
CREATE INDEX idx_compliance_assessments_v13_assessor_id ON compliance_assessments_v13(assessor_id);
