-- CivitForge Phase 122: Compliance Framework
-- Migration 122
-- Adds compliance framework management, assessment execution, finding tracking and reporting.

CREATE TABLE IF NOT EXISTS compliance_frameworks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    requirements JSONB NOT NULL DEFAULT '[]',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_assessments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    framework_id UUID NOT NULL REFERENCES compliance_frameworks(id),
    repo_id UUID REFERENCES repositories(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    findings JSONB NOT NULL DEFAULT '[]',
    score INTEGER NOT NULL DEFAULT 0,
    assessor_id UUID REFERENCES users(id),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_compliance_frameworks_name ON compliance_frameworks(name);
CREATE INDEX idx_compliance_frameworks_enabled ON compliance_frameworks(enabled);
CREATE INDEX idx_compliance_assessments_framework_id ON compliance_assessments(framework_id);
CREATE INDEX idx_compliance_assessments_repo_id ON compliance_assessments(repo_id);
CREATE INDEX idx_compliance_assessments_status ON compliance_assessments(status);
