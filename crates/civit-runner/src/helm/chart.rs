#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HelmChart {
    pub api_version: String,
    pub name: String,
    pub version: String,
    pub app_version: String,
    pub description: String,
    pub type_: ChartType,
    pub keywords: Vec<String>,
    pub home: String,
    pub sources: Vec<String>,
    pub maintainers: Vec<HelmMaintainer>,
    pub values: HelmValues,
    pub templates: Vec<HelmTemplate>,
    pub dependencies: Vec<HelmDependency>,
}

impl PartialEq for HelmChart {
    fn eq(&self, other: &Self) -> bool {
        self.api_version == other.api_version
            && self.name == other.name
            && self.version == other.version
            && self.app_version == other.app_version
            && self.description == other.description
            && self.type_ == other.type_
            && self.keywords == other.keywords
            && self.home == other.home
            && self.sources == other.sources
            && self.maintainers == other.maintainers
            && self.templates == other.templates
            && self.dependencies == other.dependencies
            && self.values.replicas == other.values.replicas
            && self.values.image == other.values.image
            && self.values.image_pull_policy == other.values.image_pull_policy
            && self.values.service_type == other.values.service_type
            && self.values.ingress_enabled == other.values.ingress_enabled
            && self.values.tolerations == other.values.tolerations
    }
}

impl Eq for HelmChart {}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ChartType {
    Application,
    Library,
}

