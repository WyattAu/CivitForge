-- CivitForge Phase 229: Workflow Engine V5 - Template Reviews & Ratings
-- Migration 229
-- Adds workflow template v2 with ratings, reviews, recommendations, analytics, and marketplace.

CREATE TABLE IF NOT EXISTS workflow_templates_v2 (
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

CREATE TABLE IF NOT EXISTS workflow_template_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES workflow_templates_v2(id),
    user_id UUID NOT NULL REFERENCES users(id),
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(template_id, user_id)
);

CREATE INDEX idx_workflow_templates_v2_name ON workflow_templates_v2(name);
CREATE INDEX idx_workflow_templates_v2_template_type ON workflow_templates_v2(template_type);
CREATE INDEX idx_workflow_templates_v2_is_public ON workflow_templates_v2(is_public);
CREATE INDEX idx_workflow_templates_v2_author_id ON workflow_templates_v2(author_id);
CREATE INDEX idx_workflow_templates_v2_usage_count ON workflow_templates_v2(usage_count DESC);
CREATE INDEX idx_workflow_templates_v2_rating ON workflow_templates_v2(rating DESC);
CREATE INDEX idx_workflow_templates_v2_public_rating ON workflow_templates_v2(is_public, rating DESC);
CREATE INDEX idx_workflow_templates_v2_public_usage ON workflow_templates_v2(is_public, usage_count DESC);
CREATE INDEX idx_workflow_template_reviews_template_id ON workflow_template_reviews(template_id);
CREATE INDEX idx_workflow_template_reviews_user_id ON workflow_template_reviews(user_id);
CREATE INDEX idx_workflow_template_reviews_rating ON workflow_template_reviews(rating);
