CREATE TABLE IF NOT EXISTS encryption_key_access_control_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_id UUID NOT NULL REFERENCES encryption_keys(id) ON DELETE CASCADE,
    principal_type TEXT NOT NULL,
    principal_id UUID NOT NULL,
    permission TEXT NOT NULL DEFAULT 'use',
    granted_by UUID NOT NULL REFERENCES users(id),
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    UNIQUE(key_id, principal_type, principal_id)
);

CREATE TABLE IF NOT EXISTS encryption_audit_log_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_id UUID NOT NULL REFERENCES encryption_keys(id),
    operation TEXT NOT NULL,
    principal_id UUID NOT NULL REFERENCES users(id),
    success BOOLEAN NOT NULL DEFAULT true,
    ip_address INET,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_encryption_key_access_control_v21_key ON encryption_key_access_control_v21(key_id);
CREATE INDEX IF NOT EXISTS idx_encryption_key_access_control_v21_principal ON encryption_key_access_control_v21(principal_type, principal_id);
CREATE INDEX IF NOT EXISTS idx_encryption_audit_log_v21_key ON encryption_audit_log_v21(key_id);
CREATE INDEX IF NOT EXISTS idx_encryption_audit_log_v21_principal ON encryption_audit_log_v21(principal_id);
CREATE INDEX IF NOT EXISTS idx_encryption_audit_log_v21_created ON encryption_audit_log_v21(created_at);
