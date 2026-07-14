-- CivitForge Phase 271: Workflow Templates V4
-- Migration 271
-- Adds workflow templates v4 with rating, reviews v3 with helpfulness, marketplace, analytics, and recommendations.

CREATE TABLE IF NOT EXISTS workflow_templates_v4 (
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

CREATE TABLE IF NOT EXISTS workflow_template_reviews_v3 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES workflow_templates_v4(id),
    user_id UUID NOT NULL REFERENCES users(id),
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review TEXT NOT NULL DEFAULT '',
    helpful_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(template_id, user_id)
);

CREATE INDEX idx_workflow_templates_v4_template_type ON workflow_templates_v4(template_type);
CREATE INDEX idx_workflow_templates_v4_is_public ON workflow_templates_v4(is_public);
CREATE INDEX idx_workflow_templates_v4_author_id ON workflow_templates_v4(author_id);
CREATE INDEX idx_workflow_templates_v4_usage_count ON workflow_templates_v4(usage_count);
CREATE INDEX idx_workflow_templates_v4_rating ON workflow_templates_v4(rating);
CREATE INDEX idx_workflow_templates_v4_created_at ON workflow_templates_v4(created_at);
CREATE INDEX idx_workflow_template_reviews_v3_template_id ON workflow_template_reviews_v3(template_id);
CREATE INDEX idx_workflow_template_reviews_v3_user_id ON workflow_template_reviews_v3(user_id);
CREATE INDEX idx_workflow_template_reviews_v3_rating ON workflow_template_reviews_v3(rating);
CREATE INDEX idx_workflow_template_reviews_v3_helpful ON workflow_template_reviews_v3(helpful_count);
CREATE INDEX idx_workflow_template_reviews_v3_created_at ON workflow_template_reviews_v3(created_at);
