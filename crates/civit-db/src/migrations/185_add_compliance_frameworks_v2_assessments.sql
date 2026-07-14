-- CivitForge Phase 185: Compliance Frameworks V2 & Assessments
-- Migration 185
-- Enhances compliance with framework versioning, assessment history, and finding tracking.

CREATE TABLE IF NOT EXISTS compliance_frameworks_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    version TEXT NOT NULL DEFAULT '1.0',
    description TEXT NOT NULL DEFAULT '',
    requirements JSONB NOT NULL DEFAULT '[]',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_assessments_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    framework_id UUID NOT NULL REFERENCES compliance_frameworks_v2(id),
    repo_id UUID REFERENCES repositories(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    findings JSONB NOT NULL DEFAULT '[]',
    score INTEGER NOT NULL DEFAULT 0,
    assessor_id UUID REFERENCES users(id),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_compliance_frameworks_v2_name ON compliance_frameworks_v2(name);
CREATE INDEX idx_compliance_frameworks_v2_version ON compliance_frameworks_v2(version);
CREATE INDEX idx_compliance_frameworks_v2_enabled ON compliance_frameworks_v2(enabled);
CREATE INDEX idx_compliance_assessments_v2_framework_id ON compliance_assessments_v2(framework_id);
CREATE INDEX idx_compliance_assessments_v2_repo_id ON compliance_assessments_v2(repo_id);
CREATE INDEX idx_compliance_assessments_v2_status ON compliance_assessments_v2(status);
CREATE INDEX idx_compliance_assessments_v2_assessor_id ON compliance_assessments_v2(assessor_id);
CREATE INDEX idx_compliance_assessments_v2_started_at ON compliance_assessments_v2(started_at);
