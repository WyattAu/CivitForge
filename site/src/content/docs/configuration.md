---
title: Configuration Reference
description: Environment variables, TLS, LDAP, federation, rate limiting, and CORS configuration for CivitForge.
---

## Overview

CivitForge is configured entirely through environment variables. The `AppConfig`
struct in `crates/civit-core/src/config.rs` reads these at startup and validates
them before the server begins accepting connections.

All configuration is immutable after startup. To change a setting, update the
environment variable and restart the service.

## Core Variables

| Variable | Type | Default | Required | Description |
|----------|------|---------|----------|-------------|
| `DATABASE_URL` | string | -- | Yes | PostgreSQL connection string. Format: `postgres://user:pass@host:port/db` |
| `JWT_SECRET` | string | auto-generated | Yes | HMAC secret for JWT signing. Minimum 32 characters (256 bits). If omitted, a random 64-char hex string is generated (non-persistent across restarts). |
| `REDIS_URL` | string | `redis://127.0.0.1:6379` | No | Redis connection string for session cache and pub/sub. |
| `CIVIT_HOST` | string | `127.0.0.1` | No | Bind address for the HTTP API server. Use `0.0.0.0` for all interfaces. |
| `CIVIT_PORT` | u16 | `8080` | No | TCP port for the HTTP API. Valid range: 1-65535. |
| `JWT_EXPIRY_HOURS` | u64 | `24` | No | JWT token lifetime in hours. Set to `0` for immediate expiry (not recommended). |
| `CIVIT_STORAGE_PATH` | string | `/var/lib/civit/repos` | No | Filesystem path for Git repository storage. |
| `UI_ASSETS_PATH` | string | `./crates/civit-ui/dist` | No | Path to the WASM UI static assets. |
| `CIVIT_DEBUG` | bool | `false` | No | Enable debug mode. Exposes additional endpoints and verbose logging. |

## TLS Configuration

CivitForge supports native TLS termination via `axum-server` with `rustls`.
Both cert and key must be provided to enable TLS; if only one is set, TLS is
disabled and a warning is logged.

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `TLS_CERT_PATH` | string | -- | Path to the PEM-encoded TLS certificate chain. |
| `TLS_KEY_PATH` | string | -- | Path to the PEM-encoded private key. |

When TLS is enabled, the server listens on the same `CIVIT_PORT` with HTTPS.
HTTP traffic is not redirected; place a reverse proxy (nginx, Caddy) in front
for HTTP-to-HTTPS redirect.

### Example

```bash
TLS_CERT_PATH=/etc/certs/server.crt
TLS_KEY_PATH=/etc/certs/server.key
```

## CORS Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `CORS_ALLOWED_ORIGINS` | string | -- | Comma-separated list of allowed origins. Empty = no CORS headers. |

```bash
CORS_ALLOWED_ORIGINS=https://app.example.com,https://admin.example.com
```

The CORS middleware uses `tower-http::cors::CorsLayer`. Preflight requests
(`OPTIONS`) are handled automatically. Credentials are allowed when the origin
is in the allowed list.

## Rate Limiting

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `RATE_LIMIT_MAX_REQUESTS` | u32 | -- | Maximum requests per window per IP. |
| `RATE_LIMIT_WINDOW_SECS` | u32 | -- | Window duration in seconds. |

Rate limiting is enforced via `tower-http` middleware. When both variables are
set, each client IP is limited to `RATE_LIMIT_MAX_REQUESTS` requests within
`RATE_LIMIT_WINDOW_SECS` seconds. Exceeding the limit returns `429 Too Many
Requests`.

When either variable is unset, rate limiting is disabled.

```bash
RATE_LIMIT_MAX_REQUESTS=100
RATE_LIMIT_WINDOW_SECS=60
```

## Security Configuration

Password and login policies are configured via environment variables. All
default values enforce a strong baseline.

### Password Policy

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `PASSWORD_MIN_LENGTH` | usize | `8` | Minimum password length. |
| `PASSWORD_MAX_LENGTH` | usize | `128` | Maximum password length. |
| `PASSWORD_REQUIRE_UPPERCASE` | bool | `true` | Require at least one uppercase letter. |
| `PASSWORD_REQUIRE_LOWERCASE` | bool | `true` | Require at least one lowercase letter. |
| `PASSWORD_REQUIRE_DIGIT` | bool | `true` | Require at least one digit. |
| `PASSWORD_REQUIRE_SPECIAL` | bool | `true` | Require at least one special character. |

