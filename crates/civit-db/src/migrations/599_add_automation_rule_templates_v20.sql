-- Automation Rule Templates v20: template management, ratings, search, recommendations
CREATE TABLE IF NOT EXISTS automation_rule_templates_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT 'general',
    rule_definition JSONB NOT NULL,
    usage_count INTEGER NOT NULL DEFAULT 0,
    rating DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_automation_rule_templates_v20_category ON automation_rule_templates_v20(category);
CREATE INDEX IF NOT EXISTS idx_automation_rule_templates_v20_rating ON automation_rule_templates_v20(rating DESC);
CREATE INDEX IF NOT EXISTS idx_automation_rule_templates_v20_usage ON automation_rule_templates_v20(usage_count DESC);
CREATE INDEX IF NOT EXISTS idx_automation_rule_templates_v20_name_search ON automation_rule_templates_v20 USING gin(name gin_trgm_ops);

CREATE TABLE IF NOT EXISTS automation_rule_template_ratings_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES automation_rule_templates_v20(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(template_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_automation_rule_template_ratings_v20_template ON automation_rule_template_ratings_v20(template_id);
CREATE INDEX IF NOT EXISTS idx_automation_rule_template_ratings_v20_user ON automation_rule_template_ratings_v20(user_id);
