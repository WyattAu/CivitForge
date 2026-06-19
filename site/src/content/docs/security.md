---
title: Security
description: STRIDE threat model, mTLS, WebAuthn, RBAC, ABAC, and audit trails for CivitForge.
---

## Overview

CivitForge implements defense-in-depth security across authentication,
authorization, cryptography, and auditing. This document covers the threat
model, security controls, and recommended hardening practices.

## STRIDE Threat Model

### Spoofing

| Threat | Control |
|--------|---------|
| User impersonation | JWT tokens with RS256/HS256 signing, bcrypt password hashing |
| Session hijacking | Short-lived tokens, Redis session storage, token rotation |
| LDAP credential theft | TLS-encrypted LDAP connections, bind DN validation |
| Federation impersonation | HTTP Signatures (RFC 9421), key verification via WebFinger |

### Tampering

| Threat | Control |
|--------|---------|
| Request manipulation | TLS transport, input validation on all endpoints |
| Git history rewriting | Branch protection rules, signed commits |
| Webhook payload tampering | HMAC-SHA256 signature verification |
| Pipeline YAML tampering | Git commit SHA pinning, hash verification |

### Repudiation

| Threat | Control |
|--------|---------|
| Action denial | Audit events table with actor, action, resource, IP, user agent |
| Log tampering | Append-only audit_events table, structured logging |
| Federation message denial | Signed activities with timestamp and sequence |

### Information Disclosure

| Threat | Control |
|--------|---------|
| Credential exposure | Tokens stored as bcrypt hashes, secrets encrypted at rest |
| Transport interception | TLS for all connections, mTLS for inter-service |
| Debug information leak | `CIVIT_DEBUG=false` in production, no stack traces in responses |
| Federation data leak | Repository visibility checks before activity delivery |

### Denial of Service

| Threat | Control |
|--------|---------|
| Brute force | Login lockout after N attempts, configurable lockout duration |
| Rate limiting | Per-IP rate limiting with configurable window |
| Resource exhaustion | Connection pool limits, pipeline timeouts, upload size limits |
| Federation flooding | Rate limiting on federation endpoints, inbox deduplication |

### Elevation of Privilege

| Threat | Control |
|--------|---------|
| Horizontal privilege | Repository-level authorization checks |
| Vertical privilege | RBAC roles (guest, user, maintainer, admin, superadmin) |
| Pipeline escape | Rootless Podman containers, Kubernetes SecurityContext |
| Federation spoofing | Actor verification, signature validation |

## Authentication

### JWT tokens

CivitForge uses JSON Web Tokens for API authentication.

**Signing algorithms:**
- HS256 (default): HMAC-SHA256 with `JWT_SECRET`
- RS256 (optional): RSA-SHA256 with keypair

**Token structure:**

```json
{
  "sub": "user-id",
  "username": "alice",
  "role": "user",
  "iat": 1750368000,
  "exp": 1750454400
}
```

**Configuration:**

| Variable | Default | Description |
|----------|---------|-------------|
| `JWT_SECRET` | auto-generated | Signing key (minimum 32 chars) |
| `JWT_EXPIRY_HOURS` | 24 | Token lifetime |

### Password hashing

Passwords are hashed with bcrypt (cost factor 12):

```
$2b$12$LJ3m4ys3Lk1TSwMCPMGB/.uP1S0Wz6uYk2VxKfGm5dH7eR3tY6iO
```

**Password policy enforcement:**

| Setting | Default | Description |
|---------|---------|-------------|
| Minimum length | 8 | Characters |
| Maximum length | 128 | Characters |
| Require uppercase | true | A-Z |
| Require lowercase | true | a-z |
| Require digit | true | 0-9 |
| Require special | true | !@#$%^&*() etc. |

### Login lockout

After `LOGIN_MAX_ATTEMPTS` (default 5) consecutive failed logins, the
account is locked for `LOGIN_LOCKOUT_SECS` (default 900 seconds / 15 minutes).

```
POST /api/v1/auth/login
HTTP/1.1 429 Too Many Requests
Retry-After: 900
```

### Personal Access Tokens (PATs)

PATs are long-lived tokens for API access:

- Stored as bcrypt hashes in `access_tokens` table
- Scoped: `read`, `write`, `admin`
- Configurable expiration
- Rotation supported
- `last_used_at` tracking

