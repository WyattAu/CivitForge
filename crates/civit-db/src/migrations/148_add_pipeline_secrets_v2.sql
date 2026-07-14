-- Migration 148: Advanced Pipeline Secrets Management v2
-- Adds environment-specific secrets, rotation tracking, and audit logging.

CREATE TABLE IF NOT EXISTS pipeline_secrets_v2 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    encrypted_value BYTEA NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    environment TEXT NOT NULL DEFAULT 'all',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, name, environment)
);

CREATE TABLE IF NOT EXISTS secret_rotation_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    secret_id UUID NOT NULL REFERENCES pipeline_secrets_v2(id) ON DELETE CASCADE,
    rotated_by UUID REFERENCES users(id),
    rotated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reason TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS secret_access_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    secret_id UUID NOT NULL REFERENCES pipeline_secrets_v2(id) ON DELETE CASCADE,
    accessed_by UUID REFERENCES users(id),
    access_type TEXT NOT NULL DEFAULT 'read',
    accessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address TEXT
);

CREATE INDEX IF NOT EXISTS idx_pipeline_secrets_v2_repo ON pipeline_secrets_v2(repo_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_secrets_v2_env ON pipeline_secrets_v2(environment);
CREATE INDEX IF NOT EXISTS idx_secret_rotation_log_secret ON secret_rotation_log(secret_id);
CREATE INDEX IF NOT EXISTS idx_secret_access_log_secret ON secret_access_log(secret_id);
