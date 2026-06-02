-- Migration 005: OIDC Identities, WebAuthn Credentials, Device Management, Refresh Tokens
-- Phase 1.5: Production Authentication

-- OIDC external identity links
CREATE TABLE IF NOT EXISTS oidc_identities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(100) NOT NULL,
    subject VARCHAR(512) NOT NULL,
    email VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider, subject)
);

-- WebAuthn passkey credentials
CREATE TABLE IF NOT EXISTS webauthn_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL DEFAULT '',
    credential_id BYTEA NOT NULL UNIQUE,
    public_key_bytes BYTEA NOT NULL,
    sign_count BIGINT NOT NULL DEFAULT 0,
    transports TEXT[] NOT NULL DEFAULT '{}',
    aaguid BYTEA NOT NULL DEFAULT '\x00',
    backed_up BOOLEAN NOT NULL DEFAULT false,
    device_type VARCHAR(50) NOT NULL DEFAULT 'single_device',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ
);

-- Device tracking for session revocation
CREATE TABLE IF NOT EXISTS devices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL DEFAULT '',
    device_id VARCHAR(255) NOT NULL,
    ip VARCHAR(45),
    user_agent TEXT,
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, device_id)
);

-- Refresh tokens (OIDC/SAML token rotation)
CREATE TABLE IF NOT EXISTS refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(128) NOT NULL UNIQUE,
    scope VARCHAR(512) NOT NULL DEFAULT '',
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ
);

-- Indexes
CREATE INDEX idx_oidc_user ON oidc_identities(user_id);
CREATE INDEX idx_oidc_provider ON oidc_identities(provider, subject);
CREATE INDEX idx_webauthn_user ON webauthn_credentials(user_id);
CREATE INDEX idx_webauthn_cred_id ON webauthn_credentials(credential_id);
CREATE INDEX idx_devices_user ON devices(user_id);
CREATE INDEX idx_devices_last_active ON devices(last_active_at);
CREATE INDEX idx_refresh_user ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_hash ON refresh_tokens(token_hash);
CREATE INDEX idx_refresh_expires ON refresh_tokens(expires_at);
