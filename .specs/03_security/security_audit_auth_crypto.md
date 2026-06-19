# Security Audit: Authentication & Crypto Modules

**Date:** 2026-06-19
**Scope:** `crates/civit-auth/src/*`, `crates/civit-crypto/src/*`
**Severity Scale:** CRITICAL / HIGH / MEDIUM / LOW / INFO

---

## Executive Summary

The authentication and cryptographic code demonstrates solid foundational design (ring for AES-GCM, proper bcrypt usage, HMAC-SHA256 for signatures). However, several issues ranging from timing-safe comparison gaps to insufficient bcrypt cost factors require remediation before production deployment.

| Severity | Count |
|----------|-------|
| CRITICAL | 1 |
| HIGH | 3 |
| MEDIUM | 5 |
| LOW | 4 |
| INFO | 2 |

---

## CRITICAL Findings

### C-1: Timing-unsafe hash comparison in `civit-crypto/src/hash.rs`

**File:** `crates/civit-crypto/src/hash.rs:89,102`
**Issue:** The `verify()` and `verify_with_algorithm()` functions compare hex-encoded hash digests using Rust's `==` operator, which short-circuits on the first differing byte. This enables a timing side-channel attack where an attacker can determine the correct hash one character at a time.

```rust
// Line 89 - timing leak
if result.hex == expected_hex {
// Line 102 - timing leak
result.hex == expected_hex
```

**Impact:** An attacker can reconstruct a valid hash digest via repeated requests with measurable timing differences. In contexts where this verifies integrity of sensitive data (e.g., file checksums, token hashes), this is exploitable.

**Remediation:** Use `subtle::ConstantTimeEq` or `ring::constant_time::verify_slices_are_equal` for all digest comparisons. Since the comparison is on hex strings, decode to bytes first and compare with constant-time logic:

```rust
use subtle::ConstantTimeEq;

pub fn verify(data: &[u8], expected_hex: &str) -> bool {
    let algorithms = [HashAlgorithm::Sha256, HashAlgorithm::Sha512];
    for algo in algorithms {
        let result = Self::hash(algo, data);
        let expected_bytes = match hex::decode(expected_hex) {
            Ok(b) => b,
            Err(_) => return false,
        };
        if result.bytes.ct_eq(&expected_bytes).into() {
            return true;
        }
    }
    false
}
```

---

## HIGH Findings

### H-1: Bcrypt cost factor too low (`DEFAULT_COST` = 10)

**File:** `crates/civit-auth/src/password.rs:70`
**Issue:** `bcrypt::hash(password, bcrypt::DEFAULT_COST)` uses a cost factor of 10 (bcrypt 0.17 default). NIST SP 800-63B and OWASP recommend a cost factor of at least 12 for password hashing. At cost 10, hashing takes ~100ms on modern hardware; at cost 12, it takes ~400ms, providing significantly better brute-force resistance.

```rust
pub fn hash_password(password: &str) -> Result<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST) // cost = 10
```

**Impact:** Reduces the cost of offline brute-force attacks against leaked password hashes by approximately 4x compared to cost 12.

**Remediation:** Use a cost factor of 12 or higher. Make it configurable:

```rust
const BCRYPT_COST: u32 = 12;

pub fn hash_password(password: &str) -> Result<String> {
    bcrypt::hash(password, BCRYPT_COST)
        .map_err(|e| AuthError::Internal(format!("Failed to hash password")))
}
```

Also, migrate existing hashes by re-hashing on next successful login if the stored hash cost < 12.

### H-2: No JWT secret length validation

**File:** `crates/civit-auth/src/jwt.rs:23-28`
**Issue:** `JwtService::new()` accepts any `&str` as the JWT secret without validating minimum length or entropy. A short or weak secret (e.g., `"secret"`, `"test"`) allows JWT forgery. The test code uses `"test-secret-key-32bytes-minimums"` which, while adequate for tests, signals no production guardrail exists.

