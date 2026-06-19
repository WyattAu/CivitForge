# Security Audit: WebAuthn and mTLS Implementations

**Scope**: CivitForge monorepo - WebAuthn and mTLS code
**Date**: 2026-06-19
**Auditor**: opencode (automated)
**Overall Risk Level**: HIGH

---

## Executive Summary

The WebAuthn and mTLS implementations contain one critical vulnerability, several high-severity issues, and multiple medium-severity findings. The most severe issue is that `CertificateAuthority::verify_chain` is a **stub** that accepts any non-empty certificate as valid, completely undermining the mTLS chain-of-trust model. The WebAuthn implementation has an unbounded in-memory state store with no TTL, enabling denial-of-service through memory exhaustion. Additionally, multiple API error paths leak internal database and system error details to clients.

---

## Findings

### CRITICAL

#### C1: Certificate Chain Verification is a Stub
- **File**: `crates/civit-crypto/src/mtls/mod.rs:138-144`
- **Description**: `CertificateAuthority::verify_chain` only checks if the certificate PEM string is non-empty. It does not parse the certificate, verify the signature chain against the CA, check validity dates, or perform any cryptographic verification. Every call returns `Ok(true)` for any non-empty input.
- **Impact**: Any self-signed or forged certificate will pass chain verification. An attacker can present a certificate signed by an unrelated CA and it will be accepted as valid. This completely defeats the purpose of the mTLS trust model.
- **Remediation**: Implement actual certificate chain verification using `x509-parser` (already a dependency) or `rustls`. Parse the certificate, verify the signature against the CA public key, check validity period, and validate the chain to the root CA.

#### C2: Rotation State Stuck in `Rotating` on Failure
- **File**: `crates/civit-crypto/src/mtls/rotation.rs:109-115,119-122`
- **Description**: The `rotate` method sets the state to `Rotating` before attempting certificate issuance. If `issue_certificate` fails, the error is returned but the state is never reset to `Active`. All subsequent rotation attempts will fail with `AlreadyRotating`.
- **Impact**: A single certificate issuance failure permanently disables the rotation mechanism until the process is restarted. In a scenario where rotation is triggered by expiry detection, this could lead to expired certificates being used in production.
- **Remediation**: Wrap the issuance in a scope that resets the state to `Active` on error (or use a guard/drop pattern). Alternatively, set state back to the previous state before returning the error.

---

### HIGH

#### H1: Unbounded In-Memory WebAuthn State Store (DoS)
- **File**: `crates/civit-auth/src/webauthn.rs:20-21`
- **Description**: `registration_states` and `authentication_states` are `DashMap` instances with no size limit, no TTL/expiration, and no cleanup mechanism. An attacker can call `start_registration` or `start_authentication` repeatedly with different user IDs to exhaust server memory.
- **Impact**: Denial of service through memory exhaustion. Each entry contains a `PasskeyRegistration` or `PasskeyAuthentication` struct which includes cryptographic challenge data. A sustained attack could consume all available memory.
- **Remediation**: Implement TTL-based expiration (e.g., 5-minute timeout for pending states), enforce maximum number of concurrent pending states per user, and add overall size limits. Use a scheduled cleanup task or wrap entries with timestamps.

#### H2: Database Error Details Leaked to Clients
- **File**: `crates/civit-core/src/api/webauthn.rs:147,206,281,298`
- **Description**: Multiple error paths construct responses with raw database/system error messages: `format!("failed to store credential: {e}")`, `format!("failed to fetch credentials: {e}")`, `format!("failed to fetch user: {e}")`, `format!("failed to generate token: {e}")`. These are returned directly to the client as JSON error responses.
- **Impact**: Information disclosure. Database error messages may contain table names, column names, constraint violations, connection strings, or other internal details that aid an attacker in reconnaissance or crafting targeted attacks.
- **Remediation**: Return generic error messages to clients (e.g., "internal server error") and log the detailed errors server-side. Follow the pattern already established in `CoreError::error_response()` which sanitizes Database, Internal, Io, Config, and Git errors.

#### H3: Client Cert Middleware Does Not Validate Certificates
- **File**: `crates/civit-crypto/src/mtls/axum.rs:91-112`
- **Description**: The `MtlsService` middleware only checks whether a `ClientCertInfo` extension exists in the request. It does not validate the certificate's expiration, revocation status, chain of trust, or any other property. The actual TLS-level verification happens at the transport layer, but this middleware provides no additional validation and does not enforce that the transport layer actually performed verification.
- **Impact**: If the transport layer (reverse proxy, load balancer) strips or does not forward client certificate information, the middleware will silently allow unauthenticated requests when `require_client_cert` is true - but only if the extension is missing. More critically, this creates a false sense of security for developers who rely on the middleware for authorization decisions.
- **Remediation**: Document clearly that the `MtlsLayer` is a secondary check and that actual mTLS verification must happen at the transport/TLS layer. Consider adding certificate property validation (expiration check, issuer verification) as a defense-in-depth measure.

