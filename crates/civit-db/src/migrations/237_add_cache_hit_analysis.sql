CREATE TABLE IF NOT EXISTS cache_hit_analysis (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cache_id UUID NOT NULL REFERENCES pipeline_caches_v2(id) ON DELETE CASCADE,
    period_start TIMESTAMPTZ NOT NULL,
    hit_count INTEGER NOT NULL DEFAULT 0,
    miss_count INTEGER NOT NULL DEFAULT 0,
    avg_hit_size_bytes BIGINT NOT NULL DEFAULT 0,
    total_size_bytes BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_cache_hit_analysis_cache_id ON cache_hit_analysis(cache_id);
CREATE INDEX IF NOT EXISTS idx_cache_hit_analysis_period ON cache_hit_analysis(period_start DESC);
