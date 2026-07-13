-- Horizontal scaling: server instances and sticky sessions
-- Migration 088

CREATE TABLE IF NOT EXISTS server_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hostname TEXT NOT NULL UNIQUE,
    ip_address INET NOT NULL,
    port INTEGER NOT NULL DEFAULT 8080,
    status TEXT NOT NULL DEFAULT 'active',
    last_heartbeat TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS sticky_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    instance_id UUID NOT NULL REFERENCES server_instances(id),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_sticky_sessions_user_id ON sticky_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sticky_sessions_instance_id ON sticky_sessions(instance_id);
CREATE INDEX IF NOT EXISTS idx_sticky_sessions_expires_at ON sticky_sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_server_instances_status ON server_instances(status);
CREATE INDEX IF NOT EXISTS idx_server_instances_last_heartbeat ON server_instances(last_heartbeat);
