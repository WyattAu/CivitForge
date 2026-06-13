-- Migration 057: Add CODEOWNERS review tracking for PR enforcement
CREATE TABLE IF NOT EXISTS codeowners_reviews (
    id BIGSERIAL PRIMARY KEY,
    pr_id BIGINT NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
    reviewer TEXT NOT NULL,
    approved BOOLEAN NOT NULL DEFAULT FALSE,
    approved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(pr_id, reviewer)
);
CREATE INDEX IF NOT EXISTS idx_codeowners_reviews_pr_id ON codeowners_reviews(pr_id);