---

### MEDIUM

#### M1: WebAuthn State Overwrite Race
- **File**: `crates/civit-auth/src/webauthn.rs:56,88-89`
- **Description**: Calling `start_registration` or `start_authentication` for the same user_id silently overwrites any existing pending state without cleanup. A legitimate user's pending challenge could be replaced by an attacker who knows or guesses the user_id.
- **Impact**: Authentication session hijacking. If an attacker initiates a registration for a victim's user_id, it overwrites the victim's pending challenge. The victim's subsequent `finish_registration` call would fail (consuming the attacker's state), but the attacker's `finish_registration` could succeed if they have a valid credential.
- **Remediation**: Check for existing pending state before inserting. Return an error or require explicit cancellation of the previous challenge. Alternatively, use the challenge response directly rather than relying on server-side state.

#### M2: Blocking I/O in Async Context
- **File**: `crates/civit-crypto/src/mtls/rotation.rs:180,187`
- **Description**: `persist_log` uses `std::fs::create_dir_all` and `std::fs::write` which are synchronous blocking I/O operations called within an async context. This can block the tokio runtime thread.
- **Impact**: Under load or with slow disk I/O, this can cause latency spikes or stall the entire async runtime, affecting all tasks on that thread.
- **Remediation**: Use `tokio::fs::create_dir_all` and `tokio::fs::write` instead, or wrap synchronous I/O in `tokio::task::spawn_blocking`.

#### M3: Serial Number Collision Risk
- **File**: `crates/civit-crypto/src/mtls/mod.rs:111-117`
- **Description**: Certificate serial numbers are derived from Unix timestamp (`{:016x}` of seconds since epoch). Two certificates issued within the same second (or the CA cert and its first issued cert which uses timestamp and timestamp+1) may have the same or predictable serial numbers.
- **Impact**: X.509 certificate serial numbers must be unique per CA. Collisions can cause issues with certificate revocation lists, OCSP, and some TLS implementations that use serial numbers for session binding.
- **Remediation**: Use a cryptographically secure random number for serial numbers (e.g., 128-bit random value) or a monotonically increasing counter.

#### M4: Unsanitized PEM Validation
- **File**: `crates/civit-crypto/src/mtls/config.rs:113-131`
- **Description**: `validate_pem_file` only checks for the presence of `-----BEGIN ` and `-----END ` strings. Any file containing those markers (including arbitrary text) would pass validation.
- **Impact**: A misconfigured deployment could load non-certificate data as a certificate, leading to TLS handshake failures or, in edge cases, acceptance of malformed certificates.
- **Remediation**: Parse the PEM content using `rustls_pemfile` or `x509-parser` to verify it is actually a valid certificate or key. The dependencies for this are already available.

---

### LOW

#### L1: Information Disclosure - User Credential Status
- **File**: `crates/civit-core/src/api/webauthn.rs:212-218`
- **Description**: The `authenticate_start` endpoint returns `{"error": "no WebAuthn credentials registered"}` with HTTP 404 when a user has no WebAuthn credentials. This confirms to an attacker whether a specific user has WebAuthn enabled.
- **Impact**: Reconnaissance. An attacker can enumerate which users have WebAuthn set up, potentially targeting those with weaker authentication methods.
- **Remediation**: Return a generic error message regardless of whether credentials exist. The difference in response can be logged server-side for debugging.

#### L2: Error State Recovery After Crash
- **File**: `crates/civit-crypto/src/mtls/rotation.rs:52-58`
- **Description**: There is no mechanism to recover the rotation state if the process crashes while in the `Rotating` state. The state is only held in memory.
- **Impact**: After a crash during rotation, the system restarts with `Active` state (default), which is actually safe. However, the rotation log may be incomplete if the crash occurred before `persist_log`.
- **Remediation**: Persist the rotation state to disk alongside the rotation log, or recover state from the log on startup.

#### L3: Silent Error Swallowing in Log Persistence
- **File**: `crates/civit-crypto/src/mtls/rotation.rs:140`
- **Description**: `self.persist_log().await.ok()` silently discards any I/O error from log persistence.
- **Impact**: Rotation events may not be persisted to disk without any indication. Audit trail gaps could hinder forensic analysis after a security incident.
- **Remediation**: Log the error at `warn` or `error` level instead of silently discarding it.

