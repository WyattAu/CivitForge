-- Migration 003: SSH Keys, Branches, Pipeline Steps, Event Log, Sessions
-- Phase 1.1: Foundation Hardening

-- Sessions table (referenced by SessionManager in session.rs)
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL UNIQUE,
    ip VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- SSH public keys (replaces InMemorySshKeyStore)
CREATE TABLE IF NOT EXISTS ssh_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_type VARCHAR(50) NOT NULL,
    public_key TEXT NOT NULL,
    fingerprint VARCHAR(64) NOT NULL UNIQUE,
    label VARCHAR(255) NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Branches (per-repo branch tracking)
CREATE TABLE IF NOT EXISTS branches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    commit_sha VARCHAR(64) NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, name)
);

-- Pipeline steps (individual step records for each pipeline run)
CREATE TABLE IF NOT EXISTS pipeline_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pipeline_id UUID NOT NULL REFERENCES pipelines(id) ON DELETE CASCADE,
    step_index INT NOT NULL,
    name VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    image VARCHAR(512) NOT NULL DEFAULT '',
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    exit_code INT,
    output TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(pipeline_id, step_index)
);

-- Event log (persistent event store for WebSocket replay)
CREATE TABLE IF NOT EXISTS event_log (
    id BIGSERIAL PRIMARY KEY,
    event_type VARCHAR(100) NOT NULL,
    topic VARCHAR(255) NOT NULL DEFAULT '',
    payload JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_sessions_user ON sessions(user_id);
CREATE INDEX idx_sessions_hash ON sessions(token_hash);
CREATE INDEX idx_sessions_expires ON sessions(expires_at);
CREATE INDEX idx_ssh_keys_user ON ssh_keys(user_id);
CREATE INDEX idx_ssh_keys_fingerprint ON ssh_keys(fingerprint);
CREATE INDEX idx_branches_repo ON branches(repo_id);
CREATE INDEX idx_branches_default ON branches(repo_id, is_default) WHERE is_default = true;
CREATE INDEX idx_pipeline_steps_pipeline ON pipeline_steps(pipeline_id);
CREATE INDEX idx_pipeline_steps_status ON pipeline_steps(status);
CREATE INDEX idx_event_log_type ON event_log(event_type);
CREATE INDEX idx_event_log_topic ON event_log(topic);
CREATE INDEX idx_event_log_created ON event_log(created_at);
