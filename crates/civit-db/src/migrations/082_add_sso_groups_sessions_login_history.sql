CREATE TABLE IF NOT EXISTS sso_group_mappings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sso_provider TEXT NOT NULL,
    sso_group_name TEXT NOT NULL,
    civitforge_role TEXT NOT NULL DEFAULT 'member',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (sso_provider, sso_group_name)
);

CREATE TABLE IF NOT EXISTS active_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    session_token_hash TEXT NOT NULL UNIQUE,
    ip_address TEXT NOT NULL DEFAULT '',
    user_agent TEXT NOT NULL DEFAULT '',
    provider TEXT NOT NULL DEFAULT 'local',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_active_sessions_user_id ON active_sessions (user_id);
CREATE INDEX IF NOT EXISTS idx_active_sessions_expires_at ON active_sessions (expires_at);

CREATE TABLE IF NOT EXISTS login_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT NOT NULL,
    provider TEXT NOT NULL DEFAULT 'local',
    success BOOLEAN NOT NULL DEFAULT false,
    ip_address TEXT NOT NULL DEFAULT '',
    user_agent TEXT NOT NULL DEFAULT '',
    failure_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_login_history_username ON login_history (username);
CREATE INDEX IF NOT EXISTS idx_login_history_created_at ON login_history (created_at);