#### L4: Unused Client Certificate Store
- **File**: `crates/civit-crypto/src/mtls/axum.rs:94`
- **Description**: The `client_cert_store` is cloned into the service closure (`_store`) but never read from or written to. The entire `client_cert_store` field is dead code.
- **Impact**: No direct security impact, but the unused code may mislead developers into thinking client certificate information is being tracked or cached.
- **Remediation**: Either implement the store functionality or remove it to reduce confusion.

#### L5: Error Path Reveals WebAuthn Internal Error Messages
- **File**: `crates/civit-auth/src/webauthn.rs:53,86`
- **Description**: `start_registration` and `start_authentication` propagate webauthn-rs internal error messages through `AuthError::Internal`. While the API layer maps these to generic errors via `CoreError::error_response`, the intermediate `AuthError` message is logged with the full detail.
- **Impact**: Low. The API layer correctly sanitizes these. However, if any other code path exposes `AuthError::Internal` directly, the webauthn-rs internal messages would be visible.
- **Remediation**: Consider mapping webauthn-rs errors to more generic messages at the service layer.

---

### INFORMATIONAL

#### I1: No CSRF Protection on WebAuthn Endpoints
- **File**: `crates/civit-core/src/api/webauthn.rs:42-54`
- **Description**: The WebAuthn API endpoints do not implement CSRF tokens. However, since these are JSON API endpoints that require an `Authorization` header (enforced by the `AuthUser` extractor), they are not vulnerable to CSRF via form submissions or image tags. Browsers will not send `Authorization` headers cross-origin without CORS preflight.
- **Impact**: None if CORS is properly configured. Verify that CORS policy restricts origins appropriately.

#### I2: User ID String Comparison for Authorization
- **File**: `crates/civit-core/src/api/webauthn.rs:73,175`
- **Description**: Authorization checks use string comparison (`auth.user_id != req.user_id`). Since these are UUID strings and not security-sensitive tokens, standard string comparison is acceptable here.
- **Impact**: None. UUIDs are not secret and do not require constant-time comparison.

#### I3: Hash Token Comparison Uses Standard Equality
- **File**: `crates/civit-core/src/api/auth.rs:28-32`
- **Description**: Token hashes are compared using standard database lookup (`validate_pat_token`), not direct comparison of stored hashes. Since the hash is used as a lookup key rather than compared against a stored value, timing attacks on the hash comparison itself are not applicable.
- **Impact**: None. The attack surface is the database lookup, which is outside the scope of this audit.

---

## OWASP Top 10 Mapping

| OWASP Category | Finding(s) | Severity |
|---|---|---|
| A01:2021 - Broken Access Control | C1 (chain verification stub), H3 (middleware does not validate certs) | Critical/High |
| A02:2021 - Cryptographic Failures | C1 (no chain verification), M3 (serial number collision) | Critical/Medium |
| A04:2021 - Insecure Design | H1 (unbounded state store), M1 (state overwrite race) | High/Medium |
| A05:2021 - Security Misconfiguration | M4 (weak PEM validation) | Medium |
| A06:2021 - Vulnerable and Outdated Components | webauthn-rs 0.5, rustls 0.23 (check for known CVEs) | Informational |
| A07:2021 - Identification and Authentication Failures | C2 (rotation state stuck), H1 (DoS via state exhaustion) | Critical/High |
| A09:2021 - Security Logging and Monitoring Failures | L3 (silent error swallowing in audit log) | Low |
| A10:2021 - Server-Side Request Forgery | N/A | N/A |

---

## Remediation Priority

1. **Immediate** (Critical):
   - Implement actual certificate chain verification in `verify_chain` (C1)
   - Fix rotation state recovery on failure (C2)

2. **Short-term** (High):
   - Add TTL and size limits to WebAuthn state stores (H1)
   - Sanitize all API error responses to prevent information leakage (H2)
   - Document or harden the mTLS middleware's dependency on transport-layer verification (H3)

3. **Medium-term** (Medium):
   - Prevent WebAuthn state overwrites for the same user (M1)
   - Replace blocking I/O with async equivalents in rotation persistence (M2)
   - Use cryptographically random serial numbers (M3)
   - Improve PEM validation to parse actual certificates (M4)

4. **Long-term** (Low/Informational):
   - Remove information disclosure about user credential status (L1)
   - Persist rotation state for crash recovery (L2)
   - Log persistence errors instead of silently discarding (L3)
   - Remove or implement unused client certificate store (L4)
   - Audit dependency versions for known CVEs (I6)
