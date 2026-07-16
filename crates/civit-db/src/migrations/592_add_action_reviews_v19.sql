-- Pipeline Actions Marketplace v22: Review v19 table
CREATE TABLE IF NOT EXISTS pipeline_action_reviews_v19 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action_id UUID NOT NULL REFERENCES pipeline_actions(id),
    user_id UUID NOT NULL REFERENCES users(id),
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review TEXT NOT NULL DEFAULT '',
    helpful_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(action_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_action_reviews_v19_action ON pipeline_action_reviews_v19(action_id);
CREATE INDEX IF NOT EXISTS idx_action_reviews_v19_user ON pipeline_action_reviews_v19(user_id);
CREATE INDEX IF NOT EXISTS idx_action_reviews_v19_rating ON pipeline_action_reviews_v19(rating);
CREATE INDEX IF NOT EXISTS idx_action_reviews_v19_helpful ON pipeline_action_reviews_v19(helpful_count DESC);