### Login Lockout

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `LOGIN_MAX_ATTEMPTS` | u32 | `5` | Maximum failed login attempts before lockout. |
| `LOGIN_LOCKOUT_SECS` | i64 | `900` | Lockout duration in seconds (15 minutes). |

## LDAP Configuration

CivitForge supports LDAP authentication as an alternative or supplement to
local accounts. When `LDAP_ENABLED=true`, users can authenticate against an
LDAP directory. Local accounts continue to work alongside LDAP.

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `LDAP_ENABLED` | bool | `false` | Enable LDAP authentication. |
| `LDAP_URL` | string | `ldap://localhost:389` | LDAP server URL. Use `ldaps://` for TLS. |
| `LDAP_BIND_DN` | string | -- | DN for the bind account. |
| `LDAP_BIND_PASSWORD` | string | -- | Password for the bind account. |
| `LDAP_USER_SEARCH_BASE` | string | `ou=users` | Base DN for user searches. |
| `LDAP_USER_FILTER` | string | `(uid={})` | Filter for user lookup. `{}` is replaced with the username. |
| `LDAP_GROUP_SEARCH_BASE` | string | `ou=groups` | Base DN for group searches. |
| `LDAP_GROUP_FILTER` | string | `(memberUid={})` | Filter for group lookup. `{}` is replaced with the username. |
| `LDAP_MAX_CONNECTIONS` | usize | `10` | Maximum connections in the LDAP connection pool. |
| `LDAP_TLS_CA_PATH` | string | -- | Path to CA certificate for LDAPS. |
| `LDAP_CONNECTION_TIMEOUT_SECS` | u64 | `10` | Connection timeout in seconds. |
| `LDAP_IDLE_TIMEOUT_SECS` | u64 | `300` | Idle connection timeout in seconds. |

### LDAP Authentication Flow

1. Client submits username/password to `/api/v1/auth/login`.
2. Server attempts LDAP bind with `LDAP_BIND_DN` + `LDAP_BIND_PASSWORD`.
3. Server searches for the user using `LDAP_USER_SEARCH_BASE` + `LDAP_USER_FILTER`.
4. Server attempts a second bind with the user's DN and submitted password.
5. On success, a local user record is created/updated and a JWT is issued.

## Federation Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `FEDERATION_ENABLED` | bool | `false` | Enable ActivityPub/ForgeFed federation. |
| `FEDERATION_INSTANCE_ID` | string | `default-instance` | Unique identifier for this instance. Required when federation is enabled. |
| `FEDERATION_INSTANCE_DOMAIN` | string | `localhost` | Public domain name. Required when federation is enabled. |

When federation is enabled, both `FEDERATION_INSTANCE_ID` and
`FEDERATION_INSTANCE_DOMAIN` must be non-empty. The instance ID must be
globally unique across the federation network.

```bash
FEDERATION_ENABLED=true
FEDERATION_INSTANCE_ID=my-forge-01
FEDERATION_INSTANCE_DOMAIN=forge.example.com
```

## Logging

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `RUST_LOG` | string | -- | Standard `tracing-subscriber` env-filter. |

CivitForge uses the `tracing` ecosystem. The `RUST_LOG` variable controls
log verbosity per crate.

```bash
RUST_LOG=civit_core=info,civit_ci=debug,tower_http=trace
```

Supported levels: `trace`, `debug`, `info`, `warn`, `error`.

## Validation Rules

The `AppConfig::validate()` method enforces these constraints at startup:

- `host` must not be empty.
- `port` must not be 0.
- `database_url` must not be empty.
- `jwt_secret` must be at least 32 characters.
- `storage_path` must not be empty.
- When `federation_enabled` is true, `federation_instance_id` and
  `federation_instance_domain` must be non-empty.

Validation failure causes the server to exit with an error message. Check logs
for the specific constraint that failed.

## Docker Compose Example

The `docker-compose.yml` at the repository root demonstrates a complete
configuration:

```yaml
environment:
  DATABASE_URL: postgres://civit:civit-dev-secure-pw-2026@postgres:5432/civit
  JWT_SECRET: change-me-change-me-dev-secret
  CIVIT_HOST: "0.0.0.0"
  CIVIT_PORT: "8080"
  REDIS_URL: redis://:civit-redis-dev-2026@redis:6379
  JWT_EXPIRY_HOURS: "24"
  CIVIT_STORAGE_PATH: /var/lib/civit/repos
  FEDERATION_ENABLED: "false"
  FEDERATION_INSTANCE_ID: "local-dev"
  FEDERATION_INSTANCE_DOMAIN: "localhost"
  RUST_LOG: civit_core=info,tower_http=debug
```
