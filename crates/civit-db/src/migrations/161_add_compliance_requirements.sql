-- CivitForge Phase 161: Compliance Requirements
-- Migration 161
-- Adds granular compliance requirements with automated checks, evidence collection, and scoring.

CREATE TABLE IF NOT EXISTS compliance_requirements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    framework_id UUID NOT NULL REFERENCES compliance_frameworks(id),
    requirement_id TEXT NOT NULL,
    description TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'medium',
    automated_check BOOLEAN NOT NULL DEFAULT false,
    check_config JSONB NOT NULL DEFAULT '{}',
    evidence_config JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_evidence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    requirement_id UUID NOT NULL REFERENCES compliance_requirements(id),
    assessment_id UUID NOT NULL REFERENCES compliance_assessments(id),
    evidence_type TEXT NOT NULL DEFAULT 'manual',
    content JSONB NOT NULL DEFAULT '{}',
    collected_by UUID REFERENCES users(id),
    collected_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_check_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    requirement_id UUID NOT NULL REFERENCES compliance_requirements(id),
    assessment_id UUID NOT NULL REFERENCES compliance_assessments(id),
    status TEXT NOT NULL DEFAULT 'pending',
    result_details JSONB NOT NULL DEFAULT '{}',
    score INTEGER NOT NULL DEFAULT 0,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_compliance_requirements_framework_id ON compliance_requirements(framework_id);
CREATE INDEX idx_compliance_requirements_requirement_id ON compliance_requirements(requirement_id);
CREATE INDEX idx_compliance_requirements_severity ON compliance_requirements(severity);
CREATE INDEX idx_compliance_requirements_automated_check ON compliance_requirements(automated_check);
CREATE INDEX idx_compliance_evidence_requirement_id ON compliance_evidence(requirement_id);
CREATE INDEX idx_compliance_evidence_assessment_id ON compliance_evidence(assessment_id);
CREATE INDEX idx_compliance_check_results_requirement_id ON compliance_check_results(requirement_id);
CREATE INDEX idx_compliance_check_results_assessment_id ON compliance_check_results(assessment_id);
CREATE INDEX idx_compliance_check_results_status ON compliance_check_results(status);
