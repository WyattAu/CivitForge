#![forbid(unsafe_code)]

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "civitforge.dev",
    version = "v1alpha1",
    kind = "CivitForgeApp",
    namespaced,
    singular = "civitforgeapp",
    plural = "civitforgeapps",
    status = "CivitForgeAppStatus",
    printcolumn = "name=Phase,type=string,JSONPath=.status.phase",
    printcolumn = "name=Ready,type=integer,JSONPath=.status.readyReplicas"
)]
pub struct CivitForgeAppSpec {
    #[serde(default = "default_replicas")]
    pub replicas: i32,

    pub image: String,

    #[serde(default = "default_tag")]
    pub tag: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_url: Option<String>,

    #[serde(default)]
    pub federation_enabled: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceRequirements>,

    #[serde(default)]
    pub components: Vec<CivitForgeAppComponent>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_selector: Option<BTreeMap<String, String>>,

    #[serde(default = "default_max_unavailable")]
    pub max_unavailable: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ResourceRequirements {
    #[serde(default)]
    pub cpu_request: String,

    #[serde(default)]
    pub cpu_limit: String,

    #[serde(default)]
    pub memory_request: String,

    #[serde(default)]
    pub memory_limit: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum CivitForgeAppComponent {
    Web,
    Runner,
    Brain,
    Registry,
}

impl CivitForgeAppComponent {
    pub fn deployment_name(&self) -> String {
        match self {
            Self::Web => "civitforge-web".to_string(),
            Self::Runner => "civitforge-runner".to_string(),
            Self::Brain => "civitforge-brain".to_string(),
            Self::Registry => "civitforge-registry".to_string(),
        }
    }

    pub fn service_name(&self) -> String {
        format!("{}-svc", self.deployment_name())
    }

    pub fn all() -> &'static [Self] {
        &[Self::Web, Self::Runner, Self::Brain, Self::Registry]
    }
}

impl std::fmt::Display for CivitForgeAppComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Web => write!(f, "Web"),
            Self::Runner => write!(f, "Runner"),
            Self::Brain => write!(f, "Brain"),
            Self::Registry => write!(f, "Registry"),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct CivitForgeAppStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,

    #[serde(default)]
    pub replicas: i32,

    #[serde(default)]
    pub ready_replicas: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    #[serde(default)]
    pub conditions: Vec<ComponentCondition>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ComponentCondition {
    pub component: CivitForgeAppComponent,
    pub status: ConditionStatus,
    pub reason: String,
    pub message: String,
    pub last_transition_time: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ConditionStatus {
    True,
    False,
    Unknown,
}

fn default_replicas() -> i32 {
    3
}

fn default_tag() -> String {
    "latest".to_string()
}

fn default_max_unavailable() -> i32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_spec() -> CivitForgeAppSpec {
        let mut node_selector = BTreeMap::new();
        node_selector.insert("role".to_string(), "app".to_string());

        CivitForgeAppSpec {
            replicas: 5,
            image: "civitforge/civitforge:latest".to_string(),
            tag: "1.0.0".to_string(),
            database_url: Some("postgres://db:5432/civit".to_string()),
            redis_url: Some("redis://cache:6379".to_string()),
            federation_enabled: true,
            resources: Some(ResourceRequirements {
                cpu_request: "100m".to_string(),
                cpu_limit: "500m".to_string(),
                memory_request: "128Mi".to_string(),
                memory_limit: "512Mi".to_string(),
            }),
            components: vec![CivitForgeAppComponent::Web, CivitForgeAppComponent::Brain],
            node_selector: Some(node_selector),
            max_unavailable: 2,
        }
    }

    #[test]
    fn test_default_values() {
        assert_eq!(default_replicas(), 3);
        assert_eq!(default_tag(), "latest");
        assert_eq!(default_max_unavailable(), 1);
    }

    #[test]
    fn test_spec_serialization_roundtrip() {
        let spec = full_spec();
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("civitforge/civitforge"));
        assert!(json.contains("1.0.0"));
        let de: CivitForgeAppSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(de.replicas, 5);
        assert_eq!(de.image, "civitforge/civitforge:latest");
        assert_eq!(de.tag, "1.0.0");
        assert!(de.federation_enabled);
        assert_eq!(de.max_unavailable, 2);
    }

