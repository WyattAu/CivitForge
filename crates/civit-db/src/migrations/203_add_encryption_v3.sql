CREATE TABLE IF NOT EXISTS encryption_key_rotations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_id UUID NOT NULL REFERENCES encryption_keys(id),
    old_key_id UUID,
    rotated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reason TEXT NOT NULL DEFAULT 'scheduled'
);

CREATE TABLE IF NOT EXISTS encryption_audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_id UUID NOT NULL REFERENCES encryption_keys(id),
    action TEXT NOT NULL,
    user_id UUID REFERENCES users(id),
    details JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_encryption_key_rotations_key_id ON encryption_key_rotations(key_id);
CREATE INDEX IF NOT EXISTS idx_encryption_key_rotations_rotated_at ON encryption_key_rotations(rotated_at);
CREATE INDEX IF NOT EXISTS idx_encryption_audit_logs_key_id ON encryption_audit_logs(key_id);
CREATE INDEX IF NOT EXISTS idx_encryption_audit_logs_user_id ON encryption_audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_encryption_audit_logs_created_at ON encryption_audit_logs(created_at);
