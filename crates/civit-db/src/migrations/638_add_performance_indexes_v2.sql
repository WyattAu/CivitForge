-- Performance indexes for hot paths - Migration 638
-- Adds composite and covering indexes for the most frequent query patterns

-- Repository lookup optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_repositories_name ON repositories(name);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_repositories_owner_name ON repositories(owner_id, name);

-- Issue query patterns (repo + status + created_at for sorting)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_issues_repo_status ON issues(repo_id, status);
CREATE INDEX IF NOT EXISTS idx_issues_created_at ON issues(created_at DESC);

-- Pull request query patterns
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_pull_requests_repo_status ON pull_requests(repository_id, status);

-- Pipeline runs (heavily queried for status dashboards)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_pipeline_runs_repo_status ON pipeline_runs(repository_id, status);

-- Commit history (time-ordered within repos)
CREATE INDEX IF NOT EXISTS idx_commits_repo_date ON commits(repository_id, committed_at DESC);

-- Event queries (type-based filtering with time ordering)
CREATE INDEX IF NOT EXISTS idx_events_type_created ON events(event_type, created_at DESC);

-- Audit events (user + time for compliance queries)
CREATE INDEX IF NOT EXISTS idx_audit_events_user_created ON audit_events(user_id, created_at DESC);

-- Access token validation (frequent auth path)
CREATE INDEX IF NOT EXISTS idx_access_tokens_hash ON access_tokens(token_hash);

-- Session lookups
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
