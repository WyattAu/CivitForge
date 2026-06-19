# WebAuthn Implementation Specification

**Document ID:** DD-WEBAUTHN-001
**Status:** Proposed
**Target Version:** v2.4.0
**Author:** Autonomous Engineering

---

## 1. Overview

Add FIDO2/WebAuthn authentication as a passwordless login option and a
two-factor authentication (2FA) method.

## 2. Supported Algorithms

| Algorithm | COSE ID | Use Case |
|---|---|---|
| ES-256 (ECDSA P-256 + SHA-256) | -7 | Primary (recommended) |
| RS-256 (RSASSA-PKCS1-v1.5 + SHA-256) | -257 | Compatibility fallback |

## 3. Registration Flow

```
Client                          Server                    Authenticator
  |                                |                          |
  |  POST /auth/webauthn/register  |                          |
  |  (challenge request)           |                          |
  |------------------------------->|                          |
  |                                |                          |
  |  challenge, user info, RP ID  |                          |
  |<-------------------------------|                          |
  |                                |                          |
  |  navigator.credentials.create()                          |
  |--------------------------------------------------------->|
  |                                |                          |
  |  attestation (public key + sig)                           |
  |<---------------------------------------------------------|
  |                                |                          |
  |  POST /auth/webauthn/register/verify                      |
  |  (attestation response)        |                          |
  |------------------------------->|                          |
  |                                |                          |
  |                                |  verify attestation      |
  |                                |  store credential        |
  |                                |                          |
  |  registration success + JWT   |                          |
  |<-------------------------------|                          |
```

## 4. Database Schema

```sql
CREATE TABLE webauthn_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    credential_id BYTEA NOT NULL UNIQUE,    -- authenticator credential ID
    public_key BYTEA NOT NULL,              -- COSE public key
    counter BIGINT NOT NULL DEFAULT 0,      -- signature counter (clone detection)
    transports TEXT[],                      -- usb, nfc, ble, internal
    attestation_format TEXT,                -- none, packed, tpm, etc.
    name TEXT,                              -- user-assigned name (e.g. "YubiKey")
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ
);

CREATE INDEX idx_webauthn_user ON webauthn_credentials(user_id);
CREATE UNIQUE INDEX idx_webauthn_cred_id ON webauthn_credentials(credential_id);
```

## 5. API Endpoints

### POST /api/v1/auth/webauthn/register/begin
Returns: `{ challenge, rp_id, user_id, user_name, supported_algorithms }`

### POST /api/v1/auth/webauthn/register/complete
Body: `{ credential_id, public_key, attestation, client_data_json }`
Returns: `{ success: true }`

### POST /api/v1/auth/webauthn/login/begin
Body: `{ username }`
Returns: `{ challenge, allowed_credentials }`

### POST /api/v1/auth/webauthn/login/complete
Body: `{ credential_id, signature, authenticator_data, client_data_json }`
Returns: `{ token, user }` (same as password login)

## 6. Implementation

Uses the `webauthn-rs` crate (pure Rust, no C FFI):

```rust
use webauthn_rs::{Webauthn, WebauthnBuilder};
use webauthn_rs::protocol::{RegistrationState, AuthenticationState};

pub struct WebAuthnService {
    webauthn: Webauthn,
    db: DbRepository,
}

impl WebAuthnService {
    pub fn new(rp_id: &str, origin: &str, db: DbRepository) -> Result<Self> {
        let builder = WebauthnBuilder::new(rp_id, &Url::parse(origin)?)?;
        let webauthn = builder.build()?;
        Ok(Self { webauthn, db })
    }

    pub async fn start_registration(&self, user: &User) -> Result<(CreationChallengeResponse, RegistrationState)> {
        let (challenge, state) = self.webauthn.start_passkey_registration(
            Uuid::new_v4(),
            &user.username,
            &user.display_name.clone().unwrap_or_default(),
            None,
        )?;
        // Store state in session for verification step
        Ok((challenge, state))
    }

    pub async fn complete_registration(&self, user: &User, response: RegisterPublicKeyCredential, state: RegistrationState) -> Result<()> {
        let credential = self.webauthn.finish_passkey_registration(&response, &state)?;
        self.db.store_webauthn_credential(user.id, &credential).await?;
        Ok(())
    }
}
```

## 7. Configuration

```env
# WebAuthn Relying Party configuration
WEBAUTHN rp_id=civitforge.example.com
WEBAUTHN_ORIGIN=https://civitforge.example.com
WEBAUTHN_REQUIRE_VERIFICATION=true
```
