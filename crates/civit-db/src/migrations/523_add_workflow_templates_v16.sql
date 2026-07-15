CREATE TABLE IF NOT EXISTS workflow_templates_v16 (
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

CREATE TABLE IF NOT EXISTS workflow_template_reviews_v15 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES workflow_templates_v16(id),
    user_id UUID NOT NULL REFERENCES users(id),
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review TEXT NOT NULL DEFAULT '',
    helpful_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(template_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_workflow_templates_v16_name ON workflow_templates_v16(name);
CREATE INDEX IF NOT EXISTS idx_workflow_templates_v16_template_type ON workflow_templates_v16(template_type);
CREATE INDEX IF NOT EXISTS idx_workflow_templates_v16_is_public ON workflow_templates_v16(is_public);
CREATE INDEX IF NOT EXISTS idx_workflow_templates_v16_author_id ON workflow_templates_v16(author_id);
CREATE INDEX IF NOT EXISTS idx_workflow_templates_v16_usage_count ON workflow_templates_v16(usage_count);
CREATE INDEX IF NOT EXISTS idx_workflow_templates_v16_rating ON workflow_templates_v16(rating);
CREATE INDEX IF NOT EXISTS idx_workflow_templates_v16_created_at ON workflow_templates_v16(created_at);

CREATE INDEX IF NOT EXISTS idx_workflow_template_reviews_v15_template_id ON workflow_template_reviews_v15(template_id);
CREATE INDEX IF NOT EXISTS idx_workflow_template_reviews_v15_user_id ON workflow_template_reviews_v15(user_id);
CREATE INDEX IF NOT EXISTS idx_workflow_template_reviews_v15_rating ON workflow_template_reviews_v15(rating);
CREATE INDEX IF NOT EXISTS idx_workflow_template_reviews_v15_helpful_count ON workflow_template_reviews_v15(helpful_count);
CREATE INDEX IF NOT EXISTS idx_workflow_template_reviews_v15_created_at ON workflow_template_reviews_v15(created_at);
