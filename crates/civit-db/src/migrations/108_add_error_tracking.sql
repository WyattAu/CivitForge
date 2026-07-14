CREATE TABLE IF NOT EXISTS error_tracking (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    error_type TEXT NOT NULL,
    message TEXT NOT NULL,
    stack_trace TEXT,
    file TEXT,
    line INTEGER,
    user_id UUID REFERENCES users(id),
    count INTEGER NOT NULL DEFAULT 1,
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved BOOLEAN NOT NULL DEFAULT false
);

CREATE INDEX IF NOT EXISTS idx_error_tracking_error_type ON error_tracking(error_type);
CREATE INDEX IF NOT EXISTS idx_error_tracking_user_id ON error_tracking(user_id);
CREATE INDEX IF NOT EXISTS idx_error_tracking_first_seen_at ON error_tracking(first_seen_at);
CREATE INDEX IF NOT EXISTS idx_error_tracking_resolved ON error_tracking(resolved);
