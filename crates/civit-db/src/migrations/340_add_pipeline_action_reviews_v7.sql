CREATE TABLE IF NOT EXISTS pipeline_action_reviews_v7 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action_id UUID NOT NULL REFERENCES pipeline_actions(id),
    user_id UUID NOT NULL REFERENCES users(id),
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review TEXT NOT NULL DEFAULT '',
    helpful_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(action_id, user_id)
);

CREATE TABLE IF NOT EXISTS review_helpfulness_v7 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    review_id UUID NOT NULL REFERENCES pipeline_action_reviews_v7(id),
    user_id UUID NOT NULL REFERENCES users(id),
    helpful BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(review_id, user_id)
);

CREATE TABLE IF NOT EXISTS review_moderation_queue_v7 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    review_id UUID NOT NULL REFERENCES pipeline_action_reviews_v7(id),
    status TEXT NOT NULL DEFAULT 'pending',
    moderator_id UUID REFERENCES users(id),
    reason TEXT,
    moderated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS review_analytics_v7 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action_id UUID NOT NULL REFERENCES pipeline_actions(id),
    period_start TIMESTAMPTZ NOT NULL,
    total_reviews INTEGER NOT NULL DEFAULT 0,
    avg_rating NUMERIC(3,2) NOT NULL DEFAULT 0,
    rating_distribution JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS review_recommendations_v7 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action_id UUID NOT NULL REFERENCES pipeline_actions(id),
    user_id UUID NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL DEFAULT '',
    confidence NUMERIC(5,4) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(action_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_pipeline_action_reviews_v7_action ON pipeline_action_reviews_v7(action_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_action_reviews_v7_user ON pipeline_action_reviews_v7(user_id);
CREATE INDEX IF NOT EXISTS idx_review_helpfulness_v7_review ON review_helpfulness_v7(review_id);
CREATE INDEX IF NOT EXISTS idx_review_moderation_queue_v7_status ON review_moderation_queue_v7(status);
CREATE INDEX IF NOT EXISTS idx_review_analytics_v7_action ON review_analytics_v7(action_id);
CREATE INDEX IF NOT EXISTS idx_review_recommendations_v7_action ON review_recommendations_v7(action_id);