### SSH key authentication

SSH keys are stored and validated for Git SSH access:

- Ed25519 and RSA keys supported
- Keys stored in the `ssh_keys` table
- Validated against `ssh-keygen` format
- Rate-limited per IP

### LDAP authentication

When `LDAP_ENABLED=true`:

1. Bind with service account (`LDAP_BIND_DN` / `LDAP_BIND_PASSWORD`)
2. Search for user (`LDAP_USER_SEARCH_BASE` + `LDAP_USER_FILTER`)
3. Bind with user credentials to verify password
4. Create/update local user record
5. Issue JWT

LDAP connections use a pool (`LDAP_MAX_CONNECTIONS`, default 10) with
idle timeout (`LDAP_IDLE_TIMEOUT_SECS`, default 300).

### OpenID Connect

OIDC is supported via migration 053 (`add_oidc.sql`):

- JWKS endpoint for key discovery
- RS256 signature verification via `ring`
- Configurable issuer and client ID
- Admin-only OIDC configuration (migration 056)

### WebAuthn

WebAuthn support via migration 058:

- CBOR parsing and structure validation
- Registration and authentication flows
- Software key fallback via `civit-crypto` HSM module

## Authorization

### RBAC (Role-Based Access Control)

CivitForge implements a role hierarchy:

| Role | Permissions |
|------|-------------|
| `guest` | Read public repositories, view public profiles |
| `user` | Create repositories, open issues, create PRs |
| `maintainer` | Manage repositories, merge PRs, manage pipelines |
| `admin` | Manage organizations, manage users, site settings |
| `superadmin` | Full system access, federation management |

Role checks are enforced in middleware before route handlers execute.

### ABAC (Attribute-Based Access Control)

For fine-grained control, CivitForge implements a CAS-style policy engine
in `civit-crypto/src/policy.rs`:

```json
{
  "subject": "user:alice",
  "action": "repo:push",
  "resource": "repo:alice/my-project",
  "condition": {
    "branch": "main",
    "time": "2026-06-19T00:00:00Z"
  },
  "effect": "allow"
}
```

Policy evaluation uses the CEL (Common Expression Language) evaluator
in `civit-crypto/src/cel/`:

- Arithmetic operators
- 15 built-in functions
- Parenthesized sub-expressions
- Variable binding

### Repository-level permissions

| Action | Owner | Maintainer | Collaborator | Public |
|--------|-------|-----------|-------------|--------|
| Read | Yes | Yes | Yes | If public |
| Push | Yes | Yes | Yes | No |
| Create issue | Yes | Yes | Yes | If public |
| Merge PR | Yes | Yes | No | No |
| Delete repo | Yes | No | No | No |
| Manage webhooks | Yes | Yes | No | No |
| Manage secrets | Yes | No | No | No |

### Organization-level permissions

| Action | Owner | Admin | Member |
|--------|-------|-------|--------|
| Create repo | Yes | Yes | No |
| Manage members | Yes | Yes | No |
| Manage teams | Yes | Yes | No |
| Delete org | Yes | No | No |

## Transport Security

### TLS

CivitForge supports TLS termination via `axum-server` with `rustls`:

```bash
TLS_CERT_PATH=/etc/certs/server.crt
TLS_KEY_PATH=/etc/certs/server.key
```

Both certificate and key must be provided. When only one is set, TLS
is disabled with a warning.

### mTLS (Mutual TLS)

For inter-service communication, CivitForge supports mTLS:

- **Certificate generation:** `rcgen` crate for X.509 CA and cert issuance
- **Fingerprint verification:** SHA-256 fingerprints via `x509-parser`
- **Configuration:** CA certificate in `civit-crypto/src/mtls.rs`

mTLS is used for:
- gRPC/VFS server authentication
- Federation instance verification
- Runner-to-server communication (optional)

### Certificate management

```rust
// Generate self-signed CA
let ca = rcgen::Certificate::new(ca_params)?;

// Issue server cert
let server_cert = ca.issue_certificate(&server_params)?;

// Verify peer certificate fingerprint
let fingerprint = sha256::digest(&peer_cert.to_der()?);
```

## Audit Trail

### Audit events

All significant actions are logged to the `audit_events` table:

