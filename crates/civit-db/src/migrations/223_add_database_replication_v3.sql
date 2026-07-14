CREATE TABLE IF NOT EXISTS database_replication_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    replica_id UUID NOT NULL REFERENCES database_replicas(id),
    config_key TEXT NOT NULL,
    config_value JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(replica_id, config_key)
);

CREATE TABLE IF NOT EXISTS database_replication_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    replica_id UUID NOT NULL REFERENCES database_replicas(id),
    alert_type TEXT NOT NULL,
    threshold DOUBLE PRECISION NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_database_replication_config_replica ON database_replication_config(replica_id);
CREATE INDEX IF NOT EXISTS idx_database_replication_config_key ON database_replication_config(config_key);
CREATE INDEX IF NOT EXISTS idx_database_replication_alerts_replica ON database_replication_alerts(replica_id);
CREATE INDEX IF NOT EXISTS idx_database_replication_alerts_type ON database_replication_alerts(alert_type);
CREATE INDEX IF NOT EXISTS idx_database_replication_alerts_enabled ON database_replication_alerts(enabled);