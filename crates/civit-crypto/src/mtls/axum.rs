#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower::{Layer, Service};
use tracing::{info, warn};

use super::config::MtlsConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCertInfo {
    pub subject: String,
    pub issuer: String,
    pub serial: String,
    pub fingerprint_sha256: String,
    pub not_after: String,
}

#[derive(Debug, Clone)]
pub struct MtlsLayer {
    require_client_cert: bool,
    client_cert_store: Arc<RwLock<Vec<ClientCertInfo>>>,
}

impl MtlsLayer {
    pub fn new(require_client_cert: bool) -> Self {
        Self {
            require_client_cert,
            client_cert_store: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_client_cert_store(
        require_client_cert: bool,
        store: Arc<RwLock<Vec<ClientCertInfo>>>,
    ) -> Self {
        Self {
            require_client_cert,
            client_cert_store: store,
        }
    }
}

impl<S> Layer<S> for MtlsLayer {
    type Service = MtlsService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MtlsService {
            inner,
            require_client_cert: self.require_client_cert,
            client_cert_store: self.client_cert_store.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MtlsService<S> {
    inner: S,
    require_client_cert: bool,
    client_cert_store: Arc<RwLock<Vec<ClientCertInfo>>>,
}

impl<S, ReqBody> Service<axum::http::Request<ReqBody>> for MtlsService<S>
where
    S: Service<axum::http::Request<ReqBody>, Response = axum::http::Response<axum::body::Body>>
        + Send
        + Clone
        + 'static,
    S::Future: Send,
    S::Error: Send + 'static,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: axum::http::Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        let require = self.require_client_cert;
        let _store = self.client_cert_store.clone();

        Box::pin(async move {
            if require {
                let has_client_cert = req.extensions().get::<ClientCertInfo>().is_some();

                if !has_client_cert {
                    warn!("request rejected: client certificate required but not provided");
                    let response = axum::http::Response::builder()
                        .status(axum::http::StatusCode::UNAUTHORIZED)
                        .body(axum::body::Body::from("client certificate required"))
                        .unwrap();
                    return Ok(response);
                }
            }

            inner.call(req).await
        })
    }
}

pub struct ClientCert(pub ClientCertInfo);

#[derive(Debug)]
pub struct MissingClientCert;

impl std::fmt::Display for MissingClientCert {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "client certificate not provided")
    }
}

impl std::error::Error for MissingClientCert {}

impl axum::response::IntoResponse for MissingClientCert {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        axum::http::Response::builder()
            .status(axum::http::StatusCode::UNAUTHORIZED)
            .body(axum::body::Body::from(self.to_string()))
            .unwrap()
    }
}