```rust
pub fn new(secret: &str, expiry_hours: u64) -> Self {
    Self {
        encoding_key: EncodingKey::from_secret(secret.as_bytes()), // no validation
        decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        expiry_hours,
    }
}
```

**Impact:** If a short secret is deployed, an attacker can forge arbitrary JWTs with any claims (admin role, any user_id).

**Remediation:** Enforce a minimum of 32 bytes (256 bits) at construction time:

```rust
pub fn new(secret: &str, expiry_hours: u64) -> Result<Self> {
    if secret.len() < 32 {
        return Err(AuthError::Config(
            "JWT secret must be at least 32 bytes".into(),
        ));
    }
    Ok(Self {
        encoding_key: EncodingKey::from_secret(secret.as_bytes()),
        decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        expiry_hours,
    })
}
```

### H-3: PAT token hash comparison uses `==` (timing-unsafe)

**File:** `crates/civit-auth/src/middleware.rs:22-26`
**Issue:** While `hash_token()` uses SHA-256, the comparison of the resulting hex digest with stored hashes is done via `==` (inferred from downstream usage in `token_validator`). The same issue applies to `pat.rs:hash_token()`.

```rust
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

The hash is returned as a hex `String` and compared with `==` wherever it is used for verification. This creates the same timing side-channel as C-1.

**Impact:** An attacker can brute-force a PAT token hash one hex character at a time using timing differences.

**Remediation:** Return raw bytes instead of hex strings for comparison contexts, and use `subtle::ConstantTimeEq` for all comparisons. Alternatively, if using hex encoding, decode both sides to bytes before comparing with constant-time equality.

---

## MEDIUM Findings

### M-1: Custom HKDF implementation in `repo_keys.rs`

**File:** `crates/civit-crypto/src/repo_keys.rs:106-142`
**Issue:** The `hmac_expand()` function implements HKDF-Expand manually rather than using a vetted library like the `hkdf` crate. While the implementation appears correct (follows RFC 5869), hand-rolled KDFs are a common source of vulnerabilities and bypass formal security review.

```rust
fn hmac_expand(prk: &[u8], info: &[u8], out: &mut [u8]) -> Result<(), String> {
    // manual HKDF-Expand implementation
}
```

**Impact:** Any subtle deviation from RFC 5869 could weaken key derivation. Manual implementations also miss edge cases (e.g., info concatenation, counter overflow) that library implementations handle.

**Remediation:** Replace with the `hkdf` crate:

```rust
use hkdf::Hkdf;
use sha2::Sha256;

// In derive():
let hkdf = Hkdf::<Sha256>::new(Some(master_key), &[]);
hkdf.expand(repo_id.as_bytes(), &mut expanded)
    .map_err(|e| RepoKeyError::Derivation(format!("HKDF expand failed")))?;
```

### M-2: Secrets not zeroed from memory after use

**Files:** Multiple locations
- `crates/civit-auth/src/password.rs` - password `&str` not zeroed
- `crates/civit-auth/src/jwt.rs` - secret retained in `EncodingKey`/`DecodingKey`
- `crates/civit-crypto/src/repo_keys.rs:31` - `key_bytes: [u8; 32]` never zeroed
- `crates/civit-auth/src/pat.rs` - generated tokens not zeroed

**Issue:** Sensitive material (passwords, keys, tokens) is not zeroed from memory after use. While Rust's garbage collector will eventually free the memory, the data persists until overwritten, and could be recovered via memory dumps, swap files, or core dumps.

**Impact:** Memory forensics attacks can extract secrets after they are logically discarded.

**Remediation:** Use the `zeroize` crate for all sensitive buffers:

```rust
use zeroize::Zeroize;

// For key_bytes in RepoEncryptionKey:
key_bytes.zeroize();

