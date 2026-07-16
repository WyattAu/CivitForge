CREATE TABLE IF NOT EXISTS cache_prediction_model_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cache_key_pattern TEXT NOT NULL,
    predicted_hit_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    predicted_size_bytes BIGINT NOT NULL DEFAULT 0,
    predicted_ttl_seconds INTEGER NOT NULL DEFAULT 3600,
    confidence DOUBLE PRECISION NOT NULL DEFAULT 0,
    last_trained_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS cache_warming_strategies_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    strategy_type TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    hit_rate_improvement DOUBLE PRECISION NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_cache_prediction_model_v20_pattern ON cache_prediction_model_v20(cache_key_pattern);
CREATE INDEX IF NOT EXISTS idx_cache_prediction_model_v20_confidence ON cache_prediction_model_v20(confidence);
CREATE INDEX IF NOT EXISTS idx_cache_warming_strategies_v20_type ON cache_warming_strategies_v20(strategy_type);
CREATE INDEX IF NOT EXISTS idx_cache_warming_strategies_v20_enabled ON cache_warming_strategies_v20(enabled);
