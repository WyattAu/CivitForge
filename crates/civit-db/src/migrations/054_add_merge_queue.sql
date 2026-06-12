-- Migration 054: Add merge queue
CREATE TABLE IF NOT EXISTS merge_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    pr_id UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'queued',
    ci_status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, pr_id)
);
CREATE INDEX IF NOT EXISTS idx_merge_queue_repo ON merge_queue(repo_id);
CREATE INDEX IF NOT EXISTS idx_merge_queue_status ON merge_queue(status);
CREATE INDEX IF NOT EXISTS idx_merge_queue_position ON merge_queue(repo_id, position);
