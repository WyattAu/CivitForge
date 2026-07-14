-- Migration 151: Test Suite Management
-- Adds test suite management with test runs and result tracking.

CREATE TABLE IF NOT EXISTS test_suites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    test_type TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS test_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    suite_id UUID NOT NULL REFERENCES test_suites(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    total_tests INTEGER NOT NULL DEFAULT 0,
    passed_tests INTEGER NOT NULL DEFAULT 0,
    failed_tests INTEGER NOT NULL DEFAULT 0,
    skipped_tests INTEGER NOT NULL DEFAULT 0,
    duration_ms INTEGER NOT NULL DEFAULT 0,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_test_suites_repo ON test_suites(repo_id);
CREATE INDEX IF NOT EXISTS idx_test_suites_type ON test_suites(test_type);
CREATE INDEX IF NOT EXISTS idx_test_runs_suite ON test_runs(suite_id);
CREATE INDEX IF NOT EXISTS idx_test_runs_status ON test_runs(status);
