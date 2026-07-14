CREATE TABLE IF NOT EXISTS database_replicas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    host TEXT NOT NULL,
    port INTEGER NOT NULL DEFAULT 5432,
    status TEXT NOT NULL DEFAULT 'syncing',
    lag_ms INTEGER NOT NULL DEFAULT 0,
    last_sync_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_database_replicas_name ON database_replicas(name);
CREATE INDEX IF NOT EXISTS idx_database_replicas_status ON database_replicas(status);
