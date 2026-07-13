-- Admin dashboard widget configuration
-- Migration 092

CREATE TABLE IF NOT EXISTS admin_dashboard_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    widget_name TEXT NOT NULL UNIQUE,
    widget_config JSONB NOT NULL DEFAULT '{}',
    position INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_admin_dashboard_config_position ON admin_dashboard_config(position);
