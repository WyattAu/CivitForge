#![forbid(unsafe_code)]

use base64::Engine;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SamlError {
    #[error("XML parse error: {0}")]
    XmlParse(String),

    #[error("missing required element: {0}")]
    MissingElement(String),

    #[error("digest verification failed")]
    DigestMismatch,

    #[error("base64 decode error: {0}")]
    Base64(String),

    #[error("invalid attribute: {0}")]
    InvalidAttribute(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAssertion {
    pub assertion_id: String,
    pub issuer: String,
    pub subject_name_id: String,
    pub conditions_not_on_or_after: Option<String>,
    pub attributes: Vec<SamlAttribute>,
    pub issue_instant: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAttribute {
    pub name: String,
    pub name_format: Option<String>,
    pub values: Vec<String>,
}

pub fn parse_saml_response(encoded_response: &str) -> Result<SamlAssertion, SamlError> {
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded_response)
        .map_err(|e| SamlError::Base64(format!("base64 decode: {e}")))?;

    let xml = String::from_utf8(decoded)
        .map_err(|e| SamlError::XmlParse(format!("invalid UTF-8: {e}")))?;

    parse_saml_xml(&xml)
}

pub fn parse_saml_xml(xml: &str) -> Result<SamlAssertion, SamlError> {
    let assertion_id = extract_xml_attr(xml, "Assertion", "ID")
        .ok_or_else(|| SamlError::MissingElement("Assertion ID".into()))?;

    let issuer = extract_element_text(xml, "Issuer")
        .ok_or_else(|| SamlError::MissingElement("Issuer".into()))?;

    let subject_name_id = extract_xml_attr(xml, "NameID", "Format")
        .map(|_| extract_element_text(xml, "NameID").unwrap_or_default())
        .unwrap_or_default();

    let issue_instant = extract_xml_attr(xml, "Assertion", "IssueInstant")
        .or_else(|| extract_element_text(xml, "IssueInstant"));
    let conditions_not_on_or_after = extract_element_text(xml, "NotOnOrAfter");

    let attributes = extract_saml_attributes(xml);

    Ok(SamlAssertion {
        assertion_id,
        issuer,
        subject_name_id,
        conditions_not_on_or_after,
        attributes,
        issue_instant,
    })
}

fn extract_element_text(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let start = xml.find(&open)?;
    let content_start = xml[start + open.len()..]
        .find('>')
        .map(|i| start + open.len() + i + 1)?;
    let end = xml[content_start..].find(&close)?;
    let text = xml[content_start..content_start + end].trim().to_string();
    if text.is_empty() { None } else { Some(text) }
}

fn extract_xml_attr(xml: &str, tag: &str, attr: &str) -> Option<String> {
    let open = format!("<{tag}");
    let start = xml.find(&open)?;
    let tag_end = xml[start..].find('>')?;
    let tag_content = &xml[start..start + tag_end];

    let pattern = format!("{attr}=\"");
    let attr_start = tag_content.find(&pattern)?;
    let value_start = attr_start + pattern.len();
    let value_end = tag_content[value_start..].find('"')?;
    Some(tag_content[value_start..value_start + value_end].to_string())
}

fn extract_saml_attributes(xml: &str) -> Vec<SamlAttribute> {
    let mut attributes = Vec::new();

    let mut search_from = 0;
    while let Some(start) = xml[search_from..].find("<Attribute ") {
        let abs_start = search_from + start;
        let end = xml[abs_start..]
            .find("</Attribute>")
            .map(|e| abs_start + e + 13);
        let tag_end = end.unwrap_or(xml.len());
        let element = &xml[abs_start..tag_end];

        let name = extract_xml_attr(element, "Attribute", "Name").unwrap_or_default();
        let name_format = extract_xml_attr(element, "Attribute", "NameFormat");

        let mut values = Vec::new();
        let mut val_search = 0;
        while let Some(val_start) = element[val_search..].find("<AttributeValue") {
            let val_abs = val_search + val_start;
            let val_tag_end = element[val_abs..]
                .find('>')
                .map(|e| val_abs + e + 1)
                .unwrap_or(val_abs);
            let val_close = element[val_tag_end..].find("</AttributeValue>");
            let val_end = val_close.map(|e| val_tag_end + e);
            let val_text = match val_end {
                Some(e) => element[val_tag_end..e].trim().to_string(),
                None => String::new(),
            };
            if !val_text.is_empty() {
                values.push(val_text);
            }
            val_search = val_abs + 13;
        }

        if !name.is_empty() {
            attributes.push(SamlAttribute {
                name,
                name_format,
                values,
            });
        }

        search_from = abs_start + 11;
    }

    attributes
}

pub fn verify_digest(algorithm: &str, digest_value: &str, data: &[u8]) -> Result<(), SamlError> {
    use sha2::{Digest, Sha256};

    let digest_bytes = base64::engine::general_purpose::STANDARD
        .decode(digest_value)
        .map_err(|e| SamlError::Base64(format!("digest decode: {e}")))?;

    let computed = match algorithm {
        "http://www.w3.org/2001/04/xmlenc#sha256" | "SHA-256" => Sha256::digest(data).to_vec(),
        _ => {
            return Err(SamlError::XmlParse(format!(
                "unsupported algorithm: {algorithm}"
            )));
        }
    };

    if digest_bytes == computed {
        Ok(())
    } else {
        Err(SamlError::DigestMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::Digest as _;

    #[test]
    fn test_parse_minimal_saml_response() {
        let xml = r#"<?xml version="1.0"?>
<Response xmlns="urn:oasis:names:tc:SAML:2.0:protocol">
  <Assertion xmlns="urn:oasis:names:tc:SAML:2.0:assertion" ID="_123" IssueInstant="2025-01-01T00:00:00Z">
    <Issuer>https://idp.example.com</Issuer>
    <Subject>
      <NameID Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress">user@example.com</NameID>
    </Subject>
    <AttributeStatement>
      <Attribute Name="email" NameFormat="urn:oasis:names:tc:SAML:2.0:attrname-format:basic">
        <AttributeValue>user@example.com</AttributeValue>
      </Attribute>
      <Attribute Name="name" NameFormat="urn:oasis:names:tc:SAML:2.0:attrname-format:basic">
        <AttributeValue>Test User</AttributeValue>
      </Attribute>
    </AttributeStatement>
  </Assertion>
</Response>"#;

        let assertion = parse_saml_xml(xml).unwrap();
        assert_eq!(assertion.assertion_id, "_123");
        assert_eq!(assertion.issuer, "https://idp.example.com");
        assert_eq!(assertion.subject_name_id, "user@example.com");
        assert_eq!(assertion.issue_instant, Some("2025-01-01T00:00:00Z".into()));
        assert_eq!(assertion.attributes.len(), 2);
    }

    #[test]
    fn test_extract_element_text() {
        let xml = "<Issuer>https://idp.example.com</Issuer>";
        assert_eq!(
            extract_element_text(xml, "Issuer"),
            Some("https://idp.example.com".into())
        );
    }

    #[test]
    fn test_extract_element_text_missing() {
        let xml = "<Foo>bar</Foo>";
        assert!(extract_element_text(xml, "Issuer").is_none());
    }

    #[test]
    fn test_extract_xml_attr() {
        let xml = r#"<Assertion ID="_abc123" IssueInstant="2025-01-01T00:00:00Z">"#;
        assert_eq!(
            extract_xml_attr(xml, "Assertion", "ID"),
            Some("_abc123".into())
        );
    }

    #[test]
    fn test_extract_xml_attr_missing() {
        let xml = "<Assertion ID='test'>";
        assert!(extract_xml_attr(xml, "Assertion", "Missing").is_none());
    }

    #[test]
    fn test_parse_saml_response_base64() {
        let xml = r#"<?xml version="1.0"?>
<Response xmlns="urn:oasis:names:tc:SAML:2.0:protocol">
  <Assertion xmlns="urn:oasis:names:tc:SAML:2.0:assertion" ID="_456" IssueInstant="2025-06-04T00:00:00Z">
    <Issuer>https://idp.test.com</Issuer>
    <Subject><NameID>admin@test.com</NameID></Subject>
  </Assertion>
</Response>"#;
        let encoded = base64::engine::general_purpose::STANDARD.encode(xml.as_bytes());
        let assertion = parse_saml_response(&encoded).unwrap();
        assert_eq!(assertion.assertion_id, "_456");
        assert_eq!(assertion.issuer, "https://idp.test.com");
    }

    #[test]
    fn test_parse_invalid_base64() {
        let result = parse_saml_response("!!!not-base64!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_digest_sha256() {
        let data = b"<test>hello</test>";
        let digest = sha2::Sha256::digest(data);
        let encoded = base64::engine::general_purpose::STANDARD.encode(digest);
        assert!(verify_digest("SHA-256", &encoded, data).is_ok());
    }

    #[test]
    fn test_verify_digest_mismatch() {
        assert!(verify_digest("SHA-256", "AAAA", b"test data").is_err());
    }

    #[test]
    fn test_verify_digest_unsupported_algorithm() {
        assert!(verify_digest("MD5", "AAAA", b"test").is_err());
    }

    #[test]
    fn test_saml_error_display() {
        let err = SamlError::MissingElement("Issuer".into());
        assert!(err.to_string().contains("Issuer"));
    }

    #[test]
    fn test_saml_assertion_serialization() {
        let assertion = SamlAssertion {
            assertion_id: "_test".into(),
            issuer: "https://idp.com".into(),
            subject_name_id: "user@idp.com".into(),
            conditions_not_on_or_after: None,
            attributes: vec![SamlAttribute {
                name: "email".into(),
                name_format: Some("basic".into()),
                values: vec!["user@test.com".into()],
            }],
            issue_instant: Some("2025-01-01T00:00:00Z".into()),
        };
        let json = serde_json::to_string(&assertion).unwrap();
        assert!(json.contains("\"assertion_id\":\"_test\""));
    }
}
