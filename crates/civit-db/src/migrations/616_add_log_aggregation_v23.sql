CREATE TABLE IF NOT EXISTS log_index_optimization_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    index_name TEXT NOT NULL,
    table_name TEXT NOT NULL,
    columns TEXT[] NOT NULL,
    query_pattern TEXT,
    improvement_percent DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS log_compression_stats_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    original_bytes BIGINT NOT NULL DEFAULT 0,
    compressed_bytes BIGINT NOT NULL DEFAULT 0,
    compression_ratio DOUBLE PRECISION NOT NULL DEFAULT 0,
    entry_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_log_index_optimization_v20_table_name ON log_index_optimization_v20(table_name);
CREATE INDEX IF NOT EXISTS idx_log_index_optimization_v20_index_name ON log_index_optimization_v20(index_name);
CREATE INDEX IF NOT EXISTS idx_log_compression_stats_v20_period ON log_compression_stats_v20(period_start, period_end);