```sql
CREATE TABLE audit_events (
    id BIGSERIAL PRIMARY KEY,
    actor_id UUID NOT NULL,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100) NOT NULL,
    resource_id UUID,
    ip_address VARCHAR(45),
    user_agent TEXT,
    outcome VARCHAR(20) NOT NULL DEFAULT 'success',
    details JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Logged actions

| Category | Actions |
|----------|---------|
| Authentication | login, logout, register, password_change, password_reset |
| Authorization | role_change, permission_grant, permission_revoke |
| Repository | create, delete, archive, fork, visibility_change |
| Issues | create, close, reopen, assign, label_change |
| Pull requests | create, merge, close, review_submit |
| Pipelines | trigger, cancel, retry, secret_access |
| Federation | activity_send, activity_receive, key_verify |
| Administration | user_ban, user_unban, site_settings_change |

### Audit query

```bash
curl -H "Authorization: Bearer <token>" \
  "http://localhost:9091/api/v1/audit?action=login&actor=alice&limit=50"
```

### Log format

Structured JSON logs include:

```json
{
  "timestamp": "2026-06-19T12:00:00Z",
  "level": "info",
  "actor": "alice",
  "action": "repo.push",
  "resource": "alice/my-project",
  "ip": "192.168.1.100",
  "user_agent": "git/2.44.0",
  "outcome": "success"
}
```

## Pipeline Security

### Container isolation

Pipeline steps execute in rootless Podman containers:

- No root access inside containers
- Network isolation between steps
- Volume mounts are read-only by default
- Resource limits (CPU, memory) enforced

### Kubernetes security context

When using the K8s operator:

```yaml
securityContext:
  runAsNonRoot: true
  runAsUser: 1000
  readOnlyRootFilesystem: true
  allowPrivilegeEscalation: false
  capabilities:
    drop:
      - ALL
```

### Secret injection

Secrets are injected as environment variables at runtime:

- Never written to disk
- Never exposed in logs
- Encrypted at rest in the database
- Scoped to specific pipelines

### SLSA provenance

CivitForge supports SLSA (Supply-chain Levels for Software Artifacts)
provenance via `civit-crypto/src/provenance.rs`:

- Signed provenance documents
- PEM codec for key storage
- Verification endpoint

## Secret Scanning

Migration 039 adds secret scanning tables. The system scans for:

- API keys and tokens
- Private keys
- Connection strings
- Hardcoded credentials

## Vulnerability Scanning

The `civit-crypto/src/vuln.rs` module integrates with the OSV API:

- Query known vulnerabilities
- CVSS score classification
- Advisory matching by package version

## Security Auditing

### cargo audit

The CI pipeline runs `cargo audit` to check for known vulnerabilities:

```bash
cargo audit --ignore RUSTSEC-2023-0071
```

Ignored advisories are documented in `.cargo/audit.toml` with rationale.

### Pre-commit hooks

```bash
# Runs: fmt, clippy, test, emoji scan
./.githooks/pre-commit
```

## Hardening Checklist

### Authentication

- [ ] Set `JWT_SECRET` to a random 32+ character string
- [ ] Enable `PASSWORD_REQUIRE_*` for strong passwords
- [ ] Set `LOGIN_MAX_ATTEMPTS` and `LOGIN_LOCKOUT_SECS`
- [ ] Enable LDAP with TLS for enterprise deployments
- [ ] Configure OIDC for SSO integration

### Network

- [ ] Enable TLS with valid certificates
- [ ] Configure `CORS_ALLOWED_ORIGINS` explicitly
- [ ] Set `RATE_LIMIT_MAX_REQUESTS` and `RATE_LIMIT_WINDOW_SECS`
- [ ] Use network policies in Kubernetes
- [ ] Place reverse proxy (nginx/Caddy) in front

### Data

- [ ] Enable PostgreSQL SSL connections (`sslmode=require`)
- [ ] Enable Redis authentication
- [ ] Encrypt backup files
- [ ] Set appropriate file permissions on storage volume
- [ ] Regular backup schedule

### Operations

- [ ] Run containers as non-root
- [ ] Enable audit logging
- [ ] Monitor security advisories
- [ ] Rotate secrets periodically
- [ ] Review access tokens regularly

### Federation

- [ ] Enable federation only when needed
- [ ] Verify remote instance keys
- [ ] Monitor federation delivery queue
- [ ] Set federation rate limits
- [ ] Review federation activities
