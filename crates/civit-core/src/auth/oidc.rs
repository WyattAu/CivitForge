#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::error::Result;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwksKey {
    pub kty: String,
    pub kid: Option<String>,
    pub alg: Option<String>,
    pub n: String,
    pub e: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwksKeySet {
    pub keys: Vec<JwksKey>,
}

struct CachedJwks {
    keys: HashMap<String, JwksKey>,
    fallback_keys: Vec<JwksKey>,
    fetched_at: Instant,
}

pub struct OidcService {
    pub config: OidcConfig,
    pub discovery: Option<OidcDiscovery>,
    pub http_client: reqwest::Client,
    jwks_cache: Arc<RwLock<Option<CachedJwks>>>,
}

const JWKS_CACHE_TTL: Duration = Duration::from_secs(300);

impl OidcService {
    pub fn new(config: OidcConfig) -> Self {
        Self {
            config,
            discovery: None,
            http_client: reqwest::Client::new(),
            jwks_cache: Arc::new(RwLock::new(None)),
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

    async fn fetch_jwks(&self) -> Result<JwksKeySet> {
        let discovery = self
            .discovery
            .as_ref()
            .ok_or_else(|| crate::error::CoreError::Auth("OIDC discovery not completed".into()))?;
        let url = discovery.jwks_uri.clone();
        let key_set = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| crate::error::CoreError::Auth(format!("JWKS fetch request failed: {e}")))?
            .error_for_status()
            .map_err(|e| crate::error::CoreError::Auth(format!("JWKS fetch response error: {e}")))?
            .json::<JwksKeySet>()
            .await
            .map_err(|e| crate::error::CoreError::Auth(format!("JWKS parse failed: {e}")))?;
        info!(keys_count = key_set.keys.len(), "JWKS fetched");
        Ok(key_set)
    }

    async fn get_jwks_key(&self, kid: &str) -> Result<JwksKey> {
        let cache_read = self.jwks_cache.read().await;
        let needs_refresh = match cache_read.as_ref() {
            Some(cached) => cached.fetched_at.elapsed() >= JWKS_CACHE_TTL,
            None => true,
        };
        drop(cache_read);

        if needs_refresh {
            let key_set = self.fetch_jwks().await?;
            let mut by_kid: HashMap<String, JwksKey> = HashMap::new();
            let mut fallbacks: Vec<JwksKey> = Vec::new();
            for key in key_set.keys {
                match &key.kid {
                    Some(k) => {
                        by_kid.insert(k.clone(), key);
                    }
                    None => fallbacks.push(key),
                }
            }
            let mut cache_write = self.jwks_cache.write().await;
            *cache_write = Some(CachedJwks {
                keys: by_kid,
                fallback_keys: fallbacks,
                fetched_at: Instant::now(),
            });
        }

        let cache = self.jwks_cache.read().await;
        let cached = cache.as_ref().ok_or_else(|| {
            crate::error::CoreError::Internal("JWKS cache missing after fetch".into())
        })?;
        if let Some(key) = cached.keys.get(kid) {
            return Ok(key.clone());
        }
        if let Some(key) = cached.fallback_keys.first() {
            return Ok(key.clone());
        }
        Err(crate::error::CoreError::Auth(format!(
            "No matching JWKS key found for kid={kid}"
        )))
    }

    pub async fn validate_id_token(&self, token: &str, nonce: Option<&str>) -> Result<OidcClaims> {
        let header = decode_header(token)
            .map_err(|e| crate::error::CoreError::Auth(format!("Invalid id_token header: {e}")))?;

        if header.alg != Algorithm::RS256 {
            return Err(crate::error::CoreError::Auth(format!(
                "Unsupported id_token algorithm: {:?}",
                header.alg
            )));
        }

        let kid = header.kid.as_deref().unwrap_or("");
        let jwks_key = self.get_jwks_key(kid).await.map_err(|e| {
            crate::error::CoreError::Auth(format!("Failed to retrieve JWKS key for kid={kid}: {e}"))
        })?;

        if jwks_key.kty != "RSA" {
            return Err(crate::error::CoreError::Auth(format!(
                "JWKS key kty is not RSA: {}",
                jwks_key.kty
            )));
        }

        let decoding_key =
            DecodingKey::from_rsa_components(&jwks_key.n, &jwks_key.e).map_err(|e| {
                crate::error::CoreError::Auth(format!("Invalid RSA key components: {e}"))
            })?;

        let mut validation = Validation::new(Algorithm::RS256);
        if let Some(issuer) = self.discovery.as_ref().map(|d| d.issuer.as_str()) {
            validation.set_issuer(&[issuer]);
        }
        validation.set_audience(&[&self.config.client_id]);

        let data = decode::<OidcClaims>(token, &decoding_key, &validation).map_err(|e| {
            crate::error::CoreError::Auth(format!("id_token verification failed: {e}"))
        })?;

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
        info!(sub = %claims.sub, "id_token validated (kid: {:?})", header.kid);
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

    const TEST_RSA_N: &str = "jrb3qSnKgoEWJHNy3OtUejaiIa3TtnZTQmNAxGV7BFMskRDpnwV_fYCiGniL22CfVcAajJUQsUZHgb5oH1zBcxnxOZq6C9FV9QaWMbQgB22Vth0IAkjfRs1ZACDo2UfVJU27eE-r3q_Z_GkxtUUz1uotxAvN1vW_G-pCzqYAwi1hOCe56d1fzOHjZ67rsM8851uQcRNtNMJdZhdkfSXeqCDLMhJJNwoU1L3KFQQSu2ixAs9WPfj9eY5g3NLGBdaQrOSWCq-4t9TjviG-iA5ZRQTHkRX7mgVMCy7TEc7w1eYxEdpOB8gVVv2m_XQ0NK5SaR3xcCmS2jHA29YZu4Vscw";
    const TEST_RSA_E: &str = "AQAB";
    const TEST_RSA_PRIVATE_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQCOtvepKcqCgRYk
c3Lc61R6NqIhrdO2dlNCY0DEZXsEUyyREOmfBX99gKIaeIvbYJ9VwBqMlRCxRkeB
vmgfXMFzGfE5mroL0VX1BpYxtCAHbZW2HQgCSN9GzVkAIOjZR9UlTbt4T6ver9n8
aTG1RTPW6i3EC83W9b8b6kLOpgDCLWE4J7np3V/M4eNnruuwzzznW5BxE200wl1m
F2R9Jd6oIMsyEkk3ChTUvcoVBBK7aLECz1Y9+P15jmDc0sYF1pCs5JYKr7i31OO+
Ib6IDllFBMeRFfuaBUwLLtMRzvDV5jER2k4HyBVW/ab9dDQ0rlJpHfFwKZLaMcDb
1hm7hWxzAgMBAAECggEAFkFx7syocDX0P4aiJNHYJd+WOh1S/53ajxGjsdzyKbL/
fLc/+ioP1IELsTLCUK7z5MSlVJi8doDIuZ15ZwapLlYfLbCzjxCRUI7l5jtk+/OQ
HiB54BgAJcOekQFmKS2QD5YHLxRxmjPJ5ph7BX3qFHuxhmShhYJjpVVLdNF4xc8K
fuofsLgKIhxco5L+duGCyTe1nqzKYX0J9dr+tadoZsWtgdFqezRQZDspCN8auOoY
yboVSCxEr1+W8YTC6LEGoWpNP4egRU7RqUbRjAEo/+GMO7IorHTRJZtBK/8tnCaZ
vbWaw4e8RUsM+Y4HYXl/hLYdTL4WZ61ENHsHfAlBDQKBgQDJyoXGl+fnkoNtzU7b
mavLohxYVvmLUGMsjsLQlkz9oZShsuu+3HXvkNklp9vi//I08pPCq7l+fC/SxKGf
HgwMn5XvjD16HDhsTyx7vjk48uGQ67QSvmj9kZq2QVVlva/5l/gW+EuGyGG2E4jd
IKsFd2C4kEyTuwbmpdfEss2i3QKBgQC1DaxFoi3oW6vNjdw03V99+hW2siTG7+NH
/zikkVubFvxWzAn07qdo3rmdZCHBRrU3vVm0llAfnBpvn9ukb++BIZGp4CJrgTqL
WDf7ZoBqjzeYjfgOzsG59+aT7A6UTvESr0HvOfCLebK3SoooXvcbNZrSabVEE+7a
BFEAYYaPjwKBgQCwKtXVhgLYwalqL+Zbg3JfKdzzJqNfg8PBP7VGoyD+AJWhAXIc
w51Wk69v13b5W5eZr/ld58veaA7cQ/wRzQqZ7qzYYCe/tmlv7UMZmP2eATT57zzb
bE5+qSJXYPxsAUljbtARRZ2yQrhDXBSNcJq9//V5y8h+5LXmoPyZZbxvJQKBgHjc
kMTHN8gl8fE9IuPzZ3ysRnR4RU678sgsGr7Y/gLw/DBg8sCb1AuQqu3jWxkvv2df
MpP3x7LiPU+IslH6GzLjmt6A7dlAIjnFAVIEofMAegePtikEYpRnZXgXm7/rVsi3
T9eHoQkqi2AKFWJPyrtSNHED+ephOBA3027iq7YHAoGAcSRGk1Pm74Vcamh5of5O
dMfZU3VG/mEVrFEVQKD2W0TF3NDvB8YuD8arb8vEXmc35te3Gs5C1rv/62d0lm39
OgURb1zgy0BtvzsYDuGg9wmy9xkArxyDQXD5eq7B9OvpKPFNIp8XeevFFZ0QFvCd
E034x0CZQuzH4z+EZNJm3/E=
-----END PRIVATE KEY-----";

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

    fn make_service_with_jwks_cache(keys: Vec<JwksKey>) -> OidcService {
        let mut svc = make_service();
        let mut by_kid: HashMap<String, JwksKey> = HashMap::new();
        let mut fallbacks: Vec<JwksKey> = Vec::new();
        for key in keys {
            match &key.kid {
                Some(k) => {
                    by_kid.insert(k.clone(), key);
                }
                None => fallbacks.push(key),
            }
        }
        svc.jwks_cache = Arc::new(tokio::sync::RwLock::new(Some(CachedJwks {
            keys: by_kid,
            fallback_keys: fallbacks,
            fetched_at: Instant::now(),
        })));
        svc
    }

    fn test_encoding_key() -> jsonwebtoken::EncodingKey {
        jsonwebtoken::EncodingKey::from_rsa_pem(TEST_RSA_PRIVATE_PEM.as_bytes()).unwrap()
    }

    fn sign_test_token(kid: Option<&str>, claims: &serde_json::Value) -> String {
        let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = kid.map(|s| s.to_string());
        jsonwebtoken::encode(&header, claims, &test_encoding_key()).unwrap()
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

    #[test]
    fn test_jwks_parsing_from_json() {
        let json_str = r#"{
            "keys": [
                {
                    "kty": "RSA",
                    "kid": "key-1",
                    "alg": "RS256",
                    "n": "n_value_base64url",
                    "e": "AQAB"
                },
                {
                    "kty": "RSA",
                    "kid": "key-2",
                    "alg": "RS256",
                    "n": "another_n_value",
                    "e": "AQAB"
                }
            ]
        }"#;
        let key_set: JwksKeySet = serde_json::from_str(json_str).unwrap();
        assert_eq!(key_set.keys.len(), 2);
        assert_eq!(key_set.keys[0].kty, "RSA");
        assert_eq!(key_set.keys[0].kid.as_deref(), Some("key-1"));
        assert_eq!(key_set.keys[0].alg.as_deref(), Some("RS256"));
        assert_eq!(key_set.keys[0].n, "n_value_base64url");
        assert_eq!(key_set.keys[0].e, "AQAB");
        assert_eq!(key_set.keys[1].kid.as_deref(), Some("key-2"));
    }

    #[test]
    fn test_jwks_parsing_single_key() {
        let json_str = r#"{
            "keys": [
                {
                    "kty": "RSA",
                    "kid": "solo",
                    "n": "modulus",
                    "e": "AQAB"
                }
            ]
        }"#;
        let key_set: JwksKeySet = serde_json::from_str(json_str).unwrap();
        assert_eq!(key_set.keys.len(), 1);
        assert_eq!(key_set.keys[0].kid.as_deref(), Some("solo"));
    }

    #[test]
    fn test_jwks_parsing_empty_keys() {
        let json_str = r#"{"keys": []}"#;
        let key_set: JwksKeySet = serde_json::from_str(json_str).unwrap();
        assert!(key_set.keys.is_empty());
    }

    #[test]
    fn test_jwks_parsing_key_without_kid() {
        let json_str = r#"{
            "keys": [
                {
                    "kty": "RSA",
                    "n": "some_n",
                    "e": "AQAB"
                }
            ]
        }"#;
        let key_set: JwksKeySet = serde_json::from_str(json_str).unwrap();
        assert!(key_set.keys[0].kid.is_none());
        assert_eq!(key_set.keys[0].n, "some_n");
    }

