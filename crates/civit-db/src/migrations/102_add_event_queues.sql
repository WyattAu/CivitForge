-- CivitForge: Event Queue System
-- Migration 102

CREATE TABLE IF NOT EXISTS event_queues (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    queue_name TEXT NOT NULL UNIQUE,
    message_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS event_queue_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    queue_id UUID NOT NULL REFERENCES event_queues(id) ON DELETE CASCADE,
    payload JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ
);

CREATE INDEX idx_event_queue_messages_queue_id ON event_queue_messages(queue_id);
CREATE INDEX idx_event_queue_messages_status ON event_queue_messages(status) WHERE status IN ('pending', 'retrying');
CREATE INDEX idx_event_queue_messages_created_at ON event_queue_messages(created_at);