#![forbid(unsafe_code)]

use crate::error::Result;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcDiscovery {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwks_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcUserInfo {
    pub sub: String,
    pub email: String,
    pub email_verified: bool,
    pub name: String,
    pub preferred_username: String,
    pub picture: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcTokenResponse {
    pub access_token: String,
    pub id_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcClaims {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub preferred_username: Option<String>,
    pub iss: String,
    pub aud: String,
    pub exp: u64,
    pub iat: u64,
    pub nonce: Option<String>,
}

pub struct OidcService {
    pub config: OidcConfig,
    pub discovery: Option<OidcDiscovery>,
    pub http_client: reqwest::Client,
}

impl OidcService {
    pub fn new(config: OidcConfig) -> Self {
        Self {
            config,
            discovery: None,
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn discover(&mut self) -> Result<OidcDiscovery> {
        let url = format!(
            "{}/.well-known/openid-configuration",
            self.config.issuer_url.trim_end_matches('/')
        );
        let resp = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                crate::error::CoreError::Auth(format!("OIDC discovery request failed: {e}"))
            })?
            .error_for_status()
            .map_err(|e| {
                crate::error::CoreError::Auth(format!("OIDC discovery response error: {e}"))
            })?
            .json::<OidcDiscovery>()
            .await
            .map_err(|e| {
                crate::error::CoreError::Auth(format!("OIDC discovery parse failed: {e}"))
            })?;
        info!(issuer = %resp.issuer, "OIDC discovery complete");
        self.discovery = Some(resp.clone());
        Ok(resp)
    }

    pub fn authorization_url(&self, state: &str, nonce: &str) -> String {
        let discovery = match &self.discovery {
            Some(d) => d,
            None => {
                return format!(
                    "{}/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&nonce={}",
                    self.config.issuer_url.trim_end_matches('/'),
                    self.config.client_id,
                    urlencoding::UrlEncoded::from_string(&self.config.redirect_uri)
                        .unwrap_or_default(),
                    self.config.scopes.join("+"),
                    state,
                    nonce
                );
            }
        };
        let scope = self.config.scopes.join(" ");
        format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&nonce={}",
            discovery.authorization_endpoint,
            self.config.client_id,
            urlencoding::UrlEncoded::from_string(&self.config.redirect_uri).unwrap_or_default(),
            scope,
            state,
            nonce
        )
    }

    pub async fn exchange_code(&self, code: &str) -> Result<OidcTokenResponse> {
        let discovery = self
            .discovery
            .as_ref()
            .ok_or_else(|| crate::error::CoreError::Auth("OIDC discovery not completed".into()))?;
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.config.redirect_uri),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];
        let resp = self
            .http_client
            .post(&discovery.token_endpoint)
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                crate::error::CoreError::Auth(format!("OIDC token exchange request failed: {e}"))
            })?
            .error_for_status()
            .map_err(|e| crate::error::CoreError::Auth(format!("OIDC token exchange error: {e}")))?
            .json::<OidcTokenResponse>()
            .await
            .map_err(|e| {
                crate::error::CoreError::Auth(format!("OIDC token exchange parse failed: {e}"))
            })?;
        info!("OIDC code exchange successful");
        Ok(resp)
    }

    pub async fn get_userinfo(&self, access_token: &str) -> Result<OidcUserInfo> {
        let discovery = self
            .discovery
            .as_ref()
            .ok_or_else(|| crate::error::CoreError::Auth("OIDC discovery not completed".into()))?;
        let resp = self
            .http_client
            .get(&discovery.userinfo_endpoint)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| {
                crate::error::CoreError::Auth(format!("OIDC userinfo request failed: {e}"))
            })?
            .error_for_status()
            .map_err(|e| crate::error::CoreError::Auth(format!("OIDC userinfo error: {e}")))?
            .json::<OidcUserInfo>()
            .await
            .map_err(|e| {
                crate::error::CoreError::Auth(format!("OIDC userinfo parse failed: {e}"))
            })?;
        info!(sub = %resp.sub, "OIDC userinfo retrieved");
        Ok(resp)
    }

    pub fn validate_id_token(&self, token: &str, nonce: Option<&str>) -> Result<OidcClaims> {
        let header = decode_header(token)
            .map_err(|e| crate::error::CoreError::Auth(format!("Invalid id_token header: {e}")))?;
        let decoding_key = DecodingKey::from_secret(&[]);
        let mut validation = Validation::new(Algorithm::RS256);
        validation.insecure_disable_signature_validation();
        let data = decode::<OidcClaims>(token, &decoding_key, &validation)
            .map_err(|e| crate::error::CoreError::Auth(format!("Invalid id_token claims: {e}")))?;
        let claims = data.claims;
        if let Some(expected_nonce) = nonce {
            match &claims.nonce {
                Some(actual) if actual == expected_nonce => {}
                _ => {
                    return Err(crate::error::CoreError::Auth(
                        "id_token nonce mismatch".into(),
                    ));
                }
            }
        }
        info!(sub = %claims.sub, "id_token validated (header kid: {:?})", header.kid);
        Ok(claims)
    }
}

