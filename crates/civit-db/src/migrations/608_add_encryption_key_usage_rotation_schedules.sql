CREATE TABLE IF NOT EXISTS encryption_key_usage_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_id UUID NOT NULL REFERENCES encryption_keys(id) ON DELETE CASCADE,
    operation TEXT NOT NULL,
    success BOOLEAN NOT NULL DEFAULT true,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS encryption_key_rotation_schedules_v20 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_id UUID NOT NULL REFERENCES encryption_keys(id) ON DELETE CASCADE,
    rotation_days INTEGER NOT NULL DEFAULT 90,
    last_rotated_at TIMESTAMPTZ,
    next_rotation_at TIMESTAMPTZ,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_encryption_key_usage_v20_key ON encryption_key_usage_v20(key_id);
CREATE INDEX IF NOT EXISTS idx_encryption_key_usage_v20_operation ON encryption_key_usage_v20(operation);
CREATE INDEX IF NOT EXISTS idx_encryption_key_usage_v20_created ON encryption_key_usage_v20(created_at);
CREATE INDEX IF NOT EXISTS idx_encryption_key_rotation_schedules_v20_key ON encryption_key_rotation_schedules_v20(key_id);
CREATE INDEX IF NOT EXISTS idx_encryption_key_rotation_schedules_v20_next ON encryption_key_rotation_schedules_v20(next_rotation_at);
