CREATE TABLE IF NOT EXISTS test_coverage_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    line_coverage DOUBLE PRECISION NOT NULL DEFAULT 0,
    branch_coverage DOUBLE PRECISION NOT NULL DEFAULT 0,
    function_coverage DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_lines INTEGER NOT NULL DEFAULT 0,
    covered_lines INTEGER NOT NULL DEFAULT 0,
    uncovered_lines INTEGER[] NOT NULL DEFAULT '{}',
    measured_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_test_coverage_v2_repo_id ON test_coverage_v2(repo_id);
CREATE INDEX idx_test_coverage_v2_measured_at ON test_coverage_v2(measured_at);
CREATE INDEX idx_test_coverage_v2_repo_file ON test_coverage_v2(repo_id, file_path);
