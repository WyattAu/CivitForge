-- Connection pooling configuration
-- Migration 090

CREATE TABLE IF NOT EXISTS pool_config (
    id INTEGER PRIMARY KEY DEFAULT 1,
    min_connections INTEGER NOT NULL DEFAULT 5,
    max_connections INTEGER NOT NULL DEFAULT 20,
    connect_timeout_secs INTEGER NOT NULL DEFAULT 30,
    idle_timeout_secs INTEGER NOT NULL DEFAULT 600,
    max_lifetime_secs INTEGER NOT NULL DEFAULT 1800
);

INSERT INTO pool_config (id, min_connections, max_connections, connect_timeout_secs, idle_timeout_secs, max_lifetime_secs)
VALUES (1, 5, 20, 30, 600, 1800)
ON CONFLICT (id) DO NOTHING;