impl<S> FromRequestParts<S> for ClientCert
where
    S: Send + Sync,
{
    type Rejection = MissingClientCert;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        match parts.extensions.get::<ClientCertInfo>() {
            Some(info) => Ok(ClientCert(info.clone())),
            None => Err(MissingClientCert),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MtlsServerConfig {
    ca_cert_path: PathBuf,
    server_cert_path: PathBuf,
    server_key_path: PathBuf,
    client_ca_path: Option<PathBuf>,
    require_client_cert: bool,
}

impl MtlsServerConfig {
    pub fn new() -> Self {
        Self {
            ca_cert_path: PathBuf::new(),
            server_cert_path: PathBuf::new(),
            server_key_path: PathBuf::new(),
            client_ca_path: None,
            require_client_cert: true,
        }
    }

    pub fn from_mtls_config(config: &MtlsConfig) -> Self {
        Self {
            ca_cert_path: config.ca_cert_path.clone(),
            server_cert_path: config.server_cert_path.clone(),
            server_key_path: config.server_key_path.clone(),
            client_ca_path: config.client_ca_path.clone(),
            require_client_cert: config.client_verification_enabled(),
        }
    }

    pub fn ca_cert_path(mut self, path: PathBuf) -> Self {
        self.ca_cert_path = path;
        self
    }

    pub fn server_cert_path(mut self, path: PathBuf) -> Self {
        self.server_cert_path = path;
        self
    }

    pub fn server_key_path(mut self, path: PathBuf) -> Self {
        self.server_key_path = path;
        self
    }

    pub fn client_ca_path(mut self, path: PathBuf) -> Self {
        self.client_ca_path = Some(path);
        self
    }

    pub fn require_client_cert(mut self, require: bool) -> Self {
        self.require_client_cert = require;
        self
    }

    pub fn build(self) -> Arc<ServerTlsConfig> {
        Arc::new(ServerTlsConfig {
            ca_cert_path: self.ca_cert_path,
            server_cert_path: self.server_cert_path,
            server_key_path: self.server_key_path,
            client_ca_path: self.client_ca_path,
            require_client_cert: self.require_client_cert,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerTlsConfig {
    pub ca_cert_path: PathBuf,
    pub server_cert_path: PathBuf,
    pub server_key_path: PathBuf,
    pub client_ca_path: Option<PathBuf>,
    pub require_client_cert: bool,
}

impl ServerTlsConfig {
    pub fn load_rustls_config(
        &self,
    ) -> Result<rustls::ServerConfig, Box<dyn std::error::Error + Send + Sync>> {
        let server_cert = std::fs::read_to_string(&self.server_cert_path)?;
        let server_key = std::fs::read_to_string(&self.server_key_path)?;

        let server_cert_chain =
            rustls_pemfile::certs(&mut server_cert.as_bytes()).collect::<Result<Vec<_>, _>>()?;
        let server_key = rustls_pemfile::private_key(&mut server_key.as_bytes())?
            .ok_or("no private key found in server key file")?;

        let mut config = rustls::ServerConfig::builder().with_no_client_auth();

        if self.require_client_cert {
            if let Some(ref client_ca_path) = self.client_ca_path {
                let client_ca_cert = std::fs::read_to_string(client_ca_path)?;
                let client_ca_certs = rustls_pemfile::certs(&mut client_ca_cert.as_bytes())
                    .collect::<Result<Vec<_>, _>>()?;

                let mut root_store = rustls::RootCertStore::empty();
                for cert in client_ca_certs {
                    root_store.add(cert)?;
                }

                let verifier =
                    rustls::server::WebPkiClientVerifier::builder(Arc::new(root_store)).build()?;
                config = rustls::ServerConfig::builder().with_client_cert_verifier(verifier);
            }
        }

        let config = config.with_single_cert(server_cert_chain, server_key)?;

        info!("rustls server configuration loaded for mTLS");
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_test_cert_files(dir: &std::path::Path) -> (PathBuf, PathBuf, PathBuf) {
        let cert_path = dir.join("server.pem");
        let key_path = dir.join("server-key.pem");
        let ca_path = dir.join("ca.pem");

        let mut cert_file = std::fs::File::create(&cert_path).unwrap();
        writeln!(
            cert_file,
            "-----BEGIN CERTIFICATE-----\nMIIBkTCB+wIJAKHGf...\n-----END CERTIFICATE-----"
        )
        .unwrap();

        let mut key_file = std::fs::File::create(&key_path).unwrap();
        writeln!(
            key_file,
            "-----BEGIN EC PRIVATE KEY-----\nMHQCAQEE...\n-----END EC PRIVATE KEY-----"
        )
        .unwrap();

        let mut ca_file = std::fs::File::create(&ca_path).unwrap();
        writeln!(
            ca_file,
            "-----BEGIN CERTIFICATE-----\nMIIBkTCB+wIJAKHGf...\n-----END CERTIFICATE-----"
        )
        .unwrap();

        (cert_path, key_path, ca_path)
    }

    #[test]
    fn test_mtls_server_config_builder() {
        let dir = tempfile::tempdir().unwrap();
        let (cert, key, ca) = create_test_cert_files(dir.path());

        let config = MtlsServerConfig::new()
            .ca_cert_path(ca)
            .server_cert_path(cert.clone())
            .server_key_path(key.clone())
            .require_client_cert(false)
            .build();

        assert_eq!(config.server_cert_path, cert);
        assert_eq!(config.server_key_path, key);
        assert!(!config.require_client_cert);
    }

    #[test]
    fn test_mtls_server_config_from_mtls_config() {
        let dir = tempfile::tempdir().unwrap();
        let (cert, key, ca) = create_test_cert_files(dir.path());
        let client_ca = dir.path().join("client-ca.pem");
        std::fs::write(
            &client_ca,
            "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----",
        )
        .unwrap();

        let mtls_config = MtlsConfig {
            ca_cert_path: ca,
            ca_key_path: dir.path().join("ca-key.pem"),
            server_cert_path: cert,
            server_key_path: key,
            client_ca_path: Some(client_ca.clone()),
        };

        let server_config = MtlsServerConfig::from_mtls_config(&mtls_config).build();
        assert!(server_config.require_client_cert);
        assert_eq!(server_config.client_ca_path, Some(client_ca));
    }

    #[tokio::test]
    async fn test_mtls_layer_rejects_without_client_cert() {
        let layer = MtlsLayer::new(true);

        let inner_service = tower::service_fn(|_req: axum::http::Request<()>| async {
            Ok::<_, std::convert::Infallible>(
                axum::http::Response::builder()
                    .body(axum::body::Body::from("ok"))
                    .unwrap(),
            )
        });

        let mut svc = layer.layer(inner_service);

        let req = axum::http::Request::builder().body(()).unwrap();

        let response = svc.call(req).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_mtls_layer_passes_with_client_cert() {
        let layer = MtlsLayer::new(true);

        let inner_service = tower::service_fn(|_req: axum::http::Request<()>| async {
            Ok::<_, std::convert::Infallible>(
                axum::http::Response::builder()
                    .body(axum::body::Body::from("ok"))
                    .unwrap(),
            )
        });

        let mut svc = layer.layer(inner_service);

        let mut req = axum::http::Request::builder().body(()).unwrap();

        req.extensions_mut().insert(ClientCertInfo {
            subject: "CN=test".into(),
            issuer: "CN=CA".into(),
            serial: "001".into(),
            fingerprint_sha256: "abc123".into(),
            not_after: "2030-01-01".into(),
        });

        let response = svc.call(req).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_mtls_layer_optional_client_cert() {
        let layer = MtlsLayer::new(false);

        let inner_service = tower::service_fn(|_req: axum::http::Request<()>| async {
            Ok::<_, std::convert::Infallible>(
                axum::http::Response::builder()
                    .body(axum::body::Body::from("ok"))
                    .unwrap(),
            )
        });

        let mut svc = layer.layer(inner_service);

        let req = axum::http::Request::builder().body(()).unwrap();

        let response = svc.call(req).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[test]
    fn test_client_cert_store_sharing() {
        let store = Arc::new(RwLock::new(Vec::new()));
        let layer1 = MtlsLayer::with_client_cert_store(true, store.clone());
        let layer2 = MtlsLayer::with_client_cert_store(true, store.clone());

        assert!(std::ptr::eq(
            &*layer1.client_cert_store as *const _,
            &*layer2.client_cert_store as *const _
        ));
    }
}
