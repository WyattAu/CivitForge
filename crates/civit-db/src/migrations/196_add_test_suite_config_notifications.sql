-- Migration 196: Test Suite Configurations and Notifications
-- Adds configuration management and notification rules for test suites.

CREATE TABLE IF NOT EXISTS test_suite_configurations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    suite_id UUID NOT NULL REFERENCES test_suites(id) ON DELETE CASCADE,
    config_key TEXT NOT NULL,
    config_value JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(suite_id, config_key)
);

CREATE TABLE IF NOT EXISTS test_suite_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    suite_id UUID NOT NULL REFERENCES test_suites(id),
    notification_type TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_test_suite_configs_suite_id ON test_suite_configurations(suite_id);
CREATE INDEX IF NOT EXISTS idx_test_suite_configs_key ON test_suite_configurations(config_key);
CREATE INDEX IF NOT EXISTS idx_test_suite_notifications_suite_id ON test_suite_notifications(suite_id);
CREATE INDEX IF NOT EXISTS idx_test_suite_notifications_type ON test_suite_notifications(notification_type);
CREATE INDEX IF NOT EXISTS idx_test_suite_notifications_enabled ON test_suite_notifications(enabled);
