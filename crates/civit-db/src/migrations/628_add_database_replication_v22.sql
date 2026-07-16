CREATE TABLE IF NOT EXISTS replication_consistency_checks_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_node TEXT NOT NULL,
    target_node TEXT NOT NULL,
    table_name TEXT NOT NULL,
    check_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    discrepancy_count INTEGER NOT NULL DEFAULT 0,
    checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS replication_failover_history_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_node TEXT NOT NULL,
    target_node TEXT NOT NULL,
    failover_type TEXT NOT NULL,
    reason TEXT NOT NULL,
    duration_ms INTEGER,
    success BOOLEAN NOT NULL DEFAULT true,
    initiated_by UUID REFERENCES users(id),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_replication_consistency_checks_v21_source ON replication_consistency_checks_v21(source_node);
CREATE INDEX IF NOT EXISTS idx_replication_consistency_checks_v21_target ON replication_consistency_checks_v21(target_node);
CREATE INDEX IF NOT EXISTS idx_replication_consistency_checks_v21_table ON replication_consistency_checks_v21(table_name);
CREATE INDEX IF NOT EXISTS idx_replication_consistency_checks_v21_status ON replication_consistency_checks_v21(status);
CREATE INDEX IF NOT EXISTS idx_replication_failover_history_v21_source ON replication_failover_history_v21(source_node);
CREATE INDEX IF NOT EXISTS idx_replication_failover_history_v21_target ON replication_failover_history_v21(target_node);
CREATE INDEX IF NOT EXISTS idx_replication_failover_history_v21_occurred ON replication_failover_history_v21(occurred_at);
