CREATE TABLE IF NOT EXISTS analytics_dashboard_templates_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT 'general',
    template_data JSONB NOT NULL DEFAULT '{}',
    usage_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS analytics_alert_rules_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_name TEXT NOT NULL,
    condition TEXT NOT NULL,
    threshold DOUBLE PRECISION NOT NULL,
    severity TEXT NOT NULL DEFAULT 'warning',
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_analytics_dashboard_templates_v21_name ON analytics_dashboard_templates_v21(name);
CREATE INDEX IF NOT EXISTS idx_analytics_dashboard_templates_v21_category ON analytics_dashboard_templates_v21(category);
CREATE INDEX IF NOT EXISTS idx_analytics_alert_rules_v21_metric ON analytics_alert_rules_v21(metric_name);
CREATE INDEX IF NOT EXISTS idx_analytics_alert_rules_v21_severity ON analytics_alert_rules_v21(severity);
