CREATE TABLE IF NOT EXISTS cache_eviction_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    policy_type TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS cache_eviction_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_id UUID NOT NULL REFERENCES cache_eviction_policies(id) ON DELETE CASCADE,
    cache_key TEXT NOT NULL,
    eviction_reason TEXT NOT NULL,
    evicted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_eviction_policies_repo ON cache_eviction_policies(repo_id);
CREATE INDEX idx_eviction_logs_policy ON cache_eviction_logs(policy_id);
