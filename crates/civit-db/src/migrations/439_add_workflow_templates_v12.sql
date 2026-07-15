-- CivitForge Phase 439: Workflow Engine V15
-- Migration 439
-- Adds workflow_templates_v12 and workflow_template_reviews_v11.

CREATE TABLE IF NOT EXISTS workflow_templates_v12 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    template_type TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}',
    is_public BOOLEAN NOT NULL DEFAULT false,
    author_id UUID REFERENCES users(id),
    usage_count INTEGER NOT NULL DEFAULT 0,
    rating DOUBLE PRECISION NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS workflow_template_reviews_v11 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES workflow_templates_v12(id),
    user_id UUID NOT NULL REFERENCES users(id),
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review TEXT NOT NULL DEFAULT '',
    helpful_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(template_id, user_id)
);

CREATE INDEX idx_workflow_templates_v12_type ON workflow_templates_v12(template_type);
CREATE INDEX idx_workflow_templates_v12_public ON workflow_templates_v12(is_public);
CREATE INDEX idx_workflow_templates_v12_author ON workflow_templates_v12(author_id);
CREATE INDEX idx_workflow_templates_v12_rating ON workflow_templates_v12(rating DESC);
CREATE INDEX idx_workflow_templates_v12_usage ON workflow_templates_v12(usage_count DESC);
CREATE INDEX idx_workflow_template_reviews_v11_template ON workflow_template_reviews_v11(template_id);
CREATE INDEX idx_workflow_template_reviews_v11_user ON workflow_template_reviews_v11(user_id);
