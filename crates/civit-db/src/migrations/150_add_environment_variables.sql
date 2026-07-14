-- Migration 150: Environment Variables v3
-- Adds environment variable management with secret support and inheritance.

CREATE TABLE IF NOT EXISTS environment_variables (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id UUID NOT NULL REFERENCES pipeline_environments(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    value TEXT NOT NULL,
    encrypted BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(environment_id, name)
);

CREATE TABLE IF NOT EXISTS environment_variable_inheritance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    child_env_id UUID NOT NULL REFERENCES pipeline_environments(id) ON DELETE CASCADE,
    parent_env_id UUID NOT NULL REFERENCES pipeline_environments(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(child_env_id, parent_env_id)
);

CREATE INDEX IF NOT EXISTS idx_environment_variables_env ON environment_variables(environment_id);
CREATE INDEX IF NOT EXISTS idx_environment_variable_inheritance_child ON environment_variable_inheritance(child_env_id);
CREATE INDEX IF NOT EXISTS idx_environment_variable_inheritance_parent ON environment_variable_inheritance(parent_env_id);
