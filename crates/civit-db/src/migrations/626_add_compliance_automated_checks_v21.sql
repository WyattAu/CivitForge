-- Compliance v24: automated checks v21, check results v21
CREATE TABLE IF NOT EXISTS compliance_automated_checks_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    requirement_id UUID NOT NULL REFERENCES compliance_requirements(id),
    check_type TEXT NOT NULL,
    check_config JSONB NOT NULL DEFAULT '{}',
    last_run_at TIMESTAMPTZ,
    last_result TEXT NOT NULL DEFAULT 'pending',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_compliance_checks_v21_requirement ON compliance_automated_checks_v21(requirement_id);
CREATE INDEX IF NOT EXISTS idx_compliance_checks_v21_type ON compliance_automated_checks_v21(check_type);
CREATE INDEX IF NOT EXISTS idx_compliance_checks_v21_enabled ON compliance_automated_checks_v21(enabled);
CREATE INDEX IF NOT EXISTS idx_compliance_checks_v21_last_result ON compliance_automated_checks_v21(last_result);

CREATE TABLE IF NOT EXISTS compliance_check_results_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    check_id UUID NOT NULL REFERENCES compliance_automated_checks_v21(id),
    result TEXT NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    run_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_compliance_results_v21_check ON compliance_check_results_v21(check_id);
CREATE INDEX IF NOT EXISTS idx_compliance_results_v21_result ON compliance_check_results_v21(result);
CREATE INDEX IF NOT EXISTS idx_compliance_results_v21_run ON compliance_check_results_v21(run_at DESC);
CREATE INDEX IF NOT EXISTS idx_compliance_results_v21_check_run ON compliance_check_results_v21(check_id, run_at DESC);
