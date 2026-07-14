CREATE TABLE IF NOT EXISTS cache_hit_analysis_v4 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cache_id UUID NOT NULL REFERENCES pipeline_caches_v2(id),
    period_start TIMESTAMPTZ NOT NULL,
    hit_count INTEGER NOT NULL DEFAULT 0,
    miss_count INTEGER NOT NULL DEFAULT 0,
    avg_hit_size_bytes BIGINT NOT NULL DEFAULT 0,
    total_size_bytes BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS cache_size_tracking_v4 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cache_id UUID NOT NULL REFERENCES pipeline_caches_v2(id) ON DELETE CASCADE,
    measured_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    size_bytes BIGINT NOT NULL DEFAULT 0,
    item_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS cache_cost_optimization_v4 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cache_id UUID NOT NULL REFERENCES pipeline_caches_v2(id) ON DELETE CASCADE,
    period_start TIMESTAMPTZ NOT NULL,
    estimated_savings_bytes BIGINT NOT NULL DEFAULT 0,
    recommended_actions JSONB NOT NULL DEFAULT '[]',
    applied_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS cache_performance_insights_v4 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cache_id UUID NOT NULL REFERENCES pipeline_caches_v2(id) ON DELETE CASCADE,
    period_start TIMESTAMPTZ NOT NULL,
    hit_rate NUMERIC(5,4) NOT NULL DEFAULT 0,
    avg_hit_latency_ms BIGINT NOT NULL DEFAULT 0,
    avg_miss_latency_ms BIGINT NOT NULL DEFAULT 0,
    eviction_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_cache_hit_analysis_v4_cache_id ON cache_hit_analysis_v4(cache_id);
CREATE INDEX IF NOT EXISTS idx_cache_hit_analysis_v4_period ON cache_hit_analysis_v4(period_start DESC);
CREATE INDEX IF NOT EXISTS idx_cache_size_tracking_v4_cache_id ON cache_size_tracking_v4(cache_id);
CREATE INDEX IF NOT EXISTS idx_cache_size_tracking_v4_measured ON cache_size_tracking_v4(measured_at DESC);
CREATE INDEX IF NOT EXISTS idx_cache_cost_optimization_v4_cache_id ON cache_cost_optimization_v4(cache_id);
CREATE INDEX IF NOT EXISTS idx_cache_performance_insights_v4_cache_id ON cache_performance_insights_v4(cache_id);
CREATE INDEX IF NOT EXISTS idx_cache_performance_insights_v4_period ON cache_performance_insights_v4(period_start DESC);
