CREATE TABLE IF NOT EXISTS code_suggestions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pr_id UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
    comment_id UUID REFERENCES pr_comments(id),
    file_path TEXT NOT NULL,
    start_line INTEGER NOT NULL,
    end_line INTEGER NOT NULL,
    suggestion TEXT NOT NULL,
    applied BOOLEAN NOT NULL DEFAULT false,
    author_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_code_suggestions_pr_id ON code_suggestions(pr_id);
CREATE INDEX IF NOT EXISTS idx_code_suggestions_author_id ON code_suggestions(author_id);
