CREATE TABLE IF NOT EXISTS realtime_channels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    channel_name TEXT NOT NULL UNIQUE,
    subscriber_count INTEGER NOT NULL DEFAULT 0,
    last_message_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS realtime_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    channel_id UUID NOT NULL REFERENCES realtime_channels(id) ON DELETE CASCADE,
    payload JSONB NOT NULL,
    sender_id UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_realtime_channels_name ON realtime_channels(channel_name);
CREATE INDEX IF NOT EXISTS idx_realtime_messages_channel_id ON realtime_messages(channel_id);
CREATE INDEX IF NOT EXISTS idx_realtime_messages_created_at ON realtime_messages(created_at);
