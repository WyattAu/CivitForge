-- CivitForge Phase 133: Pipeline Actions Marketplace
-- Migration 133
-- Adds a marketplace for reusable pipeline actions.

CREATE TABLE IF NOT EXISTS pipeline_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    action_type TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}',
    version TEXT NOT NULL DEFAULT '1.0.0',
    author_id UUID REFERENCES users(id),
    downloads INTEGER NOT NULL DEFAULT 0,
    rating DOUBLE PRECISION NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pipeline_actions_name ON pipeline_actions(name);
CREATE INDEX IF NOT EXISTS idx_pipeline_actions_action_type ON pipeline_actions(action_type);
CREATE INDEX IF NOT EXISTS idx_pipeline_actions_author_id ON pipeline_actions(author_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_actions_downloads ON pipeline_actions(downloads DESC);
CREATE INDEX IF NOT EXISTS idx_pipeline_actions_rating ON pipeline_actions(rating DESC);
