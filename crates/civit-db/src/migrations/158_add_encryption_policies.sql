CREATE TABLE IF NOT EXISTS encryption_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    data_types TEXT[] NOT NULL DEFAULT '{}',
    algorithm TEXT NOT NULL DEFAULT 'AES-256-GCM',
    key_rotation_days INTEGER NOT NULL DEFAULT 90,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_encryption_policies_name ON encryption_policies(name);
CREATE INDEX IF NOT EXISTS idx_encryption_policies_enabled ON encryption_policies(enabled);
