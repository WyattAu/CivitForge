#![deny(unsafe_code)]

#[cfg(feature = "mtls-axum")]
pub mod axum;
pub mod config;
pub mod rotation;

use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, ExtendedKeyUsagePurpose, IsCa,
    KeyPair, KeyUsagePurpose, SanType,
};
use sha2::Digest;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Certificate {
    pub cert_pem: String,
    pub key_pem: String,
    pub serial: String,
    pub common_name: String,
}

impl Certificate {
    pub fn fingerprint_sha256(&self) -> String {
        let hash = sha2::Sha256::digest(self.cert_pem.as_bytes());
        hex::encode(hash)
    }
}

pub struct CertificateAuthority {
    ca_cert: Certificate,
    ca_key_pair: KeyPair,
    ca_rcgen_cert: rcgen::Certificate,
}

impl CertificateAuthority {
    pub fn new(common_name: &str) -> anyhow::Result<Self> {
        let mut params = CertificateParams::default();
        let mut distinguished_name = DistinguishedName::new();
        distinguished_name.push(DnType::CommonName, common_name);
        params.distinguished_name = distinguished_name;
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
            KeyUsagePurpose::DigitalSignature,
        ];

        let key_pair = KeyPair::generate()?;
        let cert = params.self_signed(&key_pair)?;
        let serial = format!(
            "{:016x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs()
        );

        info!(cn = common_name, "created certificate authority");

