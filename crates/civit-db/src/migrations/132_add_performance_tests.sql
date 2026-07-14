CREATE TABLE IF NOT EXISTS performance_tests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    test_type TEXT NOT NULL,
    endpoint TEXT,
    config JSONB NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'pending',
    results JSONB NOT NULL DEFAULT '{}',
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_performance_tests_repo_id ON performance_tests(repo_id);
CREATE INDEX idx_performance_tests_status ON performance_tests(status);
CREATE INDEX idx_performance_tests_test_type ON performance_tests(test_type);
