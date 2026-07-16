CREATE TABLE IF NOT EXISTS compliance_rule_sets_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    standard TEXT NOT NULL,
    rules JSONB NOT NULL DEFAULT '[]',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_evidence_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    requirement_id UUID NOT NULL REFERENCES compliance_requirements(id),
    evidence_type TEXT NOT NULL,
    evidence_data JSONB NOT NULL DEFAULT '{}',
    verified BOOLEAN NOT NULL DEFAULT false,
    verified_by UUID REFERENCES users(id),
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
