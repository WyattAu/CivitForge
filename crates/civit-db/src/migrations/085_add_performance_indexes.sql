-- Performance indexes for common queries - Migration 085
-- Wrapped in DO blocks to handle missing tables gracefully

DO $$
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE tablename = 'repositories') THEN
        CREATE INDEX IF NOT EXISTS idx_repositories_owner_id ON repositories(owner_id);
        CREATE INDEX IF NOT EXISTS idx_repositories_visibility ON repositories(visibility);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE tablename = 'issues') THEN
        CREATE INDEX IF NOT EXISTS idx_issues_repo_id_status ON issues(repo_id, status);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE tablename = 'pull_requests') THEN
        CREATE INDEX IF NOT EXISTS idx_pull_requests_repo_id_status ON pull_requests(repo_id, status);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE tablename = 'pipeline_runs') THEN
        CREATE INDEX IF NOT EXISTS idx_pipeline_runs_repo_id ON pipeline_runs(repo_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE tablename = 'audit_events') THEN
        CREATE INDEX IF NOT EXISTS idx_audit_events_created_at ON audit_events(created_at);
        CREATE INDEX IF NOT EXISTS idx_audit_events_actor_id ON audit_events(actor_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE tablename = 'pr_comments') THEN
        CREATE INDEX IF NOT EXISTS idx_comments_pr_id ON pr_comments(pr_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE tablename = 'issue_comments') THEN
        CREATE INDEX IF NOT EXISTS idx_comments_issue_id ON issue_comments(issue_id);
    END IF;
END $$;
