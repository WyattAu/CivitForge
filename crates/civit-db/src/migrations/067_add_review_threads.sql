ALTER TABLE pr_comments ADD COLUMN IF NOT EXISTS resolved BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE pr_comments ADD COLUMN IF NOT EXISTS resolved_by UUID REFERENCES users(id);

CREATE TABLE IF NOT EXISTS pr_review_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pr_id UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    team TEXT NOT NULL DEFAULT '',
    assigned_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(pr_id, user_id)
);