impl fmt::Display for ChartType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChartType::Application => write!(f, "application"),
            ChartType::Library => write!(f, "library"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HelmMaintainer {
    pub name: String,
    pub email: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HelmValues {
    pub replicas: u32,
    pub image: String,
    pub image_pull_policy: String,
    pub resources: ContainerResources,
    pub node_selector: Option<HashMap<String, String>>,
    pub tolerations: Vec<HelmValuesToleration>,
    pub affinity: Option<HashMap<String, serde_json::Value>>,
    pub service_type: String,
    pub ingress_enabled: bool,
    pub persistence: PersistenceConfig,
}

impl Default for HelmValues {
    fn default() -> Self {
        Self {
            replicas: 3,
            image: "civitforge/app:latest".to_string(),
            image_pull_policy: "IfNotPresent".to_string(),
            resources: ContainerResources::default(),
            node_selector: None,
            tolerations: vec![],
            affinity: None,
            service_type: "ClusterIP".to_string(),
            ingress_enabled: false,
            persistence: PersistenceConfig::default(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerResources {
    pub requests: ResourceLimits,
    pub limits: ResourceLimits,
}

impl Default for ContainerResources {
    fn default() -> Self {
        Self {
            requests: ResourceLimits {
                cpu: "100m".to_string(),
                memory: "128Mi".to_string(),
            },
            limits: ResourceLimits {
                cpu: "500m".to_string(),
                memory: "512Mi".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResourceLimits {
    pub cpu: String,
    pub memory: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersistenceConfig {
    pub enabled: bool,
    pub size: String,
    pub storage_class: Option<String>,
    pub access_mode: String,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            size: "1Gi".to_string(),
            storage_class: None,
            access_mode: "ReadWriteOnce".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HelmValuesToleration {
    pub key: String,
    pub operator: HelmValuesTolerationOperator,
    pub effect: HelmValuesTolerationEffect,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HelmValuesTolerationOperator {
    Exists,
    Equal,
}

impl fmt::Display for HelmValuesTolerationOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HelmValuesTolerationOperator::Exists => write!(f, "Exists"),
            HelmValuesTolerationOperator::Equal => write!(f, "Equal"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HelmValuesTolerationEffect {
    NoSchedule,
    PreferNoSchedule,
    NoExecute,
}

impl fmt::Display for HelmValuesTolerationEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HelmValuesTolerationEffect::NoSchedule => write!(f, "NoSchedule"),
            HelmValuesTolerationEffect::PreferNoSchedule => write!(f, "PreferNoSchedule"),
            HelmValuesTolerationEffect::NoExecute => write!(f, "NoExecute"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TemplateKind {
    Deployment,
    Service,
    Ingress,
    ConfigMap,
    Secret,
    NetworkPolicy,
    HPA,
}

impl fmt::Display for TemplateKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateKind::Deployment => write!(f, "Deployment"),
            TemplateKind::Service => write!(f, "Service"),
            TemplateKind::Ingress => write!(f, "Ingress"),
            TemplateKind::ConfigMap => write!(f, "ConfigMap"),
            TemplateKind::Secret => write!(f, "Secret"),
            TemplateKind::NetworkPolicy => write!(f, "NetworkPolicy"),
            TemplateKind::HPA => write!(f, "HorizontalPodAutoscaler"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HelmTemplate {
    pub name: String,
    pub kind: TemplateKind,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HelmDependency {
    pub name: String,
    pub version: String,
    pub repository: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HelmUpgradeStrategy {
    pub max_unavailable: String,
    pub max_surge: String,
    pub rollback_history: u32,
    pub rollback_on_failure: bool,
}

impl Default for HelmUpgradeStrategy {
    fn default() -> Self {
        Self {
            max_unavailable: "25%".to_string(),
            max_surge: "25%".to_string(),
            rollback_history: 10,
            rollback_on_failure: true,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HelmNetworkPolicy {
    pub pod_selector: HashMap<String, String>,
    pub ingress_rules: Vec<NetworkPolicyRule>,
    pub egress_rules: Vec<NetworkPolicyRule>,
    pub policy_types: Vec<NetworkPolicyType>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NetworkPolicyType {
    Ingress,
    Egress,
}

impl fmt::Display for NetworkPolicyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkPolicyType::Ingress => write!(f, "Ingress"),
            NetworkPolicyType::Egress => write!(f, "Egress"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkPolicyRule {
    pub ports: Vec<NetworkPolicyPort>,
    pub from_: Vec<NetworkPolicyPeer>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NetworkPolicyPort {
    pub port: u16,
    pub protocol: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NetworkPolicyPeer {
    pub pod_selector: Option<HashMap<String, String>>,
    pub namespace_selector: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HelmHPA {
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub target_cpu_percent: Option<u32>,
    pub target_memory_percent: Option<u32>,
}

impl Default for HelmHPA {
    fn default() -> Self {
        Self {
            min_replicas: 3,
            max_replicas: 10,
            target_cpu_percent: Some(70),
            target_memory_percent: Some(80),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HelmProductionValues {
    pub replicas: u32,
    pub image_pull_policy: String,
    pub resources: ContainerResources,
    pub service_type: String,
    pub ingress_enabled: bool,
    pub persistence: PersistenceConfig,
    pub hpa: HelmHPA,
    pub upgrade_strategy: HelmUpgradeStrategy,
    pub network_policy: HelmNetworkPolicy,
    pub node_selector: HashMap<String, String>,
    pub tolerations: Vec<HelmValuesToleration>,
}

impl Default for HelmProductionValues {
    fn default() -> Self {
        Self {
            replicas: 5,
            image_pull_policy: "Always".to_string(),
            resources: ContainerResources {
                requests: ResourceLimits {
                    cpu: "250m".to_string(),
                    memory: "256Mi".to_string(),
                },
                limits: ResourceLimits {
                    cpu: "1000m".to_string(),
                    memory: "1Gi".to_string(),
                },
            },
            service_type: "LoadBalancer".to_string(),
            ingress_enabled: true,
            persistence: PersistenceConfig {
                enabled: true,
                size: "10Gi".to_string(),
                storage_class: Some("ssd".to_string()),
                access_mode: "ReadWriteOnce".to_string(),
            },
            hpa: HelmHPA {
                min_replicas: 5,
                max_replicas: 50,
                target_cpu_percent: Some(70),
                target_memory_percent: Some(80),
            },
            upgrade_strategy: HelmUpgradeStrategy {
                max_unavailable: "10%".to_string(),
                max_surge: "50%".to_string(),
                rollback_history: 20,
                rollback_on_failure: true,
            },
            network_policy: HelmNetworkPolicy {
                pod_selector: HashMap::from([("app".to_string(), "civitforge".to_string())]),
                ingress_rules: vec![],
                egress_rules: vec![],
                policy_types: vec![NetworkPolicyType::Ingress, NetworkPolicyType::Egress],
            },
            node_selector: HashMap::from([("nodepool".to_string(), "production".to_string())]),
            tolerations: vec![],
        }
    }
}

pub struct HelmChartBuilder {
    api_version: String,
    name: String,
    version: String,
    app_version: String,
    description: String,
    type_: ChartType,
    keywords: Vec<String>,
    home: String,
    sources: Vec<String>,
    maintainers: Vec<HelmMaintainer>,
    values: HelmValues,
    templates: Vec<HelmTemplate>,
    dependencies: Vec<HelmDependency>,
}

impl HelmChartBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            api_version: "v2".to_string(),
            name: name.to_string(),
            version: "0.1.0".to_string(),
            app_version: "0.1.0".to_string(),
            description: String::new(),
            type_: ChartType::Application,
            keywords: vec![],
            home: String::new(),
            sources: vec![],
            maintainers: vec![],
            values: HelmValues::default(),
            templates: vec![],
            dependencies: vec![],
        }
    }

    pub fn version(mut self, ver: &str) -> Self {
        self.version = ver.to_string();
        self
    }

    pub fn app_version(mut self, ver: &str) -> Self {
        self.app_version = ver.to_string();
        self
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn chart_type(mut self, type_: ChartType) -> Self {
        self.type_ = type_;
        self
    }

    pub fn with_values(mut self, vals: HelmValues) -> Self {
        self.values = vals;
        self
    }

    pub fn add_template(mut self, tmpl: HelmTemplate) -> Self {
        self.templates.push(tmpl);
        self
    }

    pub fn add_dependency(mut self, dep: HelmDependency) -> Self {
        self.dependencies.push(dep);
        self
    }

    pub fn add_keyword(mut self, keyword: &str) -> Self {
        self.keywords.push(keyword.to_string());
        self
    }

    pub fn add_maintainer(mut self, maintainer: HelmMaintainer) -> Self {
        self.maintainers.push(maintainer);
        self
    }

    pub fn add_source(mut self, source: &str) -> Self {
        self.sources.push(source.to_string());
        self
    }

    pub fn home(mut self, home: &str) -> Self {
        self.home = home.to_string();
        self
    }

    pub fn build(self) -> HelmChart {
        HelmChart {
            api_version: self.api_version,
            name: self.name,
            version: self.version,
            app_version: self.app_version,
            description: self.description,
            type_: self.type_,
            keywords: self.keywords,
            home: self.home,
            sources: self.sources,
            maintainers: self.maintainers,
            values: self.values,
            templates: self.templates,
            dependencies: self.dependencies,
        }
    }
}

pub struct HelmChartRenderer;

impl HelmChartRenderer {
    pub fn render_chart(chart: &HelmChart) -> String {
        let mut lines = vec![];

        lines.push(format!("apiVersion: {}", chart.api_version));
        lines.push(format!("name: {}", chart.name));
        lines.push(format!("version: {}", chart.version));
        lines.push(format!("appVersion: {}", chart.app_version));
        if !chart.description.is_empty() {
            lines.push(format!("description: {}", chart.description));
        }
        lines.push(format!("type: {}", chart.type_));
        if !chart.keywords.is_empty() {
            lines.push(format!("keywords: [{}]", chart.keywords.join(", ")));
        }
        if !chart.home.is_empty() {
            lines.push(format!("home: {}", chart.home));
        }
        if !chart.sources.is_empty() {
            lines.push("sources:".to_string());
            for src in &chart.sources {
                lines.push(format!("  - {src}"));
            }
        }
        if !chart.maintainers.is_empty() {
            lines.push("maintainers:".to_string());
            for m in &chart.maintainers {
                lines.push(format!("  - name: {}", m.name));
                lines.push(format!("    email: {}", m.email));
                if let Some(url) = &m.url {
                    lines.push(format!("    url: {url}"));
                }
            }
        }
        if !chart.dependencies.is_empty() {
            lines.push("dependencies:".to_string());
            for dep in &chart.dependencies {
                lines.push(format!("  - name: {}", dep.name));
                lines.push(format!("    version: {}", dep.version));
                lines.push(format!("    repository: {}", dep.repository));
                if let Some(alias) = &dep.alias {
                    lines.push(format!("    alias: {alias}"));
                }
            }
        }

        lines.push(String::new());
        lines.push("---".to_string());
        lines.push(Self::render_values(&chart.values));

        lines.join("\n")
    }

    pub fn render_values(values: &HelmValues) -> String {
        let mut lines = vec![];
        lines.push("replicas: {}".to_string());
        lines.push(format!("  {}", values.replicas));
        lines.push(format!("image: {}", values.image));
        lines.push(format!("imagePullPolicy: {}", values.image_pull_policy));
        lines.push("resources:".to_string());
        lines.push("  requests:".to_string());
        lines.push(format!("    cpu: {}", values.resources.requests.cpu));
        lines.push(format!("    memory: {}", values.resources.requests.memory));
        lines.push("  limits:".to_string());
        lines.push(format!("    cpu: {}", values.resources.limits.cpu));
        lines.push(format!("    memory: {}", values.resources.limits.memory));
        if let Some(ns) = &values.node_selector {
            lines.push("nodeSelector:".to_string());
            for (k, v) in ns {
                lines.push(format!("  {k}: {v}"));
            }
        }
        if !values.tolerations.is_empty() {
            lines.push("tolerations:".to_string());
            for t in &values.tolerations {
                lines.push("  - key: {}".to_string());
                lines.push(format!("    {}", t.key));
                lines.push(format!("    operator: {}", t.operator));
                if let Some(val) = &t.value {
                    lines.push(format!("    value: {val}"));
                }
                lines.push(format!("    effect: {}", t.effect));
            }
        }
        lines.push(format!("serviceType: {}", values.service_type));
        lines.push("ingress:".to_string());
        lines.push(format!("  enabled: {}", values.ingress_enabled));
        lines.push("persistence:".to_string());
        lines.push(format!("  enabled: {}", values.persistence.enabled));
        lines.push(format!("  size: {}", values.persistence.size));
        if let Some(sc) = &values.persistence.storage_class {
            lines.push(format!("  storageClass: {sc}"));
        }
        lines.push(format!("  accessMode: {}", values.persistence.access_mode));
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helm_chart_builder_defaults() {
        let chart = HelmChartBuilder::new("myapp").build();
        assert_eq!(chart.name, "myapp");
        assert_eq!(chart.api_version, "v2");
        assert_eq!(chart.version, "0.1.0");
        assert_eq!(chart.type_, ChartType::Application);
        assert!(chart.keywords.is_empty());
        assert!(chart.dependencies.is_empty());
        assert!(chart.templates.is_empty());
    }

    #[test]
    fn test_helm_chart_builder_full() {
        let chart = HelmChartBuilder::new("myapp")
            .version("1.2.3")
            .app_version("1.2.3")
            .description("A test chart")
            .chart_type(ChartType::Library)
            .add_keyword("civitforge")
            .add_keyword("platform")
            .home("https://example.com")
            .add_source("https://github.com/example/myapp")
            .add_maintainer(HelmMaintainer {
                name: "Dev".to_string(),
                email: "dev@example.com".to_string(),
                url: None,
            })
            .build();

        assert_eq!(chart.version, "1.2.3");
        assert_eq!(chart.app_version, "1.2.3");
        assert_eq!(chart.description, "A test chart");
        assert_eq!(chart.type_, ChartType::Library);
        assert_eq!(chart.keywords, vec!["civitforge", "platform"]);
        assert_eq!(chart.home, "https://example.com");
        assert_eq!(chart.maintainers.len(), 1);
        assert_eq!(chart.sources.len(), 1);
    }

    #[test]
    fn test_helm_chart_builder_with_values() {
        let values = HelmValues {
            replicas: 5,
            image: "myapp:2.0".to_string(),
            image_pull_policy: "Always".to_string(),
            ..Default::default()
        };
        let chart = HelmChartBuilder::new("myapp").with_values(values).build();
        assert_eq!(chart.values.replicas, 5);
        assert_eq!(chart.values.image, "myapp:2.0");
        assert_eq!(chart.values.image_pull_policy, "Always");
    }

    #[test]
    fn test_helm_chart_builder_add_template() {
        let tmpl = HelmTemplate {
            name: "deployment".to_string(),
            kind: TemplateKind::Deployment,
            content: "apiVersion: apps/v1".to_string(),
        };
        let chart = HelmChartBuilder::new("myapp")
            .add_template(tmpl.clone())
            .build();
        assert_eq!(chart.templates.len(), 1);
        assert_eq!(chart.templates[0].name, "deployment");
        assert_eq!(chart.templates[0].kind, TemplateKind::Deployment);
    }

    #[test]
    fn test_helm_chart_builder_add_dependency() {
        let dep = HelmDependency {
            name: "postgres".to_string(),
            version: "15.0.0".to_string(),
            repository: "https://charts.example.com".to_string(),
            alias: Some("db".to_string()),
        };
        let chart = HelmChartBuilder::new("myapp").add_dependency(dep).build();
        assert_eq!(chart.dependencies.len(), 1);
        assert_eq!(chart.dependencies[0].name, "postgres");
        assert_eq!(chart.dependencies[0].alias.as_deref(), Some("db"));
    }

    #[test]
    fn test_helm_values_default() {
        let vals = HelmValues::default();
        assert_eq!(vals.replicas, 3);
        assert_eq!(vals.image, "civitforge/app:latest");
        assert_eq!(vals.image_pull_policy, "IfNotPresent");
        assert_eq!(vals.service_type, "ClusterIP");
        assert!(!vals.ingress_enabled);
        assert!(!vals.persistence.enabled);
        assert!(vals.node_selector.is_none());
    }

    #[test]
    fn test_container_resources_default() {
        let res = ContainerResources::default();
        assert_eq!(res.requests.cpu, "100m");
        assert_eq!(res.requests.memory, "128Mi");
        assert_eq!(res.limits.cpu, "500m");
        assert_eq!(res.limits.memory, "512Mi");
    }

    #[test]
    fn test_chart_type_display() {
        assert_eq!(ChartType::Application.to_string(), "application");
        assert_eq!(ChartType::Library.to_string(), "library");
    }

    #[test]
    fn test_template_kind_display() {
        assert_eq!(TemplateKind::Deployment.to_string(), "Deployment");
        assert_eq!(TemplateKind::Service.to_string(), "Service");
        assert_eq!(TemplateKind::Ingress.to_string(), "Ingress");
        assert_eq!(TemplateKind::ConfigMap.to_string(), "ConfigMap");
        assert_eq!(TemplateKind::Secret.to_string(), "Secret");
        assert_eq!(TemplateKind::NetworkPolicy.to_string(), "NetworkPolicy");
        assert_eq!(TemplateKind::HPA.to_string(), "HorizontalPodAutoscaler");
    }

    #[test]
    fn test_helm_upgrade_strategy_default() {
        let strat = HelmUpgradeStrategy::default();
        assert_eq!(strat.max_unavailable, "25%");
        assert_eq!(strat.max_surge, "25%");
        assert_eq!(strat.rollback_history, 10);
        assert!(strat.rollback_on_failure);
    }

    #[test]
    fn test_helm_hpa_default() {
        let hpa = HelmHPA::default();
        assert_eq!(hpa.min_replicas, 3);
        assert_eq!(hpa.max_replicas, 10);
        assert_eq!(hpa.target_cpu_percent, Some(70));
        assert_eq!(hpa.target_memory_percent, Some(80));
    }

    #[test]
    fn test_helm_production_values_default() {
        let prod = HelmProductionValues::default();
        assert_eq!(prod.replicas, 5);
        assert_eq!(prod.image_pull_policy, "Always");
        assert_eq!(prod.service_type, "LoadBalancer");
        assert!(prod.ingress_enabled);
        assert!(prod.persistence.enabled);
        assert_eq!(prod.hpa.min_replicas, 5);
        assert_eq!(prod.hpa.max_replicas, 50);
        assert_eq!(prod.upgrade_strategy.max_unavailable, "10%");
    }

    #[test]
    fn test_persistence_config_default() {
        let p = PersistenceConfig::default();
        assert!(!p.enabled);
        assert_eq!(p.size, "1Gi");
        assert!(p.storage_class.is_none());
        assert_eq!(p.access_mode, "ReadWriteOnce");
    }

    #[test]
    fn test_helm_network_policy() {
        let np = HelmNetworkPolicy {
            pod_selector: HashMap::from([("app".to_string(), "api".to_string())]),
            ingress_rules: vec![],
            egress_rules: vec![],
            policy_types: vec![NetworkPolicyType::Ingress],
        };
        assert_eq!(np.pod_selector.get("app").unwrap(), "api");
        assert_eq!(np.policy_types.len(), 1);
    }

    #[test]
    fn test_network_policy_type_display() {
        assert_eq!(NetworkPolicyType::Ingress.to_string(), "Ingress");
        assert_eq!(NetworkPolicyType::Egress.to_string(), "Egress");
    }

    #[test]
    fn test_toleration_operator_display() {
        assert_eq!(HelmValuesTolerationOperator::Exists.to_string(), "Exists");
        assert_eq!(HelmValuesTolerationOperator::Equal.to_string(), "Equal");
    }

    #[test]
    fn test_toleration_effect_display() {
        assert_eq!(
            HelmValuesTolerationEffect::NoSchedule.to_string(),
            "NoSchedule"
        );
        assert_eq!(
            HelmValuesTolerationEffect::PreferNoSchedule.to_string(),
            "PreferNoSchedule"
        );
        assert_eq!(
            HelmValuesTolerationEffect::NoExecute.to_string(),
            "NoExecute"
        );
    }

    #[test]
    fn test_chart_renderer_basic() {
        let chart = HelmChartBuilder::new("testapp")
            .version("0.1.0")
            .description("Test")
            .build();
        let rendered = HelmChartRenderer::render_chart(&chart);
        assert!(rendered.contains("name: testapp"));
        assert!(rendered.contains("version: 0.1.0"));
        assert!(rendered.contains("description: Test"));
        assert!(rendered.contains("---"));
        assert!(rendered.contains("replicas:"));
    }

    #[test]
    fn test_chart_renderer_with_sources_and_maintainers() {
        let chart = HelmChartBuilder::new("testapp")
            .add_source("https://github.com/example/app")
            .add_maintainer(HelmMaintainer {
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                url: Some("https://alice.dev".to_string()),
            })
            .build();
        let rendered = HelmChartRenderer::render_chart(&chart);
        assert!(rendered.contains("sources:"));
        assert!(rendered.contains("https://github.com/example/app"));
        assert!(rendered.contains("maintainers:"));
        assert!(rendered.contains("name: Alice"));
        assert!(rendered.contains("email: alice@example.com"));
        assert!(rendered.contains("url: https://alice.dev"));
    }

    #[test]
    fn test_chart_renderer_with_dependencies() {
        let chart = HelmChartBuilder::new("testapp")
            .add_dependency(HelmDependency {
                name: "redis".to_string(),
                version: "18.0.0".to_string(),
                repository: "https://charts.bitnami.com".to_string(),
                alias: Some("cache".to_string()),
            })
            .build();
        let rendered = HelmChartRenderer::render_chart(&chart);
        assert!(rendered.contains("dependencies:"));
        assert!(rendered.contains("name: redis"));
        assert!(rendered.contains("alias: cache"));
    }

    #[test]
    fn test_chart_renderer_values_section() {
        let chart = HelmChartBuilder::new("testapp")
            .with_values(HelmValues {
                replicas: 7,
                ingress_enabled: true,
                ..Default::default()
            })
            .build();
        let rendered = HelmChartRenderer::render_chart(&chart);
        assert!(rendered.contains("resources:"));
        assert!(rendered.contains("cpu: 100m"));
        assert!(rendered.contains("memory: 128Mi"));
        assert!(rendered.contains("ingress:"));
        assert!(rendered.contains("persistence:"));
    }
}