    #[test]
    fn test_rsa_key_construction_from_components() {
        let result = DecodingKey::from_rsa_components(TEST_RSA_N, TEST_RSA_E);
        assert!(
            result.is_ok(),
            "DecodingKey::from_rsa_components should succeed with test key"
        );
    }

    #[tokio::test]
    async fn test_rs256_signature_verification() {
        let decoding_key = DecodingKey::from_rsa_components(TEST_RSA_N, TEST_RSA_E).unwrap();

        let token = sign_test_token(
            Some("test-kid"),
            &serde_json::json!({
                "sub": "user-123",
                "email": "test@example.com",
                "name": "Test User",
                "preferred_username": null,
                "iss": "https://auth.example.com",
                "aud": "test-client-id",
                "exp": 9999999999u64,
                "iat": 1500000000u64
            }),
        );

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&["https://auth.example.com"]);
        validation.set_audience(&["test-client-id"]);
        let data = decode::<OidcClaims>(&token, &decoding_key, &validation).unwrap();
        assert_eq!(data.claims.sub, "user-123");
        assert_eq!(data.claims.email, "test@example.com");
    }

    #[test]
    fn test_rs256_signature_verification_wrong_key_fails() {
        let wrong_n = "AJCn2f7faas-KvOx0njYc3v_HKGxuveu_dKcGYaqvvoaWt16dG_RRkfZ6c-KxGtGRQlxlniGG_yMLMyO3xBgmgo9tG2tUxEegA9EaZLlCBpxjoSLP7mD0LquNq7BrTn73fWEGw2yasropICspLaroivNyigZhdqew2ho_clrVqfgWlD403BAZhjY_kl24G0MTCh91TL8Z1nN3qLNSU_5nlCFgSsk7cWMYmJ_lHOxmsGwpgHHj4x3vcb2fZoAFs5qwMY-0AAL36YjqHFS";
        let decoding_key = DecodingKey::from_rsa_components(wrong_n, TEST_RSA_E).unwrap();
        let claims = serde_json::json!({
            "sub": "user-123",
            "email": "test@example.com",
            "name": "Test User",
            "preferred_username": null,
            "iss": "https://auth.example.com",
            "aud": "test-client-id",
            "exp": 9999999999u64,
            "iat": 1500000000u64
        });
        let token = sign_test_token(Some("test-kid"), &claims);
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&["https://auth.example.com"]);
        validation.set_audience(&["test-client-id"]);
        let decoded = decode::<OidcClaims>(&token, &decoding_key, &validation);
        assert!(decoded.is_err(), "Verification with wrong key should fail");
    }

    #[tokio::test]
    async fn test_validate_id_token_no_jwks_fails() {
        let svc = make_service();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let claims = serde_json::json!({
            "sub": "user-123",
            "email": "test@example.com",
            "name": "Test User",
            "preferred_username": "testuser",
            "iss": "https://auth.example.com",
            "aud": "test-client-id",
            "exp": now + 3600,
            "iat": now
        });
        let token = sign_test_token(Some("test-kid"), &claims);
        let result = svc.validate_id_token(&token, None).await;
        assert!(
            result.is_err(),
            "Expected error for tampered signature, got ok"
        );
        eprintln!("Error: {:?}", result.unwrap_err());
    }

    #[tokio::test]
    async fn test_validate_id_token_invalid_jwt() {
        let svc = make_service();
        let result = svc.validate_id_token("not-a-jwt", None).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid id_token header")
        );
    }

    #[tokio::test]
    async fn test_validate_id_token_wrong_alg_rejected() {
        let svc = make_service();
        let token = {
            use base64::Engine;
            let header_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD
                .encode(r#"{"alg":"HS256","typ":"JWT"}"#);
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let claims_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD
                .encode(format!(
                    r#"{{"sub":"u1","email":"e@e.com","name":"N","preferred_username":null,"iss":"https://auth.example.com","aud":"test-client-id","exp":{},"iat":{}}}"#,
                    now + 3600, now
                ));
            format!("{header_b64}.{claims_b64}.fakesig")
        };
        let result = svc.validate_id_token(&token, None).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unsupported id_token algorithm")
        );
    }

    #[tokio::test]
    async fn test_validate_id_token_with_valid_jwks_and_sig() {
        let jwks_key = JwksKey {
            kty: "RSA".into(),
            kid: Some("test-key-id".into()),
            alg: Some("RS256".into()),
            n: TEST_RSA_N.to_string(),
            e: TEST_RSA_E.to_string(),
        };
        let svc = make_service_with_jwks_cache(vec![jwks_key]);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let claims = serde_json::json!({
            "sub": "user-verified",
            "email": "verified@example.com",
            "name": "Verified User",
            "preferred_username": "vuser",
            "iss": "https://auth.example.com",
            "aud": "test-client-id",
            "exp": now + 3600,
            "iat": now
        });

        let token = sign_test_token(Some("test-key-id"), &claims);
        let result = svc.validate_id_token(&token, None).await;
        assert!(result.is_ok());
        let oidc_claims = result.unwrap();
        assert_eq!(oidc_claims.sub, "user-verified");
        assert_eq!(oidc_claims.email, "verified@example.com");
    }

    #[tokio::test]
    async fn test_validate_id_token_nonce_check() {
        let jwks_key = JwksKey {
            kty: "RSA".into(),
            kid: Some("key-nonce".into()),
            alg: Some("RS256".into()),
            n: TEST_RSA_N.to_string(),
            e: TEST_RSA_E.to_string(),
        };
        let svc = make_service_with_jwks_cache(vec![jwks_key]);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let claims = serde_json::json!({
            "sub": "user-nonce",
            "email": "nonce@example.com",
            "name": "Nonce User",
            "preferred_username": null,
            "iss": "https://auth.example.com",
            "aud": "test-client-id",
            "exp": now + 3600,
            "iat": now,
            "nonce": "expected-nonce"
        });

        let token = sign_test_token(Some("key-nonce"), &claims);
        let result = svc.validate_id_token(&token, Some("expected-nonce")).await;
        assert!(result.is_ok());

        let result = svc.validate_id_token(&token, Some("wrong-nonce")).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("nonce mismatch"));
    }

    #[tokio::test]
    async fn test_validate_id_token_signature_tamper_fails() {
        let jwks_key = JwksKey {
            kty: "RSA".into(),
            kid: Some("key-tamper".into()),
            alg: Some("RS256".into()),
            n: TEST_RSA_N.to_string(),
            e: TEST_RSA_E.to_string(),
        };
        let svc = make_service_with_jwks_cache(vec![jwks_key]);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let claims = serde_json::json!({
            "sub": "user-tamper",
            "email": "tamper@example.com",
            "name": "Tamper User",
            "preferred_username": null,
            "iss": "https://auth.example.com",
            "aud": "test-client-id",
            "exp": now + 3600,
            "iat": now
        });

        let mut token = sign_test_token(Some("key-tamper"), &claims);
        let last_dot = token.rfind('.').unwrap();
        let fake_sig = {
            use base64::Engine;
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b"totally-invalid-signature")
        };
        token.replace_range(last_dot.., &format!(".{fake_sig}"));

        let result = svc.validate_id_token(&token, None).await;
        match result {
            Ok(_) => panic!("Tampered signature should have been rejected!"),
            Err(e) => {
                let msg = e.to_string();
                assert!(
                    msg.contains("id_token verification failed"),
                    "Unexpected error: {msg}"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_validate_id_token_claims_missing_fields() {
        let svc = make_service_with_jwks_cache(vec![JwksKey {
            kty: "RSA".into(),
            kid: Some("missing-fields-key".into()),
            alg: Some("RS256".into()),
            n: TEST_RSA_N.to_string(),
            e: TEST_RSA_E.to_string(),
        }]);
        let token = sign_test_token(
            Some("missing-fields-key"),
            &serde_json::json!({
                "sub": "user-1"
            }),
        );
        let result = svc.validate_id_token(&token, None).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("id_token verification failed")
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
        let encoded = urlencoding::percent_encode("caf\u{00e9}");
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
