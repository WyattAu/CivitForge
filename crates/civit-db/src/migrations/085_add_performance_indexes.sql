-- Performance indexes for common queries
-- Migration 085

-- Repository lookup indexes
CREATE INDEX IF NOT EXISTS idx_repositories_owner_id ON repositories(owner_id);
CREATE INDEX IF NOT EXISTS idx_repositories_visibility ON repositories(visibility);

-- Issue and PR query optimization
CREATE INDEX IF NOT EXISTS idx_issues_repo_id_status ON issues(repo_id, status);
CREATE INDEX IF NOT EXISTS idx_pull_requests_repo_id_status ON pull_requests(repo_id, status);

-- Pipeline performance
CREATE INDEX IF NOT EXISTS idx_pipeline_runs_repo_id ON pipeline_runs(repo_id);

-- Audit log query optimization
CREATE INDEX IF NOT EXISTS idx_audit_events_created_at ON audit_events(created_at);
CREATE INDEX IF NOT EXISTS idx_audit_events_user_id ON audit_events(user_id);

-- User activity indexes
CREATE INDEX IF NOT EXISTS idx_stars_user_id ON repo_stars(user_id);
CREATE INDEX IF NOT EXISTS idx_watchers_user_id ON repo_watchers(user_id);

-- Comment lookup optimization
CREATE INDEX IF NOT EXISTS idx_comments_pr_id ON pr_comments(pr_id);
CREATE INDEX IF NOT EXISTS idx_comments_issue_id ON issue_comments(issue_id);
