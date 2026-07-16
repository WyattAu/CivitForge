-- Pipeline Caches v23: Cache hit analysis v18
CREATE TABLE IF NOT EXISTS cache_hit_analysis_v18 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cache_id UUID NOT NULL REFERENCES pipeline_caches_v2(id),
    period_start TIMESTAMPTZ NOT NULL,
    hit_count INTEGER NOT NULL DEFAULT 0,
    miss_count INTEGER NOT NULL DEFAULT 0,
    avg_hit_size_bytes BIGINT NOT NULL DEFAULT 0,
    total_size_bytes BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_cache_hit_analysis_v18_cache ON cache_hit_analysis_v18(cache_id);
CREATE INDEX IF NOT EXISTS idx_cache_hit_analysis_v18_period ON cache_hit_analysis_v18(period_start DESC);
CREATE INDEX IF NOT EXISTS idx_cache_hit_analysis_v18_cache_period ON cache_hit_analysis_v18(cache_id, period_start DESC);
