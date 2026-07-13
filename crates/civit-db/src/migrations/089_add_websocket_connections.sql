-- WebSocket scaling: connection tracking per instance
-- Migration 089

CREATE TABLE IF NOT EXISTS websocket_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    instance_id UUID NOT NULL REFERENCES server_instances(id),
    channel TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_websocket_connections_user_id ON websocket_connections(user_id);
CREATE INDEX IF NOT EXISTS idx_websocket_connections_instance_id ON websocket_connections(instance_id);
CREATE INDEX IF NOT EXISTS idx_websocket_connections_channel ON websocket_connections(channel);
