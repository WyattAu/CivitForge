-- Migration 050: Add issue and PR features
-- Issue pinning
ALTER TABLE issues ADD COLUMN IF NOT EXISTS is_pinned BOOLEAN NOT NULL DEFAULT false;
-- Issue locking
ALTER TABLE issues ADD COLUMN IF NOT EXISTS is_locked BOOLEAN NOT NULL DEFAULT false;
-- Issue due dates
ALTER TABLE issues ADD COLUMN IF NOT EXISTS due_date TIMESTAMPTZ;
-- Issue time tracking
CREATE TABLE IF NOT EXISTS issue_time_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    issue_id UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    hours NUMERIC NOT NULL CHECK (hours > 0),
    description TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_issue_time_entries_issue_id ON issue_time_entries(issue_id);
-- Issue dependencies
CREATE TABLE IF NOT EXISTS issue_dependencies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    blocking_issue_id UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    blocked_by_issue_id UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    dependency_type TEXT NOT NULL DEFAULT 'blocks',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(blocking_issue_id, blocked_by_issue_id)
);
CREATE INDEX IF NOT EXISTS idx_issue_dependencies_blocking ON issue_dependencies(blocking_issue_id);
CREATE INDEX IF NOT EXISTS idx_issue_dependencies_blocked_by ON issue_dependencies(blocked_by_issue_id);
-- PR auto-merge
ALTER TABLE pull_requests ADD COLUMN IF NOT EXISTS auto_merge BOOLEAN NOT NULL DEFAULT false;
