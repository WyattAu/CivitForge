CREATE TABLE IF NOT EXISTS discussion_labels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    discussion_id UUID NOT NULL REFERENCES discussions(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    color TEXT NOT NULL DEFAULT '#3b82f6',
    UNIQUE(discussion_id, label)
);

CREATE INDEX IF NOT EXISTS idx_discussion_labels_discussion_id ON discussion_labels(discussion_id);

CREATE TABLE IF NOT EXISTS discussion_reactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    comment_id UUID NOT NULL REFERENCES discussion_comments(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    emoji TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(comment_id, user_id, emoji)
);

CREATE INDEX IF NOT EXISTS idx_discussion_reactions_comment_id ON discussion_reactions(comment_id);
