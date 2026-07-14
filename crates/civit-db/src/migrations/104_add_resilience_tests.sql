CREATE TABLE IF NOT EXISTS resilience_tests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    test_type TEXT NOT NULL,
    target TEXT NOT NULL,
    parameters JSONB NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'pending',
    score INTEGER NOT NULL DEFAULT 0,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_resilience_tests_status ON resilience_tests(status);
CREATE INDEX idx_resilience_tests_type ON resilience_tests(test_type);
