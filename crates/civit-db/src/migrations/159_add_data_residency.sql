CREATE TABLE IF NOT EXISTS data_residency_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    data_types TEXT[] NOT NULL DEFAULT '{}',
    allowed_regions TEXT[] NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS data_residency_violations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES data_residency_rules(id),
    data_type TEXT NOT NULL,
    data_id UUID NOT NULL,
    region TEXT NOT NULL,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_data_residency_rules_name ON data_residency_rules(name);
CREATE INDEX IF NOT EXISTS idx_data_residency_rules_enabled ON data_residency_rules(enabled);
CREATE INDEX IF NOT EXISTS idx_data_residency_violations_rule_id ON data_residency_violations(rule_id);
CREATE INDEX IF NOT EXISTS idx_data_residency_violations_data_type ON data_residency_violations(data_type);
CREATE INDEX IF NOT EXISTS idx_data_residency_violations_detected_at ON data_residency_violations(detected_at);
