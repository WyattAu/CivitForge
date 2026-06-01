#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum IsolationPolicy {
    None,
    Basic,
    Strict,
    Airgapped,
}

impl Default for IsolationPolicy {
    fn default() -> Self {
        IsolationPolicy::Basic
    }
}

impl std::fmt::Display for IsolationPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IsolationPolicy::None => write!(f, "none"),
            IsolationPolicy::Basic => write!(f, "basic"),
            IsolationPolicy::Strict => write!(f, "strict"),
            IsolationPolicy::Airgapped => write!(f, "airgapped"),
        }
    }
}

impl std::str::FromStr for IsolationPolicy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(IsolationPolicy::None),
            "basic" => Ok(IsolationPolicy::Basic),
            "strict" => Ok(IsolationPolicy::Strict),
            "airgapped" => Ok(IsolationPolicy::Airgapped),
            _ => Err(format!("unknown isolation policy: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationConfig {
    pub policy: IsolationPolicy,
    pub user_namespace: bool,
    pub pid_namespace: bool,
    pub network_namespace: bool,
    pub ipc_namespace: bool,
    pub read_only_root_fs: bool,
    pub seccomp_profile: Option<String>,
    pub capabilities_drop: Vec<String>,
    pub capabilities_add: Vec<String>,
    pub masked_paths: Vec<String>,
    pub readonly_paths: Vec<String>,
}

impl Default for IsolationConfig {
    fn default() -> Self {
        Self {
            policy: IsolationPolicy::default(),
            user_namespace: true,
            pid_namespace: false,
            network_namespace: true,
            ipc_namespace: false,
            read_only_root_fs: true,
            seccomp_profile: Some("runtime/default".into()),
            capabilities_drop: vec!["ALL".into()],
            capabilities_add: vec![],
            masked_paths: vec![
                "/proc/kcore".into(),
                "/proc/latency_stats".into(),
                "/proc/timer_list".into(),
                "/proc/timer_stats".into(),
                "/proc/sched_debug".into(),
                "/sys/firmware".into(),
            ],
            readonly_paths: vec![
                "/proc/asound".into(),
                "/proc/bus".into(),
                "/proc/fs".into(),
                "/proc/irq".into(),
                "/proc/sys".into(),
                "/sys/fs/cgroup".into(),
            ],
        }
    }
}

impl IsolationConfig {
    pub fn with_policy(policy: IsolationPolicy) -> Self {
        let mut config = Self::default();
        config.apply_policy(policy);
        config
    }

    pub fn apply_policy(&mut self, policy: IsolationPolicy) {
        self.policy = policy;
        match policy {
            IsolationPolicy::None => {
                self.user_namespace = false;
                self.pid_namespace = false;
                self.network_namespace = false;
                self.ipc_namespace = false;
                self.read_only_root_fs = false;
                self.seccomp_profile = None;
                self.capabilities_drop = vec![];
                self.capabilities_add = vec![];
                self.masked_paths = vec![];
                self.readonly_paths = vec![];
            }
            IsolationPolicy::Basic => {
                self.user_namespace = true;
                self.pid_namespace = false;
                self.network_namespace = true;
                self.ipc_namespace = false;
                self.read_only_root_fs = true;
                self.seccomp_profile = Some("runtime/default".into());
                self.capabilities_drop = vec!["ALL".into()];
                self.capabilities_add = vec![];
                self.masked_paths = Self::default().masked_paths;
                self.readonly_paths = Self::default().readonly_paths;
            }
            IsolationPolicy::Strict => {
                self.user_namespace = true;
                self.pid_namespace = true;
                self.network_namespace = true;
                self.ipc_namespace = true;
                self.read_only_root_fs = true;
                self.seccomp_profile = Some("runtime/default".into());
                self.capabilities_drop = vec!["ALL".into()];
                self.capabilities_add = vec![];
                self.masked_paths = vec![
                    "/proc/kcore".into(),
                    "/proc/latency_stats".into(),
                    "/proc/timer_list".into(),
                    "/proc/timer_stats".into(),
                    "/proc/sched_debug".into(),
                    "/sys/firmware".into(),
                    "/sys/devices/virtual/powercap".into(),
                ];
                self.readonly_paths = vec![
                    "/proc/asound".into(),
                    "/proc/bus".into(),
                    "/proc/fs".into(),
                    "/proc/irq".into(),
                    "/proc/sys".into(),
                    "/sys/fs/cgroup".into(),
                    "/sys/fs/selinux".into(),
                ];
            }
            IsolationPolicy::Airgapped => {
                self.user_namespace = true;
                self.pid_namespace = true;
                self.network_namespace = true;
                self.ipc_namespace = true;
                self.read_only_root_fs = true;
                self.seccomp_profile = Some("runtime/default".into());
                self.capabilities_drop = vec!["ALL".into()];
                self.capabilities_add = vec![];
                self.masked_paths = vec![
                    "/proc/kcore".into(),
                    "/proc/latency_stats".into(),
                    "/proc/timer_list".into(),
                    "/proc/timer_stats".into(),
                    "/proc/sched_debug".into(),
                    "/sys/firmware".into(),
                    "/sys/devices/virtual/powercap".into(),
                    "/run/secrets".into(),
                    "/var/run/secrets".into(),
                ];
                self.readonly_paths = vec![
                    "/proc/asound".into(),
                    "/proc/bus".into(),
                    "/proc/fs".into(),
                    "/proc/irq".into(),
                    "/proc/sys".into(),
                    "/sys/fs/cgroup".into(),
                    "/sys/fs/selinux".into(),
                    "/etc/resolv.conf".into(),
                    "/etc/hosts".into(),
                ];
            }
        }
    }

    pub fn capabilities_effective(&self) -> Vec<String> {
        let mut caps: Vec<String> = self.capabilities_add.clone();
        caps.retain(|c| !self.capabilities_drop.contains(c));
        caps
    }

    pub fn is_network_isolated(&self) -> bool {
        self.network_namespace
    }

    pub fn is_pid_isolated(&self) -> bool {
        self.pid_namespace
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "tcp"),
            Protocol::Udp => write!(f, "udp"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortRange {
    Single(u16),
    StartEnd { start: u16, end: u16 },
}

impl PortRange {
    pub fn single(port: u16) -> Self {
        PortRange::Single(port)
    }

    pub fn range(start: u16, end: u16) -> Self {
        PortRange::StartEnd { start, end }
    }

    pub fn contains(&self, port: u16) -> bool {
        match self {
            PortRange::Single(p) => *p == port,
            PortRange::StartEnd { start, end } => port >= *start && port <= *end,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleAction {
    Allow,
    Deny,
}

impl std::fmt::Display for RuleAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleAction::Allow => write!(f, "allow"),
            RuleAction::Deny => write!(f, "deny"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRule {
    pub protocol: Protocol,
    pub port_range: PortRange,
    pub cidr: Option<String>,
    pub action: RuleAction,
    pub description: String,
}

impl NetworkRule {
    pub fn allow_tcp(port: u16, description: impl Into<String>) -> Self {
        Self {
            protocol: Protocol::Tcp,
            port_range: PortRange::Single(port),
            cidr: None,
            action: RuleAction::Allow,
            description: description.into(),
        }
    }

    pub fn deny_tcp(port: u16, description: impl Into<String>) -> Self {
        Self {
            protocol: Protocol::Tcp,
            port_range: PortRange::Single(port),
            cidr: None,
            action: RuleAction::Deny,
            description: description.into(),
        }
    }

    pub fn allow_tcp_cidr(port: u16, cidr: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            protocol: Protocol::Tcp,
            port_range: PortRange::Single(port),
            cidr: Some(cidr.into()),
            action: RuleAction::Allow,
            description: description.into(),
        }
    }

    pub fn matches_port(&self, port: u16) -> bool {
        self.port_range.contains(port)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    pub ingress_rules: Vec<NetworkRule>,
    pub egress_rules: Vec<NetworkRule>,
    pub default_deny_ingress: bool,
    pub default_deny_egress: bool,
}

impl Default for NetworkPolicy {
    fn default() -> Self {
        Self {
            ingress_rules: vec![],
            egress_rules: vec![],
            default_deny_ingress: true,
            default_deny_egress: true,
        }
    }
}

impl NetworkPolicy {
    pub fn allow_all() -> Self {
        Self {
            ingress_rules: vec![],
            egress_rules: vec![],
            default_deny_ingress: false,
            default_deny_egress: false,
        }
    }

    pub fn deny_all() -> Self {
        Self {
            ingress_rules: vec![],
            egress_rules: vec![],
            default_deny_ingress: true,
            default_deny_egress: true,
        }
    }

    pub fn is_egress_blocked(&self, port: u16) -> bool {
        if !self.default_deny_egress {
            return false;
        }
        !self.egress_rules.iter().any(|r| r.matches_port(port) && r.action == RuleAction::Allow)
    }

    pub fn is_ingress_blocked(&self, port: u16) -> bool {
        if !self.default_deny_ingress {
            return false;
        }
        !self.ingress_rules.iter().any(|r| r.matches_port(port) && r.action == RuleAction::Allow)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxSpec {
    pub id: String,
    pub image: String,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationReport {
    pub namespace_configured: bool,
    pub network_policies_applied: bool,
    pub capabilities_set: bool,
    pub violations: Vec<String>,
}

impl IsolationReport {
    pub fn is_compliant(&self) -> bool {
        self.violations.is_empty()
    }
}

impl Default for IsolationReport {
    fn default() -> Self {
        Self {
            namespace_configured: false,
            network_policies_applied: false,
            capabilities_set: false,
            violations: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct IsolationEnforcer;

impl IsolationEnforcer {
    pub fn new() -> Self {
        Self
    }

    pub fn enforce(&self, config: &IsolationConfig, spec: &SandboxSpec) -> IsolationReport {
        let mut report = IsolationReport::default();
        let mut violations = Vec::new();

        match config.policy {
            IsolationPolicy::None => {
                violations.push("no isolation policy applied".into());
            }
            IsolationPolicy::Basic => {
                report.namespace_configured = config.user_namespace;
                if config.user_namespace {
                    report.network_policies_applied = config.network_namespace;
                }
                report.capabilities_set = !config.capabilities_drop.is_empty();
            }
            IsolationPolicy::Strict => {
                report.namespace_configured =
                    config.user_namespace && config.pid_namespace && config.ipc_namespace;
                report.network_policies_applied = config.network_namespace;
                report.capabilities_set = config.capabilities_drop.contains(&"ALL".into());

                if !config.read_only_root_fs {
                    violations.push("read-only root filesystem not enabled".into());
                }
                if config.seccomp_profile.is_none() {
                    violations.push("no seccomp profile configured".into());
                }
                if !config.user_namespace {
                    violations.push("user namespace not enabled".into());
                }
                if !config.pid_namespace {
                    violations.push("pid namespace not enabled".into());
                }
            }
            IsolationPolicy::Airgapped => {
                report.namespace_configured = config.user_namespace
                    && config.pid_namespace
                    && config.network_namespace
                    && config.ipc_namespace;
                report.network_policies_applied = config.network_namespace;
                report.capabilities_set = config.capabilities_drop.contains(&"ALL".into());

                if !config.read_only_root_fs {
                    violations.push("read-only root filesystem not enabled".into());
                }
                if config.seccomp_profile.is_none() {
                    violations.push("no seccomp profile configured".into());
                }
                if !config.network_namespace {
                    violations.push("network namespace not enabled".into());
                }
                if !config.user_namespace {
                    violations.push("user namespace not enabled".into());
                }
                if !config.pid_namespace {
                    violations.push("pid namespace not enabled".into());
                }
                if !config.ipc_namespace {
                    violations.push("ipc namespace not enabled".into());
                }

                if let Some(image) = spec.image.strip_prefix("localhost/") {
                    violations.push(format!("local image '{image}' may bypass airgap"));
                }
            }
        }

        if config.capabilities_add.contains(&"SYS_ADMIN".into())
            || config.capabilities_add.contains(&"NET_ADMIN".into())
            || config.capabilities_add.contains(&"SYS_PTRACE".into())
        {
            violations.push("dangerous capability added".into());
        }

        if !config.masked_paths.contains(&"/proc/kcore".into())
            && config.policy != IsolationPolicy::None
        {
            violations.push("sensitive path /proc/kcore not masked".into());
        }

        report.violations = violations;
        report
    }
}

impl Default for IsolationEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_sandbox_spec() -> SandboxSpec {
        SandboxSpec {
            id: "test-sandbox".into(),
            image: "alpine:latest".into(),
            labels: HashMap::new(),
        }
    }

    #[test]
    fn test_isolation_policy_default() {
        assert_eq!(IsolationPolicy::default(), IsolationPolicy::Basic);
    }

    #[test]
    fn test_isolation_policy_display() {
        assert_eq!(IsolationPolicy::None.to_string(), "none");
        assert_eq!(IsolationPolicy::Strict.to_string(), "strict");
        assert_eq!(IsolationPolicy::Airgapped.to_string(), "airgapped");
    }

    #[test]
    fn test_isolation_policy_from_str() {
        assert_eq!("basic".parse::<IsolationPolicy>().unwrap(), IsolationPolicy::Basic);
        assert_eq!("strict".parse::<IsolationPolicy>().unwrap(), IsolationPolicy::Strict);
        assert_eq!("airgapped".parse::<IsolationPolicy>().unwrap(), IsolationPolicy::Airgapped);
        assert_eq!("none".parse::<IsolationPolicy>().unwrap(), IsolationPolicy::None);
        assert!("unknown".parse::<IsolationPolicy>().is_err());
    }

    #[test]
    fn test_isolation_policy_serialization() {
        let policies = vec![
            IsolationPolicy::None,
            IsolationPolicy::Basic,
            IsolationPolicy::Strict,
            IsolationPolicy::Airgapped,
        ];
        for p in policies {
            let json = serde_json::to_string(&p).unwrap();
            let de: IsolationPolicy = serde_json::from_str(&json).unwrap();
            assert_eq!(p, de);
        }
    }

    #[test]
    fn test_isolation_config_default() {
        let config = IsolationConfig::default();
        assert_eq!(config.policy, IsolationPolicy::Basic);
        assert!(config.user_namespace);
        assert!(config.read_only_root_fs);
        assert!(config.capabilities_drop.contains(&"ALL".into()));
        assert!(config.masked_paths.contains(&"/proc/kcore".into()));
    }

    #[test]
    fn test_isolation_config_with_policy_none() {
        let config = IsolationConfig::with_policy(IsolationPolicy::None);
        assert!(!config.user_namespace);
        assert!(!config.read_only_root_fs);
        assert!(config.seccomp_profile.is_none());
        assert!(config.masked_paths.is_empty());
    }

    #[test]
    fn test_isolation_config_with_policy_strict() {
        let config = IsolationConfig::with_policy(IsolationPolicy::Strict);
        assert!(config.user_namespace);
        assert!(config.pid_namespace);
        assert!(config.network_namespace);
        assert!(config.ipc_namespace);
        assert!(config.read_only_root_fs);
    }

    #[test]
    fn test_isolation_config_with_policy_airgapped() {
        let config = IsolationConfig::with_policy(IsolationPolicy::Airgapped);
        assert!(config.user_namespace);
        assert!(config.pid_namespace);
        assert!(config.network_namespace);
        assert!(config.ipc_namespace);
        assert!(config.masked_paths.contains(&"/run/secrets".into()));
    }

    #[test]
    fn test_isolation_config_apply_policy() {
        let mut config = IsolationConfig::with_policy(IsolationPolicy::None);
        config.apply_policy(IsolationPolicy::Strict);
        assert!(config.pid_namespace);
        assert!(config.ipc_namespace);
    }

    #[test]
    fn test_capabilities_effective() {
        let mut config = IsolationConfig::with_policy(IsolationPolicy::Basic);
        config.capabilities_add = vec!["NET_BIND_SERVICE".into(), "CHOWN".into()];
        let effective = config.capabilities_effective();
        assert_eq!(effective, vec!["NET_BIND_SERVICE", "CHOWN"]);
    }

    #[test]
    fn test_capabilities_effective_dropped() {
        let mut config = IsolationConfig::with_policy(IsolationPolicy::Basic);
        config.capabilities_drop = vec!["ALL".into(), "SYS_ADMIN".into()];
        config.capabilities_add = vec!["SYS_ADMIN".into()];
        let effective = config.capabilities_effective();
        assert!(effective.is_empty());
    }

    #[test]
    fn test_is_network_isolated() {
        let config = IsolationConfig::with_policy(IsolationPolicy::Basic);
        assert!(config.is_network_isolated());

        let config = IsolationConfig::with_policy(IsolationPolicy::None);
        assert!(!config.is_network_isolated());
    }

    #[test]
    fn test_port_range_single() {
        let range = PortRange::single(80);
        assert!(range.contains(80));
        assert!(!range.contains(81));
    }

    #[test]
    fn test_port_range_start_end() {
        let range = PortRange::range(8000, 9000);
        assert!(range.contains(8000));
        assert!(range.contains(8500));
        assert!(range.contains(9000));
        assert!(!range.contains(7999));
        assert!(!range.contains(9001));
    }

    #[test]
    fn test_network_rule_helpers() {
        let rule = NetworkRule::allow_tcp(80, "http");
        assert_eq!(rule.protocol, Protocol::Tcp);
        assert_eq!(rule.action, RuleAction::Allow);
        assert!(rule.matches_port(80));
        assert!(!rule.matches_port(443));

        let deny = NetworkRule::deny_tcp(22, "ssh");
        assert_eq!(deny.action, RuleAction::Deny);
    }

    #[test]
    fn test_network_rule_with_cidr() {
        let rule = NetworkRule::allow_tcp_cidr(443, "10.0.0.0/8", "internal");
        assert_eq!(rule.cidr, Some("10.0.0.0/8".into()));
        assert!(rule.matches_port(443));
    }

    #[test]
    fn test_network_policy_default() {
        let policy = NetworkPolicy::default();
        assert!(policy.default_deny_ingress);
        assert!(policy.default_deny_egress);
    }

    #[test]
    fn test_network_policy_allow_all() {
        let policy = NetworkPolicy::allow_all();
        assert!(!policy.default_deny_ingress);
        assert!(!policy.default_deny_egress);
        assert!(!policy.is_egress_blocked(443));
    }

    #[test]
    fn test_network_policy_deny_all() {
        let policy = NetworkPolicy::deny_all();
        assert!(policy.is_egress_blocked(80));
        assert!(policy.is_ingress_blocked(443));
    }

    #[test]
    fn test_network_policy_egress_rule() {
        let policy = NetworkPolicy {
            egress_rules: vec![NetworkRule::allow_tcp(443, "https")],
            ..Default::default()
        };
        assert!(!policy.is_egress_blocked(443));
        assert!(policy.is_egress_blocked(80));
    }

    #[test]
    fn test_enforcer_basic() {
        let enforcer = IsolationEnforcer::new();
        let config = IsolationConfig::with_policy(IsolationPolicy::Basic);
        let spec = test_sandbox_spec();
        let report = enforcer.enforce(&config, &spec);
        assert!(report.is_compliant());
        assert!(report.namespace_configured);
    }

    #[test]
    fn test_enforcer_none_policy_violations() {
        let enforcer = IsolationEnforcer::new();
        let config = IsolationConfig::with_policy(IsolationPolicy::None);
        let spec = test_sandbox_spec();
        let report = enforcer.enforce(&config, &spec);
        assert!(!report.is_compliant());
        assert!(!report.violations.is_empty());
    }

    #[test]
    fn test_enforcer_strict_compliant() {
        let enforcer = IsolationEnforcer::new();
        let config = IsolationConfig::with_policy(IsolationPolicy::Strict);
        let spec = test_sandbox_spec();
        let report = enforcer.enforce(&config, &spec);
        assert!(report.is_compliant());
        assert!(report.namespace_configured);
        assert!(report.network_policies_applied);
        assert!(report.capabilities_set);
    }

    #[test]
    fn test_enforcer_airgapped_compliant() {
        let enforcer = IsolationEnforcer::new();
        let config = IsolationConfig::with_policy(IsolationPolicy::Airgapped);
        let spec = test_sandbox_spec();
        let report = enforcer.enforce(&config, &spec);
        assert!(report.is_compliant());
    }

    #[test]
    fn test_enforcer_airgapped_local_image_violation() {
        let enforcer = IsolationEnforcer::new();
        let config = IsolationConfig::with_policy(IsolationPolicy::Airgapped);
        let spec = SandboxSpec {
            id: "local-sandbox".into(),
            image: "localhost/my-image:latest".into(),
            labels: HashMap::new(),
        };
        let report = enforcer.enforce(&config, &spec);
        assert!(!report.is_compliant());
        assert!(report.violations.iter().any(|v| v.contains("airgap")));
    }

    #[test]
    fn test_enforcer_dangerous_capability_violation() {
        let enforcer = IsolationEnforcer::new();
        let mut config = IsolationConfig::with_policy(IsolationPolicy::Basic);
        config.capabilities_add = vec!["SYS_ADMIN".into()];
        let spec = test_sandbox_spec();
        let report = enforcer.enforce(&config, &spec);
        assert!(!report.is_compliant());
        assert!(report.violations.iter().any(|v| v.contains("dangerous")));
    }

    #[test]
    fn test_enforcer_missing_seccomp_violation() {
        let enforcer = IsolationEnforcer::new();
        let mut config = IsolationConfig::with_policy(IsolationPolicy::Strict);
        config.seccomp_profile = None;
        let spec = test_sandbox_spec();
        let report = enforcer.enforce(&config, &spec);
        assert!(!report.is_compliant());
        assert!(report.violations.iter().any(|v| v.contains("seccomp")));
    }

    #[test]
    fn test_isolation_config_serialization() {
        let config = IsolationConfig::with_policy(IsolationPolicy::Strict);
        let json = serde_json::to_string(&config).unwrap();
        let de: IsolationConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(de.policy, IsolationPolicy::Strict);
        assert_eq!(de.pid_namespace, config.pid_namespace);
    }

    #[test]
    fn test_isolation_report_default() {
        let report = IsolationReport::default();
        assert!(!report.namespace_configured);
        assert!(report.is_compliant());
    }
}