    #[test]
    fn test_spec_optional_fields_skipped_on_serialization() {
        let spec = CivitForgeAppSpec {
            image: "test".into(),
            tag: String::new(),
            replicas: 0,
            max_unavailable: 0,
            database_url: None,
            redis_url: None,
            federation_enabled: false,
            resources: None,
            components: vec![],
            node_selector: None,
        };
        let json = serde_json::to_string(&spec).unwrap();
        assert!(!json.contains("database_url"));
        assert!(!json.contains("redis_url"));
        assert!(!json.contains("resources"));
        assert!(!json.contains("node_selector"));
    }

    #[test]
    fn test_spec_optional_fields_included_when_present() {
        let spec = full_spec();
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("database_url"));
        assert!(json.contains("redis_url"));
        assert!(json.contains("resources"));
        assert!(json.contains("node_selector"));
        assert!(json.contains("federation_enabled"));
    }

    #[test]
    fn test_spec_node_selector_roundtrip() {
        let spec = full_spec();
        let json = serde_json::to_string(&spec).unwrap();
        let de: CivitForgeAppSpec = serde_json::from_str(&json).unwrap();
        let ns = de.node_selector.unwrap();
        assert_eq!(ns.get("role").unwrap(), "app");
    }

    #[test]
    fn test_spec_components_roundtrip() {
        let spec = full_spec();
        let json = serde_json::to_string(&spec).unwrap();
        let de: CivitForgeAppSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(de.components.len(), 2);
        assert_eq!(de.components[0], CivitForgeAppComponent::Web);
        assert_eq!(de.components[1], CivitForgeAppComponent::Brain);
    }

    #[test]
    fn test_component_enum_all_variants_serialize() {
        let variants = [
            CivitForgeAppComponent::Web,
            CivitForgeAppComponent::Runner,
            CivitForgeAppComponent::Brain,
            CivitForgeAppComponent::Registry,
        ];
        for comp in &variants {
            let json = serde_json::to_string(comp).unwrap();
            let de: CivitForgeAppComponent = serde_json::from_str(&json).unwrap();
            assert_eq!(*comp, de);
        }
    }

    #[test]
    fn test_component_deployment_names() {
        assert_eq!(
            CivitForgeAppComponent::Web.deployment_name(),
            "civitforge-web"
        );
        assert_eq!(
            CivitForgeAppComponent::Runner.deployment_name(),
            "civitforge-runner"
        );
        assert_eq!(
            CivitForgeAppComponent::Brain.deployment_name(),
            "civitforge-brain"
        );
        assert_eq!(
            CivitForgeAppComponent::Registry.deployment_name(),
            "civitforge-registry"
        );
    }

    #[test]
    fn test_component_service_names() {
        assert_eq!(
            CivitForgeAppComponent::Web.service_name(),
            "civitforge-web-svc"
        );
        assert_eq!(
            CivitForgeAppComponent::Runner.service_name(),
            "civitforge-runner-svc"
        );
        assert_eq!(
            CivitForgeAppComponent::Brain.service_name(),
            "civitforge-brain-svc"
        );
        assert_eq!(
            CivitForgeAppComponent::Registry.service_name(),
            "civitforge-registry-svc"
        );
    }

    #[test]
    fn test_component_all() {
        let all = CivitForgeAppComponent::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&CivitForgeAppComponent::Web));
        assert!(all.contains(&CivitForgeAppComponent::Runner));
        assert!(all.contains(&CivitForgeAppComponent::Brain));
        assert!(all.contains(&CivitForgeAppComponent::Registry));
    }

    #[test]
    fn test_component_display() {
        assert_eq!(format!("{}", CivitForgeAppComponent::Web), "Web");
        assert_eq!(format!("{}", CivitForgeAppComponent::Registry), "Registry");
    }

    #[test]
    fn test_status_default() {
        let status = CivitForgeAppStatus::default();
        assert!(status.phase.is_none());
        assert_eq!(status.replicas, 0);
        assert_eq!(status.ready_replicas, 0);
        assert!(status.version.is_none());
        assert!(status.conditions.is_empty());
        assert!(status.last_updated.is_none());
    }

    #[test]
    fn test_status_serialization_roundtrip() {
        let status = CivitForgeAppStatus {
            phase: Some("Running".to_string()),
            replicas: 3,
            ready_replicas: 2,
            version: Some("1.0.0".to_string()),
            conditions: vec![ComponentCondition {
                component: CivitForgeAppComponent::Web,
                status: ConditionStatus::True,
                reason: "MinimumReplicasAvailable".to_string(),
                message: "Deployment has minimum availability".to_string(),
                last_transition_time: "2025-01-01T00:00:00Z".to_string(),
            }],
            last_updated: Some("2025-01-01T00:00:00Z".to_string()),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Running"));
        assert!(json.contains("MinimumReplicasAvailable"));
        let de: CivitForgeAppStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(de.phase.as_deref(), Some("Running"));
        assert_eq!(de.replicas, 3);
        assert_eq!(de.ready_replicas, 2);
        assert_eq!(de.conditions.len(), 1);
        assert_eq!(de.version.as_deref(), Some("1.0.0"));
    }

    #[test]
    fn test_status_optional_fields_skipped() {
        let status = CivitForgeAppStatus::default();
        let json = serde_json::to_string(&status).unwrap();
        assert!(!json.contains("phase"));
        assert!(!json.contains("version"));
        assert!(!json.contains("last_updated"));
    }

    #[test]
    fn test_condition_status_all_variants_serialize() {
        for status in [
            ConditionStatus::True,
            ConditionStatus::False,
            ConditionStatus::Unknown,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let de: ConditionStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, de);
        }
    }

    #[test]
    fn test_condition_status_equality() {
        assert_eq!(ConditionStatus::True, ConditionStatus::True);
        assert_ne!(ConditionStatus::True, ConditionStatus::False);
        assert_ne!(ConditionStatus::False, ConditionStatus::Unknown);
    }

    #[test]
    fn test_resource_requirements_serialization() {
        let rr = ResourceRequirements {
            cpu_request: "100m".into(),
            cpu_limit: "500m".into(),
            memory_request: "128Mi".into(),
            memory_limit: "512Mi".into(),
        };
        let json = serde_json::to_string(&rr).unwrap();
        let de: ResourceRequirements = serde_json::from_str(&json).unwrap();
        assert_eq!(de.cpu_request, "100m");
        assert_eq!(de.cpu_limit, "500m");
        assert_eq!(de.memory_request, "128Mi");
        assert_eq!(de.memory_limit, "512Mi");
    }

    #[test]
    fn test_spec_with_empty_components_defaults_to_all() {
        let spec = CivitForgeAppSpec {
            image: "test".into(),
            tag: String::new(),
            replicas: 0,
            max_unavailable: 0,
            database_url: None,
            redis_url: None,
            federation_enabled: false,
            resources: None,
            components: vec![],
            node_selector: None,
        };
        assert!(spec.components.is_empty());
        let all = CivitForgeAppComponent::all();
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn test_crd_json_schema_generation() {
        let _schema = schemars::r#gen::SchemaSettings::openapi3()
            .with(|s| {
                s.inline_subschemas = true;
                s.meta_schema = None;
            })
            .into_generator()
            .into_root_schema_for::<CivitForgeAppSpec>();
    }
}