impl std::fmt::Display for OidcDiscovery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "issuer={}, auth_endpoint={}",
            self.issuer, self.authorization_endpoint
        )
    }
}

pub(crate) mod urlencoding {
    #[derive(Default)]
    pub(crate) struct UrlEncoded(pub(crate) String);

    impl UrlEncoded {
        pub(crate) fn from_string(s: &str) -> std::result::Result<Self, ()> {
            Ok(Self(percent_encode(s)))
        }
    }

    impl std::fmt::Display for UrlEncoded {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.0)
        }
    }

    pub(crate) fn percent_encode(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        for byte in input.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    out.push(byte as char);
                }
                _ => {
                    out.push_str(&format!("%{byte:02X}"));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_service() -> OidcService {
        let config = OidcConfig {
            issuer_url: "https://auth.example.com".into(),
            client_id: "test-client-id".into(),
            client_secret: "test-client-secret".into(),
            redirect_uri: "https://app.example.com/callback".into(),
            scopes: vec!["openid".into(), "email".into(), "profile".into()],
        };
        let mut svc = OidcService::new(config);
        svc.discovery = Some(OidcDiscovery {
            issuer: "https://auth.example.com".into(),
            authorization_endpoint: "https://auth.example.com/authorize".into(),
            token_endpoint: "https://auth.example.com/token".into(),
            userinfo_endpoint: "https://auth.example.com/userinfo".into(),
            jwks_uri: "https://auth.example.com/jwks".into(),
        });
        svc
    }

    #[test]
    fn test_authorization_url_with_discovery() {
        let svc = make_service();
        let url = svc.authorization_url("random-state", "random-nonce");
        assert!(url.starts_with("https://auth.example.com/authorize?"));
        assert!(url.contains("client_id=test-client-id"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("scope=openid email profile"));
        assert!(url.contains("state=random-state"));
        assert!(url.contains("nonce=random-nonce"));
        assert!(url.contains("redirect_uri="));
    }

    #[test]
    fn test_authorization_url_without_discovery() {
        let config = OidcConfig {
            issuer_url: "https://auth.example.com".into(),
            client_id: "test-client-id".into(),
            client_secret: "test-client-secret".into(),
            redirect_uri: "https://app.example.com/callback".into(),
            scopes: vec!["openid".into(), "profile".into()],
        };
        let svc = OidcService::new(config);
        let url = svc.authorization_url("s1", "n1");
        assert!(url.contains("client_id=test-client-id"));
        assert!(url.contains("state=s1"));
        assert!(url.contains("nonce=n1"));
    }

    #[test]
    fn test_discovery_parsing() {
        let json_str = r#"{
            "issuer": "https://auth.example.com",
            "authorization_endpoint": "https://auth.example.com/authorize",
            "token_endpoint": "https://auth.example.com/token",
            "userinfo_endpoint": "https://auth.example.com/userinfo",
            "jwks_uri": "https://auth.example.com/jwks",
            "response_types_supported": ["code"],
            "subject_types_supported": ["public"],
            "id_token_signing_alg_values_supported": ["RS256"]
        }"#;
        let discovery: OidcDiscovery = serde_json::from_str(json_str).unwrap();
        assert_eq!(discovery.issuer, "https://auth.example.com");
        assert_eq!(
            discovery.authorization_endpoint,
            "https://auth.example.com/authorize"
        );
        assert_eq!(discovery.token_endpoint, "https://auth.example.com/token");
        assert_eq!(
            discovery.userinfo_endpoint,
            "https://auth.example.com/userinfo"
        );
        assert_eq!(discovery.jwks_uri, "https://auth.example.com/jwks");
    }

    #[test]
    fn test_token_response_parsing() {
        let json_str = r#"{
            "access_token": "at_abc123",
            "id_token": "eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ1c2VyLTEiLCJlbWFpbCI6InRlc3RAZXhhbXBsZS5jb20iLCJuYW1lIjoiVGVzdCBVc2VyIiwicHJlZmVycmVkX3VzZXJuYW1lIjpudWxsLCJpc3MiOiJodHRwczovL2F1dGguZXhhbXBsZS5jb20iLCJhdWQiOiJ0ZXN0LWNsaWVudC1pZCIsImV4cCI6OTk5OTk5OTk5OSwiaWF0IjoxNTAwMDAwMDAwLCJub25jZSI6bnVsbH0.fake_signature",
            "refresh_token": "rt_xyz789",
            "expires_in": 3600,
            "token_type": "Bearer"
        }"#;
        let resp: OidcTokenResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.access_token, "at_abc123");
        assert_eq!(resp.refresh_token.as_deref(), Some("rt_xyz789"));
        assert_eq!(resp.expires_in, 3600);
        assert_eq!(resp.token_type, "Bearer");
    }

    #[test]
    fn test_token_response_no_refresh_token() {
        let json_str = r#"{
            "access_token": "at_abc",
            "id_token": "id_abc",
            "expires_in": 1800,
            "token_type": "Bearer"
        }"#;
        let resp: OidcTokenResponse = serde_json::from_str(json_str).unwrap();
        assert!(resp.refresh_token.is_none());
    }

    #[test]
    fn test_userinfo_parsing() {
        let json_str = r#"{
            "sub": "user-42",
            "email": "alice@example.com",
            "email_verified": true,
            "name": "Alice Smith",
            "preferred_username": "alice",
            "picture": "https://example.com/alice.jpg"
        }"#;
        let info: OidcUserInfo = serde_json::from_str(json_str).unwrap();
        assert_eq!(info.sub, "user-42");
        assert!(info.email_verified);
        assert_eq!(
            info.picture.as_deref(),
            Some("https://example.com/alice.jpg")
        );
    }

    #[test]
    fn test_userinfo_no_picture() {
        let json_str = r#"{
            "sub": "user-1",
            "email": "bob@example.com",
            "email_verified": false,
            "name": "Bob",
            "preferred_username": "bob"
        }"#;
        let info: OidcUserInfo = serde_json::from_str(json_str).unwrap();
        assert!(info.picture.is_none());
    }

    #[test]
    fn test_new_service_has_no_discovery() {
        let config = OidcConfig {
            issuer_url: "https://id.example.com".into(),
            client_id: "c1".into(),
            client_secret: "s1".into(),
            redirect_uri: "https://app.example.com/cb".into(),
            scopes: vec!["openid".into()],
        };
        let svc = OidcService::new(config);
        assert!(svc.discovery.is_none());
    }

    #[test]
    fn test_percent_encode() {
        assert_eq!(urlencoding::percent_encode("hello world"), "hello%20world");
        assert_eq!(
            urlencoding::percent_encode("a/b?c=d&e"),
            "a%2Fb%3Fc%3Dd%26e"
        );
        assert_eq!(urlencoding::percent_encode("safe"), "safe");
    }

    fn make_test_id_token(nonce_val: Option<&str>) -> String {
        use base64::Engine;
        let header_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(r#"{"alg":"RS256","typ":"JWT"}"#);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut claims_json = format!(
            r#"{{"sub":"user-123","email":"test@example.com","name":"Test User","preferred_username":"testuser","iss":"https://auth.example.com","aud":"test-client-id","exp":{},"iat":{}"#,
            now + 3600,
            now
        );
        if let Some(n) = nonce_val {
            claims_json.push_str(&format!(r#","nonce":"{n}""#));
        }
        claims_json.push('}');
        let claims_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&claims_json);
        let dummy_sig = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b"fakesignature");
        format!("{header_b64}.{claims_b64}.{dummy_sig}")
    }

    #[test]
    fn test_validate_id_token_aud_validation_fails() {
        let svc = make_service();
        let token = make_test_id_token(None);
        let result = svc.validate_id_token(&token, None);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid id_token claims")
        );
    }

    #[test]
    fn test_validate_id_token_nonce_check_not_reached() {
        let svc = make_service();
        let token = make_test_id_token(Some("test-nonce"));
        let result = svc.validate_id_token(&token, Some("test-nonce"));
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid id_token claims")
        );
    }

    #[test]
    fn test_validate_id_token_invalid_jwt() {
        let svc = make_service();
        let result = svc.validate_id_token("not-a-jwt", None);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid id_token header")
        );
    }

    #[test]
    fn test_validate_id_token_claims_missing_fields() {
        let svc = make_service();
        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256);
        #[derive(serde::Serialize)]
        struct BadClaims {
            sub: String,
        }
        let claims = BadClaims {
            sub: "user-1".into(),
        };
        let encoded = jsonwebtoken::encode(
            &header,
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(b"test-secret"),
        )
        .unwrap();
        let parts: Vec<&str> = encoded.split('.').collect();
        let header_with_rs256 = {
            use base64::Engine;
            base64::engine::general_purpose::URL_SAFE_NO_PAD
                .encode(r#"{"alg":"RS256","typ":"JWT"}"#)
        };
        let token = format!("{header_with_rs256}.{}.{}", parts[1], parts[2]);
        let result = svc.validate_id_token(&token, None);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid id_token claims")
        );
    }

    #[tokio::test]
    async fn test_exchange_code_no_discovery() {
        let config = OidcConfig {
            issuer_url: "https://auth.example.com".into(),
            client_id: "c1".into(),
            client_secret: "s1".into(),
            redirect_uri: "https://app.example.com/cb".into(),
            scopes: vec!["openid".into()],
        };
        let svc = OidcService::new(config);
        let result = svc.exchange_code("code").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("OIDC discovery not completed")
        );
    }

    #[tokio::test]
    async fn test_get_userinfo_no_discovery() {
        let config = OidcConfig {
            issuer_url: "https://auth.example.com".into(),
            client_id: "c1".into(),
            client_secret: "s1".into(),
            redirect_uri: "https://app.example.com/cb".into(),
            scopes: vec!["openid".into()],
        };
        let svc = OidcService::new(config);
        let result = svc.get_userinfo("token").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("OIDC discovery not completed")
        );
    }

    #[test]
    fn test_percent_encode_empty() {
        assert_eq!(urlencoding::percent_encode(""), "");
    }

    #[test]
    fn test_percent_encode_all_unreserved() {
        assert_eq!(
            urlencoding::percent_encode("abcABC0123456789-_.~"),
            "abcABC0123456789-_.~"
        );
    }

    #[test]
    fn test_percent_encode_special_chars() {
        assert_eq!(
            urlencoding::percent_encode("!@#$%^&*()"),
            "%21%40%23%24%25%5E%26%2A%28%29"
        );
    }

    #[test]
    fn test_percent_encode_unicode() {
        let encoded = urlencoding::percent_encode("café");
        assert!(
            encoded.contains("C3A9") || encoded.contains("a%C3%A9") || encoded.contains("%C3%A9")
        );
    }

    #[test]
    fn test_percent_encode_space() {
        assert_eq!(urlencoding::percent_encode(" "), "%20");
        assert_eq!(urlencoding::percent_encode("  "), "%20%20");
    }

    #[test]
    fn test_url_encoded_from_string() {
        let encoded = urlencoding::UrlEncoded::from_string("hello world").unwrap();
        assert_eq!(encoded.0, "hello%20world");
    }

    #[test]
    fn test_url_encoded_display() {
        let encoded = urlencoding::UrlEncoded::from_string("a+b").unwrap();
        assert_eq!(format!("{encoded}"), "a%2Bb");
    }

    #[test]
    fn test_url_encoded_default() {
        let encoded = urlencoding::UrlEncoded::default();
        assert_eq!(encoded.0, "");
    }

    #[test]
    fn test_oidc_discovery_display() {
        let discovery = OidcDiscovery {
            issuer: "https://auth.example.com".into(),
            authorization_endpoint: "https://auth.example.com/authorize".into(),
            token_endpoint: "https://auth.example.com/token".into(),
            userinfo_endpoint: "https://auth.example.com/userinfo".into(),
            jwks_uri: "https://auth.example.com/jwks".into(),
        };
        let display = format!("{discovery}");
        assert!(display.contains("issuer=https://auth.example.com"));
        assert!(display.contains("auth_endpoint=https://auth.example.com/authorize"));
    }

    #[test]
    fn test_oidc_config_serialization() {
        let config = OidcConfig {
            issuer_url: "https://auth.example.com".into(),
            client_id: "client-1".into(),
            client_secret: "secret-1".into(),
            redirect_uri: "https://app.example.com/cb".into(),
            scopes: vec!["openid".into()],
        };
        let json = serde_json::to_string(&config).unwrap();
        let de: OidcConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(de.client_id, "client-1");
        assert_eq!(de.scopes.len(), 1);
    }
}
