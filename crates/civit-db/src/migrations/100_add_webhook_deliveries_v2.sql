-- CivitForge: Enhanced Webhook Deliveries
-- Migration 100

CREATE TABLE IF NOT EXISTS webhook_deliveries_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    webhook_id UUID NOT NULL REFERENCES webhooks(id) ON DELETE CASCADE,
    event TEXT NOT NULL,
    payload JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    response_status INTEGER,
    response_body TEXT,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    next_retry_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_webhook_deliveries_v2_webhook_id ON webhook_deliveries_v2(webhook_id);
CREATE INDEX idx_webhook_deliveries_v2_status_retry ON webhook_deliveries_v2(status, next_retry_at) WHERE status IN ('pending', 'retrying');
CREATE INDEX idx_webhook_deliveries_v2_event ON webhook_deliveries_v2(event);