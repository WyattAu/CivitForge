CREATE TABLE IF NOT EXISTS pipeline_action_reviews_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action_id UUID NOT NULL REFERENCES pipeline_actions(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review TEXT NOT NULL DEFAULT '',
    helpful_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(action_id, user_id)
);

CREATE TABLE IF NOT EXISTS review_helpfulness (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    review_id UUID NOT NULL REFERENCES pipeline_action_reviews_v2(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    helpful BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(review_id, user_id)
);

CREATE TABLE IF NOT EXISTS review_moderation_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    review_id UUID NOT NULL REFERENCES pipeline_action_reviews_v2(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    moderator_id UUID REFERENCES users(id),
    reason TEXT,
    moderated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_review_v2_action_id ON pipeline_action_reviews_v2(action_id);
CREATE INDEX IF NOT EXISTS idx_review_v2_user_id ON pipeline_action_reviews_v2(user_id);
CREATE INDEX IF NOT EXISTS idx_review_v2_rating ON pipeline_action_reviews_v2(rating);
CREATE INDEX IF NOT EXISTS idx_review_helpfulness_review_id ON review_helpfulness(review_id);
CREATE INDEX IF NOT EXISTS idx_review_moderation_status ON review_moderation_queue(status);
