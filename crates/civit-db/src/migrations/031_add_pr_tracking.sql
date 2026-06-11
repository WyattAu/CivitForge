-- Migration 031: Add PR tracking tables (comments, labels, assignees, reviewers, timeline)
-- PR base table already exists from migration 001

-- Add draft + commit SHA columns to pull_requests
ALTER TABLE pull_requests ADD COLUMN IF NOT EXISTS draft BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE pull_requests ADD COLUMN IF NOT EXISTS head_commit_sha VARCHAR(64);
ALTER TABLE pull_requests ADD COLUMN IF NOT EXISTS base_commit_sha VARCHAR(64);
ALTER TABLE pull_requests ADD COLUMN IF NOT EXISTS merge_strategy VARCHAR(32) NOT NULL DEFAULT 'merge';

-- PR comments (separate from issue comments — different workflows)
CREATE TABLE IF NOT EXISTS pr_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pr_id UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
    author_id UUID NOT NULL REFERENCES users(id),
    body TEXT NOT NULL,
    commit_sha VARCHAR(64),
    file_path VARCHAR(1024),
    start_line INTEGER,
    end_line INTEGER,
    line INTEGER,
    in_reply_to_id UUID REFERENCES pr_comments(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_pr_comments_pr_id ON pr_comments(pr_id);
CREATE INDEX IF NOT EXISTS idx_pr_comments_author ON pr_comments(author_id);

-- PR labels (reuse existing labels table, just add junction)
CREATE TABLE IF NOT EXISTS pr_labels (
    pr_id UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
    label_id UUID NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
    PRIMARY KEY (pr_id, label_id)
);

-- PR assignees (reuse users table)
CREATE TABLE IF NOT EXISTS pr_assignees (
    pr_id UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (pr_id, user_id)
);
CREATE INDEX IF NOT EXISTS idx_pr_assignees_pr_id ON pr_assignees(pr_id);
CREATE INDEX IF NOT EXISTS idx_pr_assignees_user_id ON pr_assignees(user_id);

-- PR reviewers
CREATE TABLE IF NOT EXISTS pr_reviewers (
    pr_id UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    review_status VARCHAR(32) NOT NULL DEFAULT 'pending',
    submitted_at TIMESTAMPTZ,
    PRIMARY KEY (pr_id, user_id)
);
CREATE INDEX IF NOT EXISTS idx_pr_reviewers_pr_id ON pr_reviewers(pr_id);

-- PR timeline events (for activity feed on PR detail page)
CREATE TABLE IF NOT EXISTS pr_timeline (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pr_id UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
    actor_id UUID NOT NULL REFERENCES users(id),
    event_type VARCHAR(64) NOT NULL,
    event_detail JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_pr_timeline_pr_id ON pr_timeline(pr_id);

-- PR status checks (for CI integration)
CREATE TABLE IF NOT EXISTS pr_status_checks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pr_id UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
    context VARCHAR(128) NOT NULL,
    state VARCHAR(32) NOT NULL DEFAULT 'pending',
    description TEXT NOT NULL DEFAULT '',
    target_url VARCHAR(1024),
    commit_sha VARCHAR(64),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_pr_status_checks_pr_id ON pr_status_checks(pr_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_pr_status_checks_unique ON pr_status_checks(pr_id, context, commit_sha);
