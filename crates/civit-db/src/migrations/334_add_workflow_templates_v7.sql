-- CivitForge Phase 334: Workflow Templates V7
-- Migration 334
-- Adds workflow templates v7 with reviews v6, marketplace v7, analytics v7, and recommendations v7.

CREATE TABLE IF NOT EXISTS workflow_templates_v7 (
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

CREATE TABLE IF NOT EXISTS workflow_template_reviews_v6 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES workflow_templates_v7(id),
    user_id UUID NOT NULL REFERENCES users(id),
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review TEXT NOT NULL DEFAULT '',
    helpful_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(template_id, user_id)
);

CREATE INDEX idx_workflow_templates_v7_template_type ON workflow_templates_v7(template_type);
CREATE INDEX idx_workflow_templates_v7_is_public ON workflow_templates_v7(is_public);
CREATE INDEX idx_workflow_templates_v7_author_id ON workflow_templates_v7(author_id);
CREATE INDEX idx_workflow_templates_v7_usage_count ON workflow_templates_v7(usage_count);
CREATE INDEX idx_workflow_templates_v7_rating ON workflow_templates_v7(rating);
CREATE INDEX idx_workflow_templates_v7_created_at ON workflow_templates_v7(created_at);
CREATE INDEX idx_workflow_template_reviews_v6_template_id ON workflow_template_reviews_v6(template_id);
CREATE INDEX idx_workflow_template_reviews_v6_user_id ON workflow_template_reviews_v6(user_id);
CREATE INDEX idx_workflow_template_reviews_v6_rating ON workflow_template_reviews_v6(rating);
CREATE INDEX idx_workflow_template_reviews_v6_helpful ON workflow_template_reviews_v6(helpful_count);
CREATE INDEX idx_workflow_template_reviews_v6_created_at ON workflow_template_reviews_v6(created_at);
