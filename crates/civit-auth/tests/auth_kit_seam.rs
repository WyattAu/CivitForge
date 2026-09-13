//! Integration tests for the `civit-auth` estate-kit seams:
//!
//! - `password::{hash_password, verify_password}` → `salting`
//! - `jwt::JwtService` → `tokenkit::service::JwtService`
//!
//! These lock the authentication contracts every civit-* service relies on.

use civit_auth::jwt::JwtService;
use civit_auth::password::{hash_password, verify_password};

const PASSWORD: &str = "hunter2-but-database-safe";

#[test]
fn password_hash_and_verify_round_trip() {
    let hash = hash_password(PASSWORD).expect("hashing must succeed");
    assert_ne!(hash, PASSWORD);
    assert!(verify_password(PASSWORD, &hash), "correct password verifies");
}

#[test]
fn password_verify_rejects_wrong_password_and_garbage_hash() {
    let hash = hash_password(PASSWORD).expect("hash");
    assert!(!verify_password("not the password", &hash));
    assert!(!verify_password(PASSWORD, "definitely-not-a-phc-hash"));
}

#[test]
fn password_hashes_are_salted_per_call() {
    let a = hash_password(PASSWORD).expect("hash a");
    let b = hash_password(PASSWORD).expect("hash b");
    assert_ne!(a, b, "salting kit must produce a distinct hash per call");
}

#[test]
fn jwt_generate_and_validate_round_trip() {
    let svc = JwtService::new("unit-test-signing-secret-0123456789ab", 1).expect("service");
    let token = svc
        .generate_token("user-42", "wyatt", "admin", Some("org-7"))
        .expect("token");
    let claims = svc.validate_token(&token).expect("claims");
    assert_eq!(claims.sub, "user-42");
    assert_eq!(claims.username, "wyatt");
    assert_eq!(claims.role, "admin");
    assert_eq!(claims.org_id.as_deref(), Some("org-7"));
    assert_eq!(claims.iss.as_deref(), Some("civitforge"));
}

#[test]
fn jwt_rejects_token_signed_with_other_secret() {
    let a = JwtService::new("secret-one-0123456789abcdef-32bytes", 1).expect("a");
    let b = JwtService::new("secret-two-0123456789abcdef-32bytes", 1).expect("b");
    let token = a.generate_token("u", "n", "member", None).expect("token");
    assert!(b.validate_token(&token).is_err(), "cross-secret token must fail");
}

#[test]
fn jwt_extract_bearer_parses_authorization_header() {
    assert_eq!(JwtService::extract_bearer("Bearer abc.def.ghi"), Some("abc.def.ghi"));
    assert_eq!(JwtService::extract_bearer("Basic dXNlcjpwYXNz"), None);
    assert_eq!(JwtService::extract_bearer(""), None);
}
