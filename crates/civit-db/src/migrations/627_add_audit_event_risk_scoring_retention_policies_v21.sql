-- Audit Trail v24: risk scoring v21, retention policies v21
CREATE TABLE IF NOT EXISTS audit_event_risk_scoring_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES audit_events(id) ON DELETE CASCADE,
    risk_score DOUBLE PRECISION NOT NULL DEFAULT 0,
    risk_factors JSONB NOT NULL DEFAULT '[]',
    mitigation_suggestions JSONB NOT NULL DEFAULT '[]',
    scored_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_risk_scoring_v21_event ON audit_event_risk_scoring_v21(event_id);
CREATE INDEX IF NOT EXISTS idx_risk_scoring_v21_score ON audit_event_risk_scoring_v21(risk_score DESC);
CREATE INDEX IF NOT EXISTS idx_risk_scoring_v21_scored ON audit_event_risk_scoring_v21(scored_at DESC);

CREATE TABLE IF NOT EXISTS audit_retention_policies_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_category TEXT NOT NULL,
    retention_days INTEGER NOT NULL DEFAULT 365,
    archive_after_days INTEGER,
    delete_after_days INTEGER,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(event_category)
);

CREATE INDEX IF NOT EXISTS idx_retention_policies_v21_category ON audit_retention_policies_v21(event_category);
CREATE INDEX IF NOT EXISTS idx_retention_policies_v21_enabled ON audit_retention_policies_v21(enabled);
