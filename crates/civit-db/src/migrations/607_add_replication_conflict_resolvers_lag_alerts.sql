CREATE TABLE IF NOT EXISTS replication_conflict_resolvers_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_name TEXT NOT NULL,
    resolver_type TEXT NOT NULL DEFAULT 'last_write_wins',
    custom_logic JSONB,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(table_name)
);

CREATE TABLE IF NOT EXISTS replication_lag_alerts_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_node TEXT NOT NULL,
    target_node TEXT NOT NULL,
    threshold_ms INTEGER NOT NULL DEFAULT 1000,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_replication_conflict_resolvers_v20_table ON replication_conflict_resolvers_v20(table_name);
CREATE INDEX IF NOT EXISTS idx_replication_lag_alerts_v20_source ON replication_lag_alerts_v20(source_node);
CREATE INDEX IF NOT EXISTS idx_replication_lag_alerts_v20_target ON replication_lag_alerts_v20(target_node);
