-- CivitForge Phase 206: Compliance Frameworks V3 & Evidence V2
-- Migration 206
-- Enhances compliance with framework versioning, evidence collection, and compliance scoring.

CREATE TABLE IF NOT EXISTS compliance_frameworks_v3 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    version TEXT NOT NULL DEFAULT '2.0',
    description TEXT NOT NULL DEFAULT '',
    requirements JSONB NOT NULL DEFAULT '[]',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_evidence_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assessment_id UUID NOT NULL REFERENCES compliance_assessments_v2(id),
    requirement_id TEXT NOT NULL,
    evidence_type TEXT NOT NULL,
    evidence_data JSONB NOT NULL DEFAULT '{}',
    verified BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_compliance_frameworks_v3_name ON compliance_frameworks_v3(name);
CREATE INDEX idx_compliance_frameworks_v3_version ON compliance_frameworks_v3(version);
CREATE INDEX idx_compliance_frameworks_v3_enabled ON compliance_frameworks_v3(enabled);
CREATE INDEX idx_compliance_evidence_v2_assessment_id ON compliance_evidence_v2(assessment_id);
CREATE INDEX idx_compliance_evidence_v2_requirement_id ON compliance_evidence_v2(requirement_id);
CREATE INDEX idx_compliance_evidence_v2_evidence_type ON compliance_evidence_v2(evidence_type);
CREATE INDEX idx_compliance_evidence_v2_verified ON compliance_evidence_v2(verified);
CREATE INDEX idx_compliance_evidence_v2_created_at ON compliance_evidence_v2(created_at);
