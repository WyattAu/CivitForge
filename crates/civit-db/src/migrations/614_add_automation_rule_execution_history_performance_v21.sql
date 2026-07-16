CREATE TABLE IF NOT EXISTS automation_rule_execution_history_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES automation_rules(id) ON DELETE CASCADE,
    trigger_event TEXT NOT NULL,
    matched BOOLEAN NOT NULL DEFAULT false,
    action_taken TEXT,
    duration_ms INTEGER,
    error_message TEXT,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS automation_rule_performance_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES automation_rules(id) ON DELETE CASCADE,
    metric_name TEXT NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    measured_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_automation_rule_execution_history_v21_rule_id ON automation_rule_execution_history_v21(rule_id);
CREATE INDEX IF NOT EXISTS idx_automation_rule_execution_history_v21_executed_at ON automation_rule_execution_history_v21(executed_at);
CREATE INDEX IF NOT EXISTS idx_automation_rule_performance_v21_rule_id ON automation_rule_performance_v21(rule_id);
CREATE INDEX IF NOT EXISTS idx_automation_rule_performance_v21_metric_name ON automation_rule_performance_v21(metric_name);
