# mTLS Hardening Specification

**Document ID:** DD-MTLS-001
**Status:** Proposed
**Target Version:** v2.4.0
**Author:** Autonomous Engineering

---

## 1. Overview

Enforce mutual TLS (mTLS) for all inter-service communication within the
CivitForge deployment. This prevents man-in-the-middle attacks and ensures
that only authenticated services can communicate with each other.

## 2. Scope

| Communication | Current | Target |
|---|---|---|
| Client → civit-core (HTTPS) | TLS (server cert) | TLS (server cert) + optional client cert |
| civit-core → PostgreSQL | Plaintext | mTLS required |
| civit-core → Redis | Plaintext/auth | mTLS required |
| civit-core → civit-runner | HTTP | mTLS required |
| civit-core → civit-vfs (gRPC) | Plaintext | mTLS required |
| civit-core → civit-brain (inference) | HTTP | mTLS required |
| Federation (instance → instance) | HTTP Signatures | HTTP Signatures + mTLS |

## 3. Certificate Authority

### 3.1 Internal CA

A private CA (using `rcgen` or `step-ca`) issues certificates for:

- `civit-core.internal` (server)
- `civit-runner.internal` (client + server)
- `civit-vfs.internal` (server)
- `civit-brain.internal` (server)
- `postgres.internal` (server, requires client cert)
- `redis.internal` (server, requires client cert)

### 3.2 Certificate Properties

```
Validity: 90 days (automatic rotation)
Key type: ECDSA P-256 (ES256)
Key usage: digitalSignature, keyEncipherment
Extended key usage: serverAuth, clientAuth
SAN: service.internal (DNS name)
```

## 4. Implementation

### 4.1 Configuration

```env
# mTLS configuration
MTLS_ENABLED=true
MTLS_CA_CERT_PATH=/etc/civit/certs/ca.pem
MTLS_SERVER_CERT_PATH=/etc/civit/certs/server.pem
MTLS_SERVER_KEY_PATH=/etc/civit/certs/server.key
MTLS_CLIENT_CERT_PATH=/etc/civit/certs/client.pem
MTLS_CLIENT_KEY_PATH=/etc/civit/certs/client.key
MTLS_VERIFY_DEPTH=2
```

### 4.2 PostgreSQL mTLS

```env
DATABASE_URL=postgres://civit@postgres:5432/civit?sslmode=verify-full&sslrootcert=/etc/civit/certs/ca.pem&sslcert=/etc/civit/certs/client.pem&sslkey=/etc/civit/certs/client.key
```

PostgreSQL `pg_hba.conf`:
```
hostssl civit civit all cert map=civit-map
```

### 4.3 Axum Server (civit-core)

```rust
pub async fn serve_mtls(
    app: Router,
    config: &AppConfig,
) -> Result<()> {
    if !config.mtls_enabled {
        return serve_plain(app, config).await;
    }

    let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(
        &config.mtls_server_cert_path,
        &config.mtls_server_key_path,
    )
    .await?;

    // Client certificate verification via mTLS
    let mut root_store = rustls::RootCertStore::empty();
    root_store.add_parsable_certificates(
        &fs::read(&config.mtls_ca_cert_path)?,
    );

    let client_verifier = rustls::server::WebPinnedClientCertVerifier::new(
        Arc::new(root_store),
        config.mtls_verify_depth,
    );

    let tls_config = tls_config.with_client_certificate_verifier(client_verifier);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
```

### 4.4 gRPC Server (civit-vfs)

```rust
let tls_config = ServerTlsConfig::new()
    .identity(Identity::from_pem(
        server_cert,
        server_key,
    ))
    .client_ca_root(Certificate::from_pem(ca_cert));

tonic::transport::Server::builder()
    .tls_config(tls_config)?
    .add_service(vfs_service)
    .serve(addr)
    .await?;
```

## 5. Certificate Rotation

Certificates rotate every 90 days. The rotation flow:

1. **Generate new certificate** (via internal CA or SPIFFE/SPIRE)
2. **Atomic swap**: write new cert/key to temp path, rename atomically
3. **Reload**: civit-core watches cert files with `notify` crate, reloads
   TLS config without dropping connections
4. **Grace period**: old certificate valid for 24h overlap to prevent disruption

## 6. SPIFFE/SPIRE Integration (Optional)

For Kubernetes deployments, integrate with SPIFFE workload identity:

- Each pod gets a SPIFFE ID: `spiffe://civitforge/civit-core`
- SPIRE agent injects SVIDs (X.509 + JWT) into the pod
- Services verify peers via SPIFFE trust domain

## 7. Testing

- **Unit**: Certificate parsing, SAN validation
- **Integration**: mTLS handshake between civit-core and PostgreSQL
- **Security**: Reject expired certs, wrong CA, wrong SAN
- **Chaos**: Rotate cert during active connections, verify no drops
