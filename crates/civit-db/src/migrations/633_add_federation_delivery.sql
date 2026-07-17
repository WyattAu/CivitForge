CREATE TABLE IF NOT EXISTS federation_delivery_queue_v1 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_actor_url TEXT NOT NULL,
    target_inbox_url TEXT NOT NULL,
    activity_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 5,
    last_error TEXT,
    next_retry_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    delivered_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS federation_peer_state_v1 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    instance_url TEXT NOT NULL UNIQUE,
    last_seen_at TIMESTAMPTZ,
    delivery_success_count INTEGER NOT NULL DEFAULT 0,
    delivery_failure_count INTEGER NOT NULL DEFAULT 0,
    avg_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
