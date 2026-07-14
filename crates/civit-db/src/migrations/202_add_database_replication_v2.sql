CREATE TABLE IF NOT EXISTS database_replication_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    replica_id UUID NOT NULL REFERENCES database_replicas(id),
    operation TEXT NOT NULL,
    table_name TEXT NOT NULL,
    record_id UUID,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS database_replication_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    replica_id UUID NOT NULL REFERENCES database_replicas(id),
    period_start TIMESTAMPTZ NOT NULL,
    operations_count INTEGER NOT NULL DEFAULT 0,
    avg_lag_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_database_replication_logs_replica_id ON database_replication_logs(replica_id);
CREATE INDEX IF NOT EXISTS idx_database_replication_logs_status ON database_replication_logs(status);
CREATE INDEX IF NOT EXISTS idx_database_replication_logs_created_at ON database_replication_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_database_replication_stats_replica_id ON database_replication_stats(replica_id);
CREATE INDEX IF NOT EXISTS idx_database_replication_stats_period_start ON database_replication_stats(period_start);
