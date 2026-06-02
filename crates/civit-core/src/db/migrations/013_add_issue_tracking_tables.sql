-- CivitForge Phase 11: Issue Tracking
-- Migration 013
-- Adds issue_comments, labels, milestones, issue_labels, issue_assignees,
-- and timeline/change tracking tables.
-- NOTE: issues table already exists from migration 001 with UUID id.

-- Issue Comments
CREATE TABLE IF NOT EXISTS issue_comments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    issue_id        UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    author_id       UUID NOT NULL REFERENCES users(id),
    body            TEXT NOT NULL,
    is_edited       BOOLEAN NOT NULL DEFAULT FALSE,
    edited_at       TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Labels (global per-repo)
CREATE TABLE IF NOT EXISTS labels (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    color           TEXT NOT NULL DEFAULT '#808080',  -- hex color code
    description     TEXT DEFAULT '',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, name)
);

-- Issue ↔ Label many-to-many
CREATE TABLE IF NOT EXISTS issue_labels (
    issue_id        UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    label_id        UUID NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
    PRIMARY KEY (issue_id, label_id)
);

-- Issue ↔ Assignee (v1.0: single assignee, but table supports multiple)
CREATE TABLE IF NOT EXISTS issue_assignees (
    issue_id        UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id),
    assigned_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (issue_id, user_id)
);

-- Milestones (per-repo)
CREATE TABLE IF NOT EXISTS milestones (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id         UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    title           TEXT NOT NULL,
    description     TEXT DEFAULT '',
    state           TEXT NOT NULL DEFAULT 'open', -- 'open', 'closed'
    due_on          DATE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, title)
);

-- Issue Timeline (tracks all state changes for audit trail)
CREATE TABLE IF NOT EXISTS issue_timeline (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    issue_id        UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    actor_id        UUID NOT NULL REFERENCES users(id),
    event_type      TEXT NOT NULL,               -- 'opened', 'closed', 'reopened', 'assigned',
                                            -- 'unassigned', 'labeled', 'unlabeled',
                                            -- 'milestoned', 'demilestoned', 'edited',
                                            -- 'transferred', 'locked', 'unlocked', 'commented'
    event_detail    TEXT DEFAULT '',            -- JSON with event-specific data
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Issue Reactions (emoji reactions on comments/issues)
CREATE TABLE IF NOT EXISTS issue_reactions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    issue_id        UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    comment_id      UUID REFERENCES issue_comments(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id),
    emoji           TEXT NOT NULL,               -- e.g. "thumbs_up", "heart", "rocket"
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(issue_id, user_id, emoji)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_issue_comments_issue ON issue_comments(issue_id);
CREATE INDEX IF NOT EXISTS idx_labels_repo ON labels(repo_id);
CREATE INDEX IF NOT EXISTS idx_issue_labels_issue ON issue_labels(issue_id);
CREATE INDEX IF NOT EXISTS idx_issue_labels_label ON issue_labels(label_id);
CREATE INDEX IF NOT EXISTS idx_issue_assignees_issue ON issue_assignees(issue_id);
CREATE INDEX IF NOT EXISTS idx_milestones_repo ON milestones(repo_id, state);
CREATE INDEX IF NOT EXISTS idx_issue_timeline_issue ON issue_timeline(issue_id);
CREATE INDEX IF NOT EXISTS idx_issue_reactions_issue ON issue_reactions(issue_id);
