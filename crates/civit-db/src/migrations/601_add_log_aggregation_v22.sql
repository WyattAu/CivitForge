-- Log aggregation v22: log_retention_policies_v19 and log_archives_v19
CREATE TABLE IF NOT EXISTS log_retention_policies_v19 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service TEXT NOT NULL,
    level TEXT NOT NULL,
    retention_days INTEGER NOT NULL DEFAULT 30,
    archive_after_days INTEGER,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(service, level)
);

CREATE TABLE IF NOT EXISTS log_archives_v19 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service TEXT NOT NULL,
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    entry_count INTEGER NOT NULL DEFAULT 0,
    size_bytes BIGINT NOT NULL DEFAULT 0,
    archive_path TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
