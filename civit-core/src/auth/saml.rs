#![forbid(unsafe_code)]

use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlConfig {
    pub entity_id: String,
    pub sso_url: String,
    pub slo_url: String,
    pub certificate: String,
    pub name_id_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlRequest {
    pub id: String,
    pub issue_instant: DateTime<Utc>,
    pub destination: String,
    pub issuer: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SamlStatus {
    Success,
    Failure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlResponse {
    pub id: String,
    pub issue_instant: DateTime<Utc>,
    pub status: SamlStatus,
    pub name_id: String,
    pub attributes: HashMap<String, String>,
}

pub struct SamlService {
    pub config: SamlConfig,
}

impl SamlService {
    pub fn new(config: SamlConfig) -> Self {
        Self { config }
    }

    pub fn build_authn_request(&self) -> String {
        let request_id = format!("_{}", uuid::Uuid::new_v4());
        let issue_instant = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
                     xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion"
                     ID="{id}"
                     Version="2.0"
                     IssueInstant="{issue_instant}"
                     Destination="{destination}"
                     ProtocolBinding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
                     AssertionConsumerServiceURL="{acs_url}">
  <saml:Issuer>{issuer}</saml:Issuer>
  <samlp:NameIDPolicy Format="{name_id_format}" AllowCreate="true"/>
  <samlp:RequestedAuthnContext Comparison="exact">
    <saml:AuthnContextClassRef>urn:oasis:names:tc:SAML:2.0:ac:classes:PasswordProtectedTransport</saml:AuthnContextClassRef>
  </samlp:RequestedAuthnContext>
</samlp:AuthnRequest>"#,
            id = request_id,
            issue_instant = issue_instant,
            destination = self.config.sso_url,
            acs_url = self.config.slo_url,
            issuer = self.config.entity_id,
            name_id_format = self.config.name_id_format,
        );
        info!(id = %request_id, "SAML AuthnRequest generated");
        xml
    }

    pub fn parse_response(saml_response: &str) -> Result<SamlResponse> {
        let id = extract_xml_attr(saml_response, "Response", "ID").unwrap_or_default();
        let issue_instant_str =
            extract_xml_attr(saml_response, "Response", "IssueInstant").unwrap_or_default();
        let issue_instant = DateTime::parse_from_rfc3339(&issue_instant_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let name_id = extract_tag_text_any_ns(saml_response, "NameID").unwrap_or_default();

        let status = if saml_response.contains("StatusCode") && saml_response.contains("Success") {
            SamlStatus::Success
        } else {
            SamlStatus::Failure
        };

        let mut attributes = HashMap::new();
        let mut search_from = 0;
        while let Some(attr_start) = saml_response[search_from..]
            .find("<Attribute ")
            .or_else(|| saml_response[search_from..].find("<saml:Attribute "))
        {
            let abs_start = search_from + attr_start;
            let close_tag = "</saml:Attribute>";
            let close_tag2 = "</Attribute>";
            let end_offset = saml_response[abs_start..]
                .find(close_tag)
                .or_else(|| saml_response[abs_start..].find(close_tag2));
            if let Some(tag_len) = end_offset {
                let close_len = close_tag.len();
                let full = &saml_response[abs_start..abs_start + tag_len + close_len];
                if let Some(name_start) = full.find("Name=\"") {
                    let name_part = &full[name_start + 6..];
                    if let Some(name_end) = name_part.find('"') {
                        let attr_name = &name_part[..name_end];
                        if let Some(value) = extract_tag_text_any_ns(full, "AttributeValue") {
                            attributes.insert(attr_name.to_string(), value);
                        }
                    }
                }
                search_from = abs_start + tag_len + close_len;
            } else {
                break;
            }
        }

        info!(id = %id, status = ?status, "SAML response parsed");
        Ok(SamlResponse {
            id,
            issue_instant,
            status,
            name_id,
            attributes,
        })
    }

    pub fn is_valid_signature(&self) -> bool {
        // SECURITY: Signature validation is fail-closed until XML-DSIG implementation.
        // Full XML-DSIG validation requires:
        // 1. Extract <ds:Signature> element from SAML response XML
        // 2. Compute SHA-256 digest of <ds:SignedInfo> canonicalization
        // 3. Verify RSA/ECDSA signature against the configured X.509 certificate
        // 4. Validate certificate chain and expiration
        // This will be implemented with the xmlsec or ring crates in Phase 1.
        // Until then, SAML authentication is not usable in production.
        false
    }
}

fn find_tag_start(xml: &str, from: usize, tag_name: &str) -> Option<usize> {
    let substr = &xml[from..];
    let ns_prefixed = format!(":{tag_name}");
    let _direct = format!("<{tag_name}");
    let mut pos = 0;
    while let Some(lt) = substr[pos..].find('<') {
        let abs = pos + lt;
        let after_lt = &substr[abs + 1..];
        if after_lt.starts_with(tag_name) {
            let next = after_lt.as_bytes().get(tag_name.len()).copied();
            if next == Some(b' ')
                || next == Some(b'>')
                || next == Some(b'/')
                || next == Some(b'\n')
                || next == Some(b'\r')
            {
                return Some(from + abs);
            }
        }
        if after_lt.contains(&ns_prefixed) {
            if let Some(ns_pos) = after_lt.find(&ns_prefixed) {
                let after_ns = &after_lt[ns_pos + ns_prefixed.len()..];
                let next = after_ns.as_bytes().first().copied();
                if next == Some(b' ')
                    || next == Some(b'>')
                    || next == Some(b'/')
                    || next == Some(b'\n')
                    || next == Some(b'\r')
                {
                    return Some(from + abs + ns_pos);
                }
            }
        }
        pos = abs + 1;
        if pos >= substr.len() {
            break;
        }
    }
    None
}

fn extract_xml_attr(xml: &str, tag_name: &str, attr_name: &str) -> Option<String> {
    let tag_start = find_tag_start(xml, 0, tag_name)?;
    let after_tag = &xml[tag_start..];
    let close_bracket = after_tag.find('>')?;
    let tag_header = &after_tag[..close_bracket];
    let pattern = format!("{attr_name}=\"");
    if let Some(p) = tag_header.find(&pattern) {
        let after_p = &tag_header[p + pattern.len()..];
        if let Some(q) = after_p.find('"') {
            return Some(after_p[..q].to_string());
        }
    }
    None
}

fn extract_tag_text_any_ns(xml: &str, tag_name: &str) -> Option<String> {
    let tag_start = find_tag_start(xml, 0, tag_name)?;
    let after_open = &xml[tag_start..];
    let close_bracket = after_open.find('>')?;
    let content_start = tag_start + close_bracket + 1;

    let close_pattern = format!(":{tag_name}>");
    let close_pattern2 = format!("<{tag_name}>");
    let close_pattern3 = format!("<{tag_name}/");

    let mut end_pos: Option<usize> = None;
    let mut pos = content_start;
    while let Some(lt) = xml[pos..].find('<') {
        let abs = pos + lt;
        let fragment = &xml[abs..];
        if fragment.contains(&close_pattern)
            || fragment.contains(&close_pattern2)
            || fragment.contains(&close_pattern3)
        {
            end_pos = Some(abs);
            break;
        }
        pos = abs + 1;
        if pos >= xml.len() {
            break;
        }
    }

    let end = end_pos?;
    let text = xml[content_start..end].trim().to_string();
    if !text.is_empty() { Some(text) } else { None }
}

#[cfg(test)]
#[allow(dead_code)]
fn extract_tag_text(xml: &str, tag_name: &str) -> Option<String> {
    let open = format!("<{tag_name}>");
    let close = format!("</{tag_name}>");
    if let Some(start) = xml.find(&open) {
        let content_start = start + open.len();
        if let Some(end) = xml[content_start..].find(&close) {
            let text = xml[content_start..content_start + end].trim().to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_service() -> SamlService {
        SamlService::new(SamlConfig {
            entity_id: "https://app.example.com/saml".into(),
            sso_url: "https://idp.example.com/sso".into(),
            slo_url: "https://app.example.com/saml/acs".into(),
            certificate: "-----BEGIN CERTIFICATE-----\nMIIB...\n-----END CERTIFICATE-----".into(),
            name_id_format: "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress".into(),
        })
    }

    #[test]
    fn test_build_authn_request() {
        let svc = make_service();
        let xml = svc.build_authn_request();
        assert!(xml.contains("<samlp:AuthnRequest"));
        assert!(xml.contains("<saml:Issuer>https://app.example.com/saml</saml:Issuer>"));
        assert!(xml.contains("Destination=\"https://idp.example.com/sso\""));
        assert!(xml.contains("urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress"));
        assert!(xml.contains("PasswordProtectedTransport"));
        assert!(xml.contains("IssueInstant=\""));
        assert!(xml.starts_with("<?xml version=\"1.0\""));
    }

    #[test]
    fn test_parse_response_success() {
        let saml_xml = r#"<samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
    ID="_resp-001"
    IssueInstant="2024-01-15T10:30:00Z"
    Version="2.0">
  <saml:Issuer xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion">https://idp.example.com</saml:Issuer>
  <samlp:Status>
    <samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Success"/>
  </samlp:Status>
  <saml:Assertion xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion">
    <saml:Subject>
      <saml:NameID Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress">alice@example.com</saml:NameID>
    </saml:Subject>
    <saml:AttributeStatement>
      <saml:Attribute Name="email">
        <saml:AttributeValue>alice@example.com</saml:AttributeValue>
      </saml:Attribute>
      <saml:Attribute Name="given_name">
        <saml:AttributeValue>Alice</saml:AttributeValue>
      </saml:Attribute>
    </saml:AttributeStatement>
  </saml:Assertion>
</samlp:Response>"#;
        let response = SamlService::parse_response(saml_xml).unwrap();
        assert_eq!(response.id, "_resp-001");
        assert_eq!(response.status, SamlStatus::Success);
        assert_eq!(response.name_id, "alice@example.com");
        assert_eq!(
            response.attributes.get("email").unwrap(),
            "alice@example.com"
        );
        assert_eq!(response.attributes.get("given_name").unwrap(), "Alice");
    }

    #[test]
    fn test_parse_response_failure() {
        let saml_xml = r#"<samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
    ID="_resp-fail"
    IssueInstant="2024-01-15T10:30:00Z"
    Version="2.0">
  <samlp:Status>
    <samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Responder"/>
  </samlp:Status>
</samlp:Response>"#;
        let response = SamlService::parse_response(saml_xml).unwrap();
        assert_eq!(response.status, SamlStatus::Failure);
        assert!(response.name_id.is_empty());
    }

    #[test]
    fn test_parse_response_no_attributes() {
        let saml_xml = r#"<samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
    ID="_resp-plain"
    IssueInstant="2024-01-15T10:30:00Z"
    Version="2.0">
  <samlp:Status>
    <samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Success"/>
  </samlp:Status>
</samlp:Response>"#;
        let response = SamlService::parse_response(saml_xml).unwrap();
        assert!(response.attributes.is_empty());
    }

    #[test]
    fn test_is_valid_signature_fail_closed() {
        let svc = make_service();
        // Signature validation is fail-closed until XML-DSIG is implemented.
        // This test verifies the security property: unsigned responses are rejected.
        assert!(!svc.is_valid_signature());
    }

    #[test]
    fn test_build_authn_request_generates_unique_ids() {
        let svc = make_service();
        let xml1 = svc.build_authn_request();
        let xml2 = svc.build_authn_request();
        assert_ne!(xml1, xml2);
        assert!(extract_xml_attr(&xml1, "AuthnRequest", "ID").is_some());
        assert!(extract_xml_attr(&xml2, "AuthnRequest", "ID").is_some());
        assert_ne!(
            extract_xml_attr(&xml1, "AuthnRequest", "ID"),
            extract_xml_attr(&xml2, "AuthnRequest", "ID")
        );
    }
}
