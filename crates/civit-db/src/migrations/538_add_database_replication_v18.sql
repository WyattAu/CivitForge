CREATE TABLE IF NOT EXISTS database_replication_config_v16 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    replica_id UUID NOT NULL REFERENCES database_replicas(id),
    config_key TEXT NOT NULL,
    config_value JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(replica_id, config_key)
);

CREATE TABLE IF NOT EXISTS database_replication_alerts_v16 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    replica_id UUID NOT NULL REFERENCES database_replicas(id),
    alert_type TEXT NOT NULL,
    threshold DOUBLE PRECISION NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_database_replication_config_v16_replica ON database_replication_config_v16(replica_id);
CREATE INDEX IF NOT EXISTS idx_database_replication_config_v16_key ON database_replication_config_v16(config_key);
CREATE INDEX IF NOT EXISTS idx_database_replication_alerts_v16_replica ON database_replication_alerts_v16(replica_id);
CREATE INDEX IF NOT EXISTS idx_database_replication_alerts_v16_type ON database_replication_alerts_v16(alert_type);
CREATE INDEX IF NOT EXISTS idx_database_replication_alerts_v16_enabled ON database_replication_alerts_v16(enabled);
