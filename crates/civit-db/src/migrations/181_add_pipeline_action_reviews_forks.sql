-- CivitForge Phase 181: Pipeline Actions Reviews & Forks
-- Migration 181
-- Adds action reviews, ratings, forking, recommendations, and analytics.

CREATE TABLE IF NOT EXISTS pipeline_action_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action_id UUID NOT NULL REFERENCES pipeline_actions(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(action_id, user_id)
);

CREATE TABLE IF NOT EXISTS pipeline_action_forks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action_id UUID NOT NULL REFERENCES pipeline_actions(id),
    forked_by UUID NOT NULL REFERENCES users(id),
    new_name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pipeline_action_reviews_action_id ON pipeline_action_reviews(action_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_action_reviews_user_id ON pipeline_action_reviews(user_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_action_reviews_rating ON pipeline_action_reviews(rating);
CREATE INDEX IF NOT EXISTS idx_pipeline_action_forks_action_id ON pipeline_action_forks(action_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_action_forks_forked_by ON pipeline_action_forks(forked_by);