// For password strings:
let mut password = password.to_string();
// ... use password ...
password.zeroize();
```

For `EncodingKey`/`DecodingKey` in JWT, the `jsonwebtoken` crate does not expose zeroization, but the secret should at minimum be loaded from a source that supports zeroization (e.g., `secrecy::Secret`).

### M-3: RSA SSH keys accepted without minimum bit-length check

**File:** `crates/civit-auth/src/ssh.rs:23-38`
**Issue:** `validate_ssh_key_type()` accepts `ssh-rsa` without checking the key's actual bit length. NIST deprecated 1024-bit RSA in 2013 and recommends minimum 2048-bit keys. An attacker could register a 512-bit or 1024-bit RSA key.

```rust
let valid_types = [
    "ssh-ed25519",
    "ssh-rsa", // accepted regardless of key size
    ...
];
```

**Impact:** Weak RSA keys (1024-bit or less) can be factored in practical timeframes, allowing an attacker to forge SSH authentication.

**Remediation:** Either:
1. Decode the base64 public key and parse the RSA key to check bit length (>= 2048), or
2. Prefer Ed25519/ECDSA keys and reject `ssh-rsa` entirely (modern recommendation), or
3. At minimum, document and enforce a minimum key size check in the API layer.

### M-4: `hash_token` in `pat.rs` uses bare SHA-256 without keying

**File:** `crates/civit-auth/src/pat.rs:7-11`
**Issue:** PAT tokens are hashed with unkeyed SHA-256 for storage. While SHA-256 preimage resistance is not practically broken, using a keyed hash (HMAC) with a server-side key provides defense-in-depth against potential future SHA-256 weaknesses and prevents rainbow table attacks on leaked hash databases.

```rust
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

**Impact:** If the hash database is leaked, an attacker can brute-force tokens using commodity hardware. Keyed hashing with a server-side secret forces the attacker to also obtain the key.

**Remediation:** Use HMAC-SHA256 with a server-side pepper key stored in environment variables or a secrets manager:

```rust
pub fn hash_token(token: &str, pepper: &[u8]) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(pepper)
        .expect("HMAC accepts any key length");
    mac.update(token.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
```

### M-5: `hash_token` in `middleware.rs` is identical and has same issues

**File:** `crates/civit-auth/src/middleware.rs:22-26`
**Issue:** Duplicate `hash_token` implementation with same unkeyed SHA-256 issue as M-4. Code duplication increases maintenance burden and risk of inconsistency.

**Remediation:** Consolidate into a single implementation (ideally the one in `pat.rs`) and apply the same HMAC fix from M-4.

---

## LOW Findings

### L-1: Password minimum length default is 8 characters

**File:** `crates/civit-auth/src/password.rs:16`
**Issue:** The default `PasswordPolicy` sets `min_length: 8`. OWASP and NIST SP 800-63B recommend a minimum of 12 characters for user-chosen passwords, as 8-character passwords can be brute-forced in hours with modern GPUs.

```rust
fn default() -> Self {
    Self {
        min_length: 8, // too low
```

**Remediation:** Increase default to 12 characters. For existing users, enforce on next password change.

### L-2: No rate limiting on password authentication endpoint

**Files:** `crates/civit-core/src/api/auth.rs`, `crates/civit-core/src/api/mod.rs`
**Issue:** Rate limiting is applied globally (100 requests/60s by default) but there is no per-endpoint or per-IP rate limiting specifically for the login/password-verification endpoint. The SSH auth module has its own `RateLimiter` (5 attempts/60s with 300s ban), but HTTP password authentication does not.

**Impact:** An attacker can attempt thousands of password guesses per minute if the global rate limit is generous, or bypass global limits by distributing requests.

**Remediation:** Add per-IP rate limiting specifically for login endpoints (e.g., 5 failed attempts per 60 seconds, progressive backoff or account lockout).

### L-3: Error messages may leak implementation details

