-- CivitForge Phase 193: Pipeline Action Categories
-- Migration 193
-- Adds category management for pipeline actions marketplace.

CREATE TABLE IF NOT EXISTS pipeline_action_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    parent_id UUID REFERENCES pipeline_action_categories(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS pipeline_action_category_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action_id UUID NOT NULL REFERENCES pipeline_actions(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES pipeline_action_categories(id) ON DELETE CASCADE,
    UNIQUE(action_id, category_id)
);

CREATE INDEX IF NOT EXISTS idx_pipeline_action_categories_name ON pipeline_action_categories(name);
CREATE INDEX IF NOT EXISTS idx_pipeline_action_categories_parent_id ON pipeline_action_categories(parent_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_action_category_members_action_id ON pipeline_action_category_members(action_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_action_category_members_category_id ON pipeline_action_category_members(category_id);
