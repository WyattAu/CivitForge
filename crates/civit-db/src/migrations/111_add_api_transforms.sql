CREATE TABLE IF NOT EXISTS api_transforms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    route TEXT NOT NULL,
    request_transform JSONB,
    response_transform JSONB,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_api_transforms_route ON api_transforms(route);
CREATE INDEX IF NOT EXISTS idx_api_transforms_enabled ON api_transforms(enabled);
