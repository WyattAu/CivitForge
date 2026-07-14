-- CivitForge Phase 140: Encryption at Rest
-- Migration 140
-- Adds tables for encryption key management and encrypted data storage.

CREATE TABLE IF NOT EXISTS encryption_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    algorithm TEXT NOT NULL DEFAULT 'AES-256-GCM',
    key_material BYTEA NOT NULL,
    rotation_date TIMESTAMPTZ,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_encryption_keys_name ON encryption_keys(name);
CREATE INDEX IF NOT EXISTS idx_encryption_keys_enabled ON encryption_keys(enabled);

CREATE TABLE IF NOT EXISTS encrypted_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_id UUID NOT NULL REFERENCES encryption_keys(id),
    data_type TEXT NOT NULL,
    data_id UUID NOT NULL,
    encrypted_data BYTEA NOT NULL,
    iv BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_encrypted_data_key_id ON encrypted_data(key_id);
CREATE INDEX IF NOT EXISTS idx_encrypted_data_data_type ON encrypted_data(data_type);
CREATE INDEX IF NOT EXISTS idx_encrypted_data_data_id ON encrypted_data(data_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_encrypted_data_unique ON encrypted_data(data_type, data_id);
