use crate::error::{AuthError, Result};
use dashmap::DashMap;
use url::Url;
use uuid::Uuid;
use webauthn_rs::prelude::*;

pub use webauthn_rs::prelude::{
    CreationChallengeResponse, CredentialID, Passkey, PasskeyAuthentication, PasskeyRegistration,
    PublicKeyCredential, RegisterPublicKeyCredential, RequestChallengeResponse,
};

pub struct WebAuthnConfig {
    pub relying_party_name: String,
    pub relying_party_id: String,
    pub origin: String,
}

pub struct WebAuthnService {
    webauthn: Webauthn,
    registration_states: DashMap<String, PasskeyRegistration>,
    authentication_states: DashMap<String, PasskeyAuthentication>,
}

impl WebAuthnService {
    pub fn new(config: WebAuthnConfig) -> Result<Self> {
        let origin_url = Url::parse(&config.origin)
            .map_err(|e| AuthError::Config(format!("invalid WebAuthn origin URL: {e}")))?;
        let wb = WebauthnBuilder::new(&config.relying_party_id, &origin_url)
            .map_err(|e| AuthError::Config(format!("failed to build WebAuthn builder: {e}")))?
            .rp_name(&config.relying_party_name)
            .build()
            .map_err(|e| AuthError::Config(format!("failed to initialize WebAuthn: {e}")))?;

        Ok(Self {
            webauthn: wb,
            registration_states: DashMap::new(),
            authentication_states: DashMap::new(),
        })
    }

    pub fn start_registration(
        &self,
        user_id: &str,
        user_name: &str,
        user_display_name: &str,
    ) -> Result<CreationChallengeResponse> {
        let uuid = Uuid::parse_str(user_id)
            .map_err(|e| AuthError::BadRequest(format!("invalid user_id: {e}")))?;
        let (ccr, state) = self
            .webauthn
            .start_passkey_registration(uuid, user_name, user_display_name, None)
            .map_err(|e| {
                AuthError::Internal(format!("failed to generate registration challenge: {e}"))
            })?;

        self.registration_states.insert(user_id.to_string(), state);
        Ok(ccr)
    }

    pub fn finish_registration(
        &self,
        user_id: &str,
        credential: RegisterPublicKeyCredential,
    ) -> Result<Passkey> {
        let (_key, state) = self
            .registration_states
            .remove(user_id)
            .ok_or_else(|| AuthError::BadRequest("no pending registration for this user".into()))?;

        let passkey = self
            .webauthn
            .finish_passkey_registration(&credential, &state)
            .map_err(|e| AuthError::BadRequest(format!("registration failed: {e}")))?;

        Ok(passkey)
    }

    pub fn start_authentication(
        &self,
        user_id: &str,
        passkeys: Vec<Passkey>,
    ) -> Result<RequestChallengeResponse> {
        let (rcr, state) = self
            .webauthn
            .start_passkey_authentication(&passkeys)
            .map_err(|e| AuthError::Internal(format!("failed to generate auth challenge: {e}")))?;

        self.authentication_states
            .insert(user_id.to_string(), state);
        Ok(rcr)
    }

    pub fn finish_authentication(
        &self,
        user_id: &str,
        credential: PublicKeyCredential,
    ) -> Result<()> {
        let (_key, state) = self.authentication_states.remove(user_id).ok_or_else(|| {
            AuthError::BadRequest("no pending authentication for this user".into())
        })?;

        self.webauthn
            .finish_passkey_authentication(&credential, &state)
            .map_err(|e| AuthError::Auth(format!("authentication failed: {e}")))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> WebAuthnConfig {
        WebAuthnConfig {
            relying_party_name: "CivitForge Test".into(),
            relying_party_id: "localhost".into(),
            origin: "http://localhost:8080".into(),
        }
    }

    #[test]
    fn test_webauthn_service_creation() {
        let service = WebAuthnService::new(test_config());
        assert!(service.is_ok());
    }

    #[test]
    fn test_webauthn_service_invalid_origin() {
        let config = WebAuthnConfig {
            relying_party_name: "CivitForge".into(),
            relying_party_id: "localhost".into(),
            origin: "not-a-url".into(),
        };
        let result = WebAuthnService::new(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_start_registration() {
        let service = WebAuthnService::new(test_config()).unwrap();
        let result = service.start_registration(
            "550e8400-e29b-41d4-a716-446655440000",
            "alice",
            "Alice Smith",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_start_registration_invalid_uuid() {
        let service = WebAuthnService::new(test_config()).unwrap();
        let result = service.start_registration("not-a-uuid", "alice", "Alice Smith");
        assert!(result.is_err());
    }

    #[test]
    fn test_start_authentication() {
        let service = WebAuthnService::new(test_config()).unwrap();
        let result = service.start_authentication("user-1", vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_finish_registration_no_pending_state() {
        let service = WebAuthnService::new(test_config()).unwrap();
        let cred_json = r#"{
            "id": "dGVzdA==",
            "rawId": "dGVzdA==",
            "response": {
                "attestationObject": "o2NmbXRlcGFja2VkQ0FEQg",
                "clientDataJSON": "eyJ0eXBlIjogIndlYmF1dGhuLmNyZWF0ZSIsImNoYWxsZW5nZSI6ICIifQ",
                "transports": [],
                "authenticatorExtensionResults": null
            },
            "type": "public-key",
            "getClientExtensionResults": {}
        }"#;
        let cred: RegisterPublicKeyCredential = serde_json::from_str(cred_json).unwrap();
        let result = service.finish_registration("nonexistent-user", cred);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("no pending registration"));
    }

    #[test]
    fn test_finish_authentication_no_pending_state() {
        let service = WebAuthnService::new(test_config()).unwrap();
        let cred_json = r#"{
            "id": "dGVzdA==",
            "rawId": "dGVzdA==",
            "response": {
                "authenticatorData": "dGVzdA==",
                "clientDataJSON": "eyJ0eXBlIjogIndlYmF1dGhuLmF1dGhuZXRrZXkiLCJjaGFsbGVuZ2UiOiAiIn0",
                "signature": "dGVzdA==",
                "userHandle": null
            },
            "type": "public-key",
            "getClientExtensionResults": {}
        }"#;
        let cred: PublicKeyCredential = serde_json::from_str(cred_json).unwrap();
        let result = service.finish_authentication("nonexistent-user", cred);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("no pending authentication"));
    }
}
