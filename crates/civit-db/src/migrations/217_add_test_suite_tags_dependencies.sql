-- Migration 217: Test Suite Tags and Dependencies
-- Adds tag-based filtering, dependency management, and execution ordering for test suites.

CREATE TABLE IF NOT EXISTS test_suite_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    suite_id UUID NOT NULL REFERENCES test_suites(id) ON DELETE CASCADE,
    tag TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(suite_id, tag)
);

CREATE TABLE IF NOT EXISTS test_suite_dependencies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    suite_id UUID NOT NULL REFERENCES test_suites(id),
    depends_on_suite_id UUID NOT NULL REFERENCES test_suites(id),
    dependency_type TEXT NOT NULL DEFAULT 'blocks',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(suite_id, depends_on_suite_id)
);

CREATE INDEX IF NOT EXISTS idx_test_suite_tags_suite_id ON test_suite_tags(suite_id);
CREATE INDEX IF NOT EXISTS idx_test_suite_tags_tag ON test_suite_tags(tag);
CREATE INDEX IF NOT EXISTS idx_test_suite_tags_suite_tag ON test_suite_tags(suite_id, tag);
CREATE INDEX IF NOT EXISTS idx_test_suite_deps_suite_id ON test_suite_dependencies(suite_id);
CREATE INDEX IF NOT EXISTS idx_test_suite_deps_depends_on ON test_suite_dependencies(depends_on_suite_id);
CREATE INDEX IF NOT EXISTS idx_test_suite_deps_type ON test_suite_dependencies(dependency_type);
