CREATE TABLE IF NOT EXISTS live_collaboration_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_type TEXT NOT NULL,
    resource_id UUID NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id),
    cursor_position JSONB,
    last_active TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_live_collaboration_resource ON live_collaboration_sessions(resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_live_collaboration_user_id ON live_collaboration_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_live_collaboration_last_active ON live_collaboration_sessions(last_active);
