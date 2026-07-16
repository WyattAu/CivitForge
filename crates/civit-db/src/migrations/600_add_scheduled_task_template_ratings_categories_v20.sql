-- Scheduled Task Template Ratings and Categories v20: ratings, categories, search, recommendations
CREATE TABLE IF NOT EXISTS scheduled_task_template_ratings_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES scheduled_task_templates_v18(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(template_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_scheduled_task_template_ratings_v20_template ON scheduled_task_template_ratings_v20(template_id);
CREATE INDEX IF NOT EXISTS idx_scheduled_task_template_ratings_v20_user ON scheduled_task_template_ratings_v20(user_id);

CREATE TABLE IF NOT EXISTS scheduled_task_template_categories_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    parent_id UUID REFERENCES scheduled_task_template_categories_v20(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_scheduled_task_template_categories_v20_parent ON scheduled_task_template_categories_v20(parent_id);
CREATE INDEX IF NOT EXISTS idx_scheduled_task_template_categories_v20_name ON scheduled_task_template_categories_v20(name);
