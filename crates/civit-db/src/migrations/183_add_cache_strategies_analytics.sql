-- CivitForge Phase 183: Cache Strategies & Analytics
-- Migration 183
-- Adds cache strategy management, analytics, optimization, and cost analysis.

CREATE TABLE IF NOT EXISTS cache_strategies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    strategy_type TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS cache_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cache_id UUID NOT NULL REFERENCES pipeline_caches_v2(id),
    hit_count INTEGER NOT NULL DEFAULT 0,
    miss_count INTEGER NOT NULL DEFAULT 0,
    size_bytes BIGINT NOT NULL DEFAULT 0,
    last_accessed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_cache_strategies_repo_id ON cache_strategies(repo_id);
CREATE INDEX IF NOT EXISTS idx_cache_strategies_type ON cache_strategies(strategy_type);
CREATE INDEX IF NOT EXISTS idx_cache_analytics_cache_id ON cache_analytics(cache_id);
CREATE INDEX IF NOT EXISTS idx_cache_analytics_hit_count ON cache_analytics(hit_count DESC);
CREATE INDEX IF NOT EXISTS idx_cache_analytics_size ON cache_analytics(size_bytes DESC);