        Ok(Self {
            ca_cert: Certificate {
                cert_pem: cert.pem(),
                key_pem: key_pair.serialize_pem(),
                serial,
                common_name: common_name.into(),
            },
            ca_key_pair: key_pair,
            ca_rcgen_cert: cert,
        })
    }

    pub fn ca_certificate(&self) -> &Certificate {
        &self.ca_cert
    }

    pub fn issue_certificate(
        &self,
        common_name: &str,
        sans: &[String],
        days_valid: u32,
    ) -> anyhow::Result<Certificate> {
        let mut params = CertificateParams::default();
        let mut distinguished_name = DistinguishedName::new();
        distinguished_name.push(DnType::CommonName, common_name);
        params.distinguished_name = distinguished_name;
        params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ];
        params.extended_key_usages = vec![
            ExtendedKeyUsagePurpose::ServerAuth,
            ExtendedKeyUsagePurpose::ClientAuth,
        ];

        for san in sans {
            if let Some(rest) = san.strip_prefix("dns:") {
                params
                    .subject_alt_names
                    .push(SanType::DnsName(rest.try_into()?));
            } else if let Some(rest) = san.strip_prefix("ip:") {
                let ip: std::net::IpAddr = rest.parse()?;
                params.subject_alt_names.push(SanType::IpAddress(ip));
            } else {
                params
                    .subject_alt_names
                    .push(SanType::DnsName(san.as_str().try_into()?));
            }
        }

        let key_pair = KeyPair::generate()?;
        let serial = format!(
            "{:016x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs()
                + 1
        );

        let ca_cert = &self.ca_rcgen_cert;

        let cert = params.signed_by(&key_pair, ca_cert, &self.ca_key_pair)?;

        info!(
            cn = common_name,
            days = days_valid,
            sans = sans.len(),
            "issued certificate"
        );

        Ok(Certificate {
            cert_pem: cert.pem(),
            key_pem: key_pair.serialize_pem(),
            serial,
            common_name: common_name.into(),
        })
    }

    pub fn verify_chain(&self, cert_pem: &str) -> anyhow::Result<bool> {
        use x509_parser::certificate::X509Certificate;
        use x509_parser::pem::parse_x509_pem;
        use x509_parser::prelude::FromDer;

        if cert_pem.is_empty() {
            anyhow::bail!("empty certificate");
        }

        let mut der_list: Vec<Vec<u8>> = Vec::new();
        let mut input = cert_pem.as_bytes();

        loop {
            if input.is_empty() {
                break;
            }
            match parse_x509_pem(input) {
                Ok((remaining, pem)) => {
                    der_list.push(pem.contents);
                    input = remaining;
                }
                Err(_) => break,
            }
        }

        if der_list.is_empty() {
            anyhow::bail!("no valid certificates found in chain PEM");
        }

        let mut certs: Vec<X509Certificate<'_>> = Vec::new();
        for der_bytes in &der_list {
            let (_, x509_cert) = X509Certificate::from_der(der_bytes)
                .map_err(|e| anyhow::anyhow!("failed to parse X.509 certificate: {e}"))?;
            certs.push(x509_cert);
        }

        for cert in &certs {
            let validity = cert.validity();
            let now_i64 = chrono::Utc::now().timestamp();
            if now_i64 < validity.not_before.timestamp() {
                anyhow::bail!(
                    "certificate is not yet valid (not_before: {})",
                    validity.not_before.timestamp()
                );
            }
            if now_i64 > validity.not_after.timestamp() {
                anyhow::bail!(
                    "certificate has expired (not_after: {})",
                    validity.not_after.timestamp()
                );
            }
        }

        let ca_pem_bytes = self.ca_cert.cert_pem.as_bytes();
        let (_, ca_pem_data) = parse_x509_pem(ca_pem_bytes)
            .map_err(|e| anyhow::anyhow!("failed to parse CA certificate PEM: {e}"))?;
        let ca_der = ca_pem_data.contents;
        let (_, ca_cert) = X509Certificate::from_der(&ca_der)
            .map_err(|e| anyhow::anyhow!("failed to parse CA certificate: {e}"))?;

        if certs.len() == 1 {
            let end_cert = &certs[0];
            let issuer_raw = end_cert.issuer().as_raw();
            let ca_subject_raw = ca_cert.subject().as_raw();
            if issuer_raw != ca_subject_raw {
                anyhow::bail!("single certificate issuer does not match CA subject");
            }

            end_cert
                .verify_signature(Some(ca_cert.public_key()))
                .map_err(|e| {
                    anyhow::anyhow!("single certificate signature verification failed: {e}")
                })?;

            info!("single certificate verified against CA");
            return Ok(true);
        }

        for i in 0..certs.len() - 1 {
            let child = &certs[i];
            let parent = &certs[i + 1];

            let issuer_raw = child.issuer().as_raw();
            let subject_raw = parent.subject().as_raw();
            if issuer_raw != subject_raw {
                anyhow::bail!(
                    "certificate chain break: cert[{}] issuer does not match cert[{}] subject",
                    i,
                    i + 1
                );
            }

            child
                .verify_signature(Some(parent.public_key()))
                .map_err(|e| {
                    anyhow::anyhow!("certificate[{i}] signature verification failed: {e}")
                })?;
        }

        let last_cert = certs.last().unwrap();
        let issuer_raw = last_cert.issuer().as_raw();
        let ca_subject_raw = ca_cert.subject().as_raw();
        if issuer_raw != ca_subject_raw {
            anyhow::bail!("chain root issuer does not match CA subject");
        }

        last_cert
            .verify_signature(Some(ca_cert.public_key()))
            .map_err(|e| anyhow::anyhow!("chain root signature verification failed: {e}"))?;

        info!(
            chain_len = certs.len(),
            "certificate chain verification passed"
        );
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ca() {
        let ca = CertificateAuthority::new("CivitForge CA").unwrap();
        assert_eq!(ca.ca_certificate().common_name, "CivitForge CA");
        assert!(!ca.ca_certificate().cert_pem.is_empty());
        assert!(!ca.ca_certificate().key_pem.is_empty());
    }

    #[test]
    fn test_issue_certificate() {
        let ca = CertificateAuthority::new("Test CA").unwrap();
        let cert = ca
            .issue_certificate("test.civit.local", &["test.civit.local".into()], 365)
            .unwrap();
        assert_eq!(cert.common_name, "test.civit.local");
        assert!(!cert.cert_pem.is_empty());
        assert!(!cert.key_pem.is_empty());
    }

    #[test]
    fn test_certificate_fingerprint() {
        let ca = CertificateAuthority::new("FP CA").unwrap();
        let fp = ca.ca_certificate().fingerprint_sha256();
        assert!(!fp.is_empty());
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn test_issue_with_sans() {
        let ca = CertificateAuthority::new("SAN CA").unwrap();
        let cert = ca
            .issue_certificate(
                "multi.local",
                &["dns:multi.local".into(), "dns:alt.local".into()],
                90,
            )
            .unwrap();
        assert_eq!(cert.common_name, "multi.local");
    }

    #[test]
    fn test_verify_chain() {
        let ca = CertificateAuthority::new("Chain CA").unwrap();
        let cert = ca
            .issue_certificate("chain.local", &["chain.local".into()], 30)
            .unwrap();
        let valid = ca.verify_chain(&cert.cert_pem).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_verify_chain_empty() {
        let ca = CertificateAuthority::new("Empty CA").unwrap();
        assert!(ca.verify_chain("").is_err());
    }

    #[test]
    fn test_verify_chain_invalid_pem() {
        let ca = CertificateAuthority::new("Invalid CA").unwrap();
        assert!(ca.verify_chain("not-a-valid-pem").is_err());
    }
}