**File:** `crates/civit-auth/src/password.rs:71`
**Issue:** The error message from `hash_password` includes the bcrypt error detail: `format!("Failed to hash password: {e}")`. This could leak internal implementation details in error responses if errors propagate to the client.

```rust
.map_err(|e| AuthError::Internal(format!("Failed to hash password: {e}")))
```

**Remediation:** Use a generic error message and log the detailed error server-side:

```rust
.map_err(|e| {
    tracing::error!(error = %e, "password hashing failed");
    AuthError::Internal("password hashing failed".into())
})
```

### L-4: JWT validation uses `Validation::default()` without explicit algorithm restrictions

**File:** `crates/civit-auth/src/jwt.rs:53`
**Issue:** `Validation::default()` from the `jsonwebtoken` crate allows the `HS256` algorithm by default, which is correct for HMAC-signed JWTs. However, it does not explicitly set `algorithms: vec![Algorithm::HS256]`, meaning if the library default changes, other algorithms could be accepted.

```rust
let validation = Validation::default();
```

**Remediation:** Explicitly restrict to the expected algorithm:

```rust
use jsonwebtoken::Algorithm;

let mut validation = Validation::new(Algorithm::HS256);
validation.set_required_spec_claims(&["exp", "iat"]);
```

---

## INFO Findings

### I-1: Test secrets in `#[cfg(test)]` are safe

**Files:** `jwt.rs:72,97-98`, `repo_keys.rs:247-252`
**Observation:** Hardcoded test secrets like `"test-secret-key-32bytes-minimums"` and the deterministic test master key are correctly gated behind `#[cfg(test)]` and will not compile into production binaries. No action required.

### I-2: FIPS self-test uses hardcoded test vectors (expected behavior)

**File:** `crates/civit-crypto/src/fips_selftest.rs:66-67, 92, 121`
**Observation:** The FIPS self-test module uses NIST/RFC test vectors (e.g., SHA-256 of `"abc"`, HMAC test from RFC 4231). These are standard known-answer tests and are appropriate for FIPS compliance verification. No action required.

---

## Summary of Remediation Priority

| # | Finding | Severity | Effort |
|---|---------|----------|--------|
| C-1 | Timing-unsafe hash comparison in hash.rs | CRITICAL | Low |
| H-1 | Bcrypt cost factor 10 -> 12 | HIGH | Low |
| H-2 | JWT secret length validation | HIGH | Low |
| H-3 | PAT token hash timing leak | HIGH | Low |
| M-1 | Custom HKDF -> use `hkdf` crate | MEDIUM | Medium |
| M-2 | Zeroize secrets in memory | MEDIUM | Medium |
| M-3 | RSA SSH key minimum bit-length | MEDIUM | Medium |
| M-4 | Unkeyed SHA-256 for PAT hashing | MEDIUM | Low |
| M-5 | Duplicate hash_token in middleware.rs | MEDIUM | Low |
| L-1 | Password min length 8 -> 12 | LOW | Low |
| L-2 | Rate limit on login endpoint | LOW | Medium |
| L-3 | Error message information leakage | LOW | Low |
| L-4 | Explicit JWT algorithm restriction | LOW | Low |

---

## Additional Recommendations

1. **Audit trail:** Consider adding structured logging for all authentication events (successful/failed logins, PAT creation, SSH key registration) with IP addresses for forensic analysis.

2. **Secret rotation:** Implement a mechanism for JWT secret rotation and PAT token revocation. Currently, JWTs cannot be revoked before expiry.

3. **HTTPS enforcement:** Ensure all authentication endpoints are only accessible over HTTPS (TLS 1.2+). This should be enforced at the reverse proxy or application layer.

4. **Password breach checking:** Consider integrating with the Have I Been Pwned API (k-anonymity model) to reject passwords found in known breaches.

5. **Argon2 consideration:** For new deployments, consider migrating from bcrypt to Argon2id (via the `argon2` crate), which provides better resistance to GPU/ASIC attacks and won the Password Hashing Competition.
