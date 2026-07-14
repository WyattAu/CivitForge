-- CivitForge Phase 187: Workflow Templates
-- Migration 187
-- Adds workflow templates and usage tracking for template marketplace.

CREATE TABLE IF NOT EXISTS workflow_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    template_type TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}',
    is_public BOOLEAN NOT NULL DEFAULT false,
    author_id UUID REFERENCES users(id),
    usage_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS workflow_template_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES workflow_templates(id),
    user_id UUID NOT NULL REFERENCES users(id),
    used_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_workflow_templates_template_type ON workflow_templates(template_type);
CREATE INDEX idx_workflow_templates_is_public ON workflow_templates(is_public);
CREATE INDEX idx_workflow_templates_author_id ON workflow_templates(author_id);
CREATE INDEX idx_workflow_templates_usage_count ON workflow_templates(usage_count);
CREATE INDEX idx_workflow_templates_created_at ON workflow_templates(created_at);
CREATE INDEX idx_workflow_template_usage_template_id ON workflow_template_usage(template_id);
CREATE INDEX idx_workflow_template_usage_user_id ON workflow_template_usage(user_id);
CREATE INDEX idx_workflow_template_usage_used_at ON workflow_template_usage(used_at);
CREATE INDEX idx_workflow_template_usage_template_user ON workflow_template_usage(template_id, user_id);
