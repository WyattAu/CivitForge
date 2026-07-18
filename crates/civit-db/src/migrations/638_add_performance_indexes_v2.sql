-- Performance indexes for hot paths - Migration 638
-- Adds composite and covering indexes for the most frequent query patterns
-- Wrapped in DO blocks to handle missing tables gracefully

DO $$
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE tablename = 'repositories') THEN
        CREATE INDEX IF NOT EXISTS idx_repositories_name ON repositories(name);
        CREATE INDEX IF NOT EXISTS idx_repositories_owner_name ON repositories(owner_id, name);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE tablename = 'issues') THEN
        CREATE INDEX IF NOT EXISTS idx_issues_repo_status ON issues(repo_id, status);
        CREATE INDEX IF NOT EXISTS idx_issues_created_at ON issues(created_at DESC);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE tablename = 'pull_requests') THEN
        CREATE INDEX IF NOT EXISTS idx_pull_requests_repo_status ON pull_requests(repo_id, status);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE tablename = 'pipeline_runs') THEN
        CREATE INDEX IF NOT EXISTS idx_pipeline_runs_repo_status ON pipeline_runs(repo_id, status);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE tablename = 'commits') THEN
        CREATE INDEX IF NOT EXISTS idx_commits_repo_date ON commits(repo_id, committed_at DESC);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE tablename = 'events') THEN
        CREATE INDEX IF NOT EXISTS idx_events_type_created ON events(event_type, created_at DESC);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE tablename = 'audit_events') THEN
        CREATE INDEX IF NOT EXISTS idx_audit_events_user_created ON audit_events(actor_id, created_at DESC);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE tablename = 'access_tokens') THEN
        CREATE INDEX IF NOT EXISTS idx_access_tokens_hash ON access_tokens(token_hash);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE tablename = 'sessions') THEN
        CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
    END IF;
END $$;
