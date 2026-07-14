-- CivitForge Phase 195: Cache Warming Rules
-- Migration 195
-- Adds cache warming rules and execution logs for pipeline caching.

CREATE TABLE IF NOT EXISTS cache_warming_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    trigger_type TEXT NOT NULL,
    cache_keys TEXT[] NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS cache_warming_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES cache_warming_rules(id) ON DELETE CASCADE,
    cache_keys TEXT[] NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'success',
    duration_ms INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_cache_warming_rules_repo_id ON cache_warming_rules(repo_id);
CREATE INDEX IF NOT EXISTS idx_cache_warming_rules_trigger_type ON cache_warming_rules(trigger_type);
CREATE INDEX IF NOT EXISTS idx_cache_warming_logs_rule_id ON cache_warming_logs(rule_id);
CREATE INDEX IF NOT EXISTS idx_cache_warming_logs_status ON cache_warming_logs(status);
CREATE INDEX IF NOT EXISTS idx_cache_warming_logs_created_at ON cache_warming_logs(created_at DESC);
