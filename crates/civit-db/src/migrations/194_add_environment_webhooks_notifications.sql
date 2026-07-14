-- CivitForge Phase 194: Environment Webhooks & Notifications
-- Migration 194
-- Adds webhook and notification configuration for pipeline environments.

CREATE TABLE IF NOT EXISTS environment_webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id UUID NOT NULL REFERENCES pipeline_environments(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    events TEXT[] NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS environment_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id UUID NOT NULL REFERENCES pipeline_environments(id) ON DELETE CASCADE,
    notification_type TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS environment_webhook_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    webhook_id UUID NOT NULL REFERENCES environment_webhooks(id) ON DELETE CASCADE,
    event TEXT NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'pending',
    response_status INTEGER,
    response_body TEXT,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    next_retry_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_environment_webhooks_environment_id ON environment_webhooks(environment_id);
CREATE INDEX IF NOT EXISTS idx_environment_webhooks_events ON environment_webhooks USING GIN(events);
CREATE INDEX IF NOT EXISTS idx_environment_notifications_environment_id ON environment_notifications(environment_id);
CREATE INDEX IF NOT EXISTS idx_environment_notifications_type ON environment_notifications(notification_type);
CREATE INDEX IF NOT EXISTS idx_environment_webhook_deliveries_webhook_id ON environment_webhook_deliveries(webhook_id);
CREATE INDEX IF NOT EXISTS idx_environment_webhook_deliveries_status ON environment_webhook_deliveries(status);
