#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DevicePosture {
    Managed,
    Unmanaged,
    Unknown,
}

impl fmt::Display for DevicePosture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Managed => write!(f, "managed"),
            Self::Unmanaged => write!(f, "unmanaged"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AbacSubject {
    pub id: String,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
    pub attributes: HashMap<String, serde_json::Value>,
}

impl AbacSubject {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            roles: Vec::new(),
            groups: Vec::new(),
            attributes: HashMap::new(),
        }
    }

    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }

    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.groups.push(group.into());
        self
    }

    pub fn with_attribute(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.attributes.insert(key.into(), value);
        self
    }
}

#[derive(Debug, Clone)]
pub struct AbacResource {
    pub id: String,
    pub type_: String,
    pub owner_id: Option<String>,
    pub attributes: HashMap<String, serde_json::Value>,
}

impl AbacResource {
    pub fn new(id: impl Into<String>, type_: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            type_: type_.into(),
            owner_id: None,
            attributes: HashMap::new(),
        }
    }

    pub fn with_owner(mut self, owner_id: impl Into<String>) -> Self {
        self.owner_id = Some(owner_id.into());
        self
    }

    pub fn with_attribute(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.attributes.insert(key.into(), value);
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct AbacEnvironment {
    pub time_of_day: Option<String>,
    pub day_of_week: Option<String>,
    pub ip_address: Option<String>,
    pub device_posture: Option<DevicePosture>,
    pub location: Option<String>,
}

impl AbacEnvironment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_time_of_day(mut self, time: impl Into<String>) -> Self {
        self.time_of_day = Some(time.into());
        self
    }

    pub fn with_day_of_week(mut self, day: impl Into<String>) -> Self {
        self.day_of_week = Some(day.into());
        self
    }

    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    pub fn with_device_posture(mut self, posture: DevicePosture) -> Self {
        self.device_posture = Some(posture);
        self
    }

    pub fn with_location(mut self, loc: impl Into<String>) -> Self {
        self.location = Some(loc.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionType {
    RoleMatch,
    GroupMatch,
    AttributeEquals,
    AttributeContains,
    IpInRange,
    TimeInRange,
    DayOfWeekMatch,
    GeoMatch,
    DevicePostureMatch,
    ResourceOwnerMatch,
}

impl fmt::Display for ConditionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RoleMatch => write!(f, "role_match"),
            Self::GroupMatch => write!(f, "group_match"),
            Self::AttributeEquals => write!(f, "attribute_equals"),
            Self::AttributeContains => write!(f, "attribute_contains"),
            Self::IpInRange => write!(f, "ip_in_range"),
            Self::TimeInRange => write!(f, "time_in_range"),
            Self::DayOfWeekMatch => write!(f, "day_of_week_match"),
            Self::GeoMatch => write!(f, "geo_match"),
            Self::DevicePostureMatch => write!(f, "device_posture_match"),
            Self::ResourceOwnerMatch => write!(f, "resource_owner_match"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCondition {
    pub condition_type: ConditionType,
    pub parameter: String,
    pub value: serde_json::Value,
}

impl PolicyCondition {
    pub fn new(
        condition_type: ConditionType,
        parameter: impl Into<String>,
        value: serde_json::Value,
    ) -> Self {
        Self {
            condition_type,
            parameter: parameter.into(),
            value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AbacContext {
    pub subject: AbacSubject,
    pub resource: AbacResource,
    pub action: String,
    pub environment: AbacEnvironment,
}

impl AbacContext {
    pub fn new(
        subject: AbacSubject,
        resource: AbacResource,
        action: impl Into<String>,
        environment: AbacEnvironment,
    ) -> Self {
        Self {
            subject,
            resource,
            action: action.into(),
            environment,
        }
    }
}

pub fn evaluate_condition(condition: &PolicyCondition, context: &AbacContext) -> bool {
    match condition.condition_type {
        ConditionType::RoleMatch => context
            .subject
            .roles
            .iter()
            .any(|r| r == &condition.parameter),
        ConditionType::GroupMatch => context
            .subject
            .groups
            .iter()
            .any(|g| g == &condition.parameter),
        ConditionType::AttributeEquals => {
            if let Some(v) = context.subject.attributes.get(&condition.parameter) {
                *v == condition.value
            } else if let Some(v) = context.resource.attributes.get(&condition.parameter) {
                *v == condition.value
            } else {
                false
            }
        }
        ConditionType::AttributeContains => {
            if let Some(v) = context.subject.attributes.get(&condition.parameter) {
                if let Some(arr) = v.as_array() {
                    arr.contains(&condition.value)
                } else {
                    v.to_string().contains(&condition.value.to_string())
                }
            } else {
                false
            }
        }
        ConditionType::IpInRange => {
            if let Some(ref ip_str) = context.environment.ip_address {
                match_parse_ip_in_range(ip_str, &condition.parameter, &condition.value)
            } else {
                false
            }
        }
        ConditionType::TimeInRange => {
            if let Some(ref time) = context.environment.time_of_day {
                is_time_in_range(time, &condition.parameter, &condition.value)
            } else {
                false
            }
        }
        ConditionType::DayOfWeekMatch => {
            if let Some(ref day) = context.environment.day_of_week {
                match_day_of_week(day, &condition.value)
            } else {
                false
            }
        }
        ConditionType::GeoMatch => {
            if let Some(ref loc) = context.environment.location {
                loc == &condition.parameter
            } else {
                false
            }
        }
        ConditionType::DevicePostureMatch => {
            if let Some(ref posture) = context.environment.device_posture {
                posture.to_string() == condition.parameter
            } else {
                false
            }
        }
        ConditionType::ResourceOwnerMatch => {
            context.resource.owner_id.as_ref() == Some(&context.subject.id)
        }
    }
}

fn match_parse_ip_in_range(
    ip_str: &str,
    _range_type: &str,
    range_value: &serde_json::Value,
) -> bool {
    if range_value.is_string() {
        let cidr = range_value.as_str().unwrap_or("");
        if let Ok(addr) = ip_str.parse::<IpAddr>() {
            return ip_in_cidr(addr, cidr);
        }
    }
    if let Some(arr) = range_value.as_array() {
        let allowed: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
        return allowed.contains(&ip_str);
    }
    false
}

fn ip_in_cidr(ip: IpAddr, cidr: &str) -> bool {
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return false;
    }
    let network: IpAddr = match parts[0].parse() {
        Ok(a) => a,
        Err(_) => return false,
    };
    let prefix_len: u32 = match parts[1].parse() {
        Ok(n) => n,
        Err(_) => return false,
    };

    match (ip, network) {
        (IpAddr::V4(ip_v4), IpAddr::V4(net_v4)) => {
            if prefix_len > 32 {
                return false;
            }
            let ip_bits = u32::from(ip_v4);
            let net_bits = u32::from(net_v4);
            let mask = if prefix_len == 0 {
                0u32
            } else {
                !0u32 << (32 - prefix_len)
            };
            (ip_bits & mask) == (net_bits & mask)
        }
        _ => false,
    }
}

fn is_time_in_range(time: &str, _range_type: &str, range_value: &serde_json::Value) -> bool {
    if let Some(arr) = range_value.as_array() {
        if arr.len() >= 2 {
            let start = arr[0].as_str().unwrap_or("");
            let end = arr[1].as_str().unwrap_or("");
            return time >= start && time <= end;
        }
    }
    false
}

fn match_day_of_week(day: &str, expected: &serde_json::Value) -> bool {
    if let Some(arr) = expected.as_array() {
        let days: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
        return days.contains(&day);
    }
    if let Some(s) = expected.as_str() {
        return day == s;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::too_many_arguments)]
    fn make_context(
        roles: Vec<&str>,
        groups: Vec<&str>,
        resource_owner: Option<&str>,
        ip: Option<&str>,
        device: Option<DevicePosture>,
        location: Option<&str>,
        time: Option<&str>,
        day: Option<&str>,
    ) -> AbacContext {
        let mut subject = AbacSubject::new("user-1");
        for r in roles {
            subject = subject.with_role(r);
        }
        for g in groups {
            subject = subject.with_group(g);
        }

        let mut resource = AbacResource::new("resource-1", "document");
        if let Some(owner) = resource_owner {
            resource = resource.with_owner(owner);
        }

        let mut env = AbacEnvironment::new();
        if let Some(i) = ip {
            env = env.with_ip(i);
        }
        if let Some(d) = device {
            env = env.with_device_posture(d);
        }
        if let Some(l) = location {
            env = env.with_location(l);
        }
        if let Some(t) = time {
            env = env.with_time_of_day(t);
        }
        if let Some(d) = day {
            env = env.with_day_of_week(d);
        }

        AbacContext::new(subject, resource, "read", env)
    }

    #[test]
    fn test_role_match() {
        let ctx = make_context(vec!["admin"], vec![], None, None, None, None, None, None);
        let cond = PolicyCondition::new(ConditionType::RoleMatch, "admin", serde_json::Value::Null);
        assert!(evaluate_condition(&cond, &ctx));

        let cond2 =
            PolicyCondition::new(ConditionType::RoleMatch, "guest", serde_json::Value::Null);
        assert!(!evaluate_condition(&cond2, &ctx));
    }

    #[test]
    fn test_group_match() {
        let ctx = make_context(
            vec![],
            vec!["engineering"],
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let cond = PolicyCondition::new(
            ConditionType::GroupMatch,
            "engineering",
            serde_json::Value::Null,
        );
        assert!(evaluate_condition(&cond, &ctx));
    }

    #[test]
    fn test_attribute_equals() {
        let subject = AbacSubject::new("user-1").with_attribute(
            "department",
            serde_json::Value::String("engineering".into()),
        );
        let resource = AbacResource::new("r1", "doc")
            .with_attribute("sensitivity", serde_json::Value::String("high".into()));
        let ctx = AbacContext::new(subject, resource, "read", AbacEnvironment::new());

        let cond = PolicyCondition::new(
            ConditionType::AttributeEquals,
            "department",
            serde_json::json!("engineering"),
        );
        assert!(evaluate_condition(&cond, &ctx));

        let cond2 = PolicyCondition::new(
            ConditionType::AttributeEquals,
            "sensitivity",
            serde_json::json!("high"),
        );
        assert!(evaluate_condition(&cond2, &ctx));
    }

    #[test]
    fn test_attribute_contains() {
        let subject = AbacSubject::new("user-1")
            .with_attribute("tags", serde_json::json!(["rust", "crypto", "security"]));
        let resource = AbacResource::new("r1", "doc");
        let ctx = AbacContext::new(subject, resource, "read", AbacEnvironment::new());

        let cond = PolicyCondition::new(
            ConditionType::AttributeContains,
            "tags",
            serde_json::json!("crypto"),
        );
        assert!(evaluate_condition(&cond, &ctx));

        let cond2 = PolicyCondition::new(
            ConditionType::AttributeContains,
            "tags",
            serde_json::json!("python"),
        );
        assert!(!evaluate_condition(&cond2, &ctx));
    }

    #[test]
    fn test_ip_in_range_list() {
        let ctx = make_context(
            vec![],
            vec![],
            None,
            Some("10.0.0.1"),
            None,
            None,
            None,
            None,
        );
        let cond = PolicyCondition::new(
            ConditionType::IpInRange,
            "allowlist",
            serde_json::json!(["10.0.0.1", "10.0.0.2"]),
        );
        assert!(evaluate_condition(&cond, &ctx));

        let cond2 = PolicyCondition::new(
            ConditionType::IpInRange,
            "allowlist",
            serde_json::json!(["192.168.1.1"]),
        );
        assert!(!evaluate_condition(&cond2, &ctx));
    }

    #[test]
    fn test_ip_in_cidr() {
        let ctx = make_context(
            vec![],
            vec![],
            None,
            Some("10.0.0.5"),
            None,
            None,
            None,
            None,
        );
        let cond = PolicyCondition::new(
            ConditionType::IpInRange,
            "subnet",
            serde_json::json!("10.0.0.0/24"),
        );
        assert!(evaluate_condition(&cond, &ctx));

        let cond2 = PolicyCondition::new(
            ConditionType::IpInRange,
            "subnet",
            serde_json::json!("192.168.0.0/16"),
        );
        assert!(!evaluate_condition(&cond2, &ctx));
    }

    #[test]
    fn test_time_in_range() {
        let ctx = make_context(vec![], vec![], None, None, None, None, Some("14:30"), None);
        let cond = PolicyCondition::new(
            ConditionType::TimeInRange,
            "business_hours",
            serde_json::json!(["09:00", "17:00"]),
        );
        assert!(evaluate_condition(&cond, &ctx));

        let ctx2 = make_context(vec![], vec![], None, None, None, None, Some("20:00"), None);
        assert!(!evaluate_condition(&cond, &ctx2));
    }

    #[test]
    fn test_day_of_week_match() {
        let ctx = make_context(vec![], vec![], None, None, None, None, None, Some("Monday"));
        let cond = PolicyCondition::new(
            ConditionType::DayOfWeekMatch,
            "weekdays",
            serde_json::json!(["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"]),
        );
        assert!(evaluate_condition(&cond, &ctx));

        let cond2 = PolicyCondition::new(
            ConditionType::DayOfWeekMatch,
            "weekdays",
            serde_json::json!("Saturday"),
        );
        assert!(!evaluate_condition(&cond2, &ctx));
    }

    #[test]
    fn test_geo_match() {
        let ctx = make_context(vec![], vec![], None, None, None, Some("US"), None, None);
        let cond = PolicyCondition::new(ConditionType::GeoMatch, "US", serde_json::Value::Null);
        assert!(evaluate_condition(&cond, &ctx));

        let cond2 = PolicyCondition::new(ConditionType::GeoMatch, "EU", serde_json::Value::Null);
        assert!(!evaluate_condition(&cond2, &ctx));
    }

    #[test]
    fn test_device_posture_match() {
        let ctx = make_context(
            vec![],
            vec![],
            None,
            None,
            Some(DevicePosture::Managed),
            None,
            None,
            None,
        );
        let cond = PolicyCondition::new(
            ConditionType::DevicePostureMatch,
            "managed",
            serde_json::Value::Null,
        );
        assert!(evaluate_condition(&cond, &ctx));

        let ctx2 = make_context(
            vec![],
            vec![],
            None,
            None,
            Some(DevicePosture::Unmanaged),
            None,
            None,
            None,
        );
        assert!(!evaluate_condition(&cond, &ctx2));
    }

    #[test]
    fn test_resource_owner_match() {
        let ctx = make_context(vec![], vec![], Some("user-1"), None, None, None, None, None);
        let cond = PolicyCondition::new(
            ConditionType::ResourceOwnerMatch,
            "",
            serde_json::Value::Null,
        );
        assert!(evaluate_condition(&cond, &ctx));

        let ctx2 = make_context(
            vec![],
            vec![],
            Some("other-user"),
            None,
            None,
            None,
            None,
            None,
        );
        assert!(!evaluate_condition(&cond, &ctx2));
    }
}
