-- CivitForge Phase 313: Workflow Templates V6
-- Migration 313
-- Adds workflow templates v6 with reviews v5, marketplace v6, analytics v6, and recommendations v6.

CREATE TABLE IF NOT EXISTS workflow_templates_v6 (
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

CREATE TABLE IF NOT EXISTS workflow_template_reviews_v5 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES workflow_templates_v6(id),
    user_id UUID NOT NULL REFERENCES users(id),
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review TEXT NOT NULL DEFAULT '',
    helpful_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(template_id, user_id)
);

CREATE INDEX idx_workflow_templates_v6_template_type ON workflow_templates_v6(template_type);
CREATE INDEX idx_workflow_templates_v6_is_public ON workflow_templates_v6(is_public);
CREATE INDEX idx_workflow_templates_v6_author_id ON workflow_templates_v6(author_id);
CREATE INDEX idx_workflow_templates_v6_usage_count ON workflow_templates_v6(usage_count);
CREATE INDEX idx_workflow_templates_v6_rating ON workflow_templates_v6(rating);
CREATE INDEX idx_workflow_templates_v6_created_at ON workflow_templates_v6(created_at);
CREATE INDEX idx_workflow_template_reviews_v5_template_id ON workflow_template_reviews_v5(template_id);
CREATE INDEX idx_workflow_template_reviews_v5_user_id ON workflow_template_reviews_v5(user_id);
CREATE INDEX idx_workflow_template_reviews_v5_rating ON workflow_template_reviews_v5(rating);
CREATE INDEX idx_workflow_template_reviews_v5_helpful ON workflow_template_reviews_v5(helpful_count);
CREATE INDEX idx_workflow_template_reviews_v5_created_at ON workflow_template_reviews_v5(created_at);
