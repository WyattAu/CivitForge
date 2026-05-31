#![forbid(unsafe_code)]

pub mod jwt;
pub mod mfa;
pub mod oidc;
pub mod rbac;
pub mod saml;
pub mod session;
pub mod token_rotation;

pub use jwt::{Claims, JwtService};
pub use mfa::{TotpService, WebAuthnService};
pub use oidc::OidcService;
pub use rbac::{Action, Permission, Policy, PolicyEngine, Role};
pub use saml::SamlService;
pub use session::TokenRotationService;
pub use token_rotation::{TokenRotationConfig, TokenValidationResult};
