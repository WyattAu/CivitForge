CREATE TABLE IF NOT EXISTS firewall_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    action TEXT NOT NULL DEFAULT 'allow',
    protocol TEXT NOT NULL DEFAULT 'tcp',
    source_ip INET,
    source_port INTEGER,
    destination_ip INET,
    destination_port INTEGER,
    enabled BOOLEAN NOT NULL DEFAULT true,
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS firewall_rule_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES firewall_rules(id) ON DELETE CASCADE,
    source_ip INET NOT NULL,
    destination_ip INET,
    destination_port INTEGER,
    action_taken TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_firewall_rules_enabled ON firewall_rules(enabled);
CREATE INDEX IF NOT EXISTS idx_firewall_rules_priority ON firewall_rules(priority);
CREATE INDEX IF NOT EXISTS idx_firewall_rule_logs_rule_id ON firewall_rule_logs(rule_id);
CREATE INDEX IF NOT EXISTS idx_firewall_rule_logs_created_at ON firewall_rule_logs(created_at);
