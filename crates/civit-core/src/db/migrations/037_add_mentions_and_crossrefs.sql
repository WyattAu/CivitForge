-- Migration 037: Add comment mentions and cross-references tables

-- Comment mentions (@username references in comment bodies)
CREATE TABLE IF NOT EXISTS comment_mentions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    comment_id UUID NOT NULL,
    comment_type VARCHAR(16) NOT NULL CHECK (comment_type IN ('issue', 'pr')),
    mentioned_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (comment_id, comment_type, mentioned_user_id)
);
CREATE INDEX IF NOT EXISTS idx_comment_mentions_user ON comment_mentions(mentioned_user_id);
CREATE INDEX IF NOT EXISTS idx_comment_mentions_comment ON comment_mentions(comment_id, comment_type);

-- Cross-references (#NNN references to issues/PRs in comments)
CREATE TABLE IF NOT EXISTS comment_cross_references (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_comment_id UUID NOT NULL,
    source_comment_type VARCHAR(16) NOT NULL CHECK (source_comment_type IN ('issue', 'pr')),
    source_repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    target_number INTEGER NOT NULL,
    target_type VARCHAR(16) NOT NULL CHECK (target_type IN ('issue', 'pr')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_cross_refs_target ON comment_cross_references(target_number, target_type);
CREATE INDEX IF NOT EXISTS idx_cross_refs_source ON comment_cross_references(source_comment_id, source_comment_type);
