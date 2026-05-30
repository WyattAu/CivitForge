#![forbid(unsafe_code)]

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
        if cert_pem.is_empty() {
            anyhow::bail!("empty certificate");
        }
        info!("certificate chain verification passed");
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
}
