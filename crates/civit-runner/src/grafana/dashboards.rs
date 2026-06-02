#![forbid(unsafe_code)]

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Dashboard {
    pub uid: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub panels: Vec<Panel>,
    pub templating: Vec<DashboardVariable>,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Panel {
    pub id: u32,
    pub title: String,
    pub type_: PanelType,
    pub grid_pos: GridPos,
    pub targets: Vec<PanelTarget>,
    pub thresholds: Vec<Threshold>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PanelType {
    Graph,
    Table,
    Stat,
    Logs,
}

impl std::fmt::Display for PanelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PanelType::Graph => write!(f, "graph"),
            PanelType::Table => write!(f, "table"),
            PanelType::Stat => write!(f, "stat"),
            PanelType::Logs => write!(f, "logs"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GridPos {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PanelTarget {
    pub prometheus_expr: String,
    pub legend_format: String,
    pub ref_id: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Threshold {
    pub value: f64,
    pub color: String,
    pub severity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DashboardVariable {
    pub name: String,
    pub label: String,
    pub query: String,
    pub current: Option<String>,
    pub include_all: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertRule {
    pub name: String,
    pub condition: AlertCondition,
    pub for_duration: std::time::Duration,
    pub severity: AlertSeverity,
    pub annotations: HashMap<String, String>,
    pub notification_channels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertCondition {
    pub query: String,
    pub operator: AlertConditionOperator,
    pub threshold: u64,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertConditionOperator {
    Gt,
    Lt,
    Eq,
    Neq,
}

impl std::fmt::Display for AlertConditionOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertConditionOperator::Gt => write!(f, ">"),
            AlertConditionOperator::Lt => write!(f, "<"),
            AlertConditionOperator::Eq => write!(f, "=="),
            AlertConditionOperator::Neq => write!(f, "!="),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertSeverity {
    Warning,
    Critical,
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Warning => write!(f, "warning"),
            AlertSeverity::Critical => write!(f, "critical"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardTemplate {
    pub name: String,
    pub description: String,
    pub panels: Vec<Panel>,
    pub variables: Vec<DashboardVariable>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRuleTemplate {
    pub name: String,
    pub expr: String,
    pub for_duration: std::time::Duration,
    pub severity: AlertSeverity,
    pub message: String,
}

pub struct PredefinedDashboards;

impl PredefinedDashboards {
    pub fn api_latency() -> Dashboard {
        Dashboard {
            uid: "civitforge-api-latency".to_string(),
            title: "API Latency".to_string(),
            description: "API request latency metrics".to_string(),
            tags: vec!["civitforge".to_string(), "api".to_string(), "latency".to_string()],
            panels: vec![
                Panel {
                    id: 1,
                    title: "Request Latency (p99)".to_string(),
                    type_: PanelType::Graph,
                    grid_pos: GridPos { x: 0, y: 0, w: 12, h: 8 },
                    targets: vec![PanelTarget {
                        prometheus_expr: "histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket[5m])) by (le, method, path))".to_string(),
                        legend_format: "{{method}} {{path}}".to_string(),
                        ref_id: "A".to_string(),
                    }],
                    thresholds: vec![
                        Threshold { value: 0.1, color: "green".to_string(), severity: Some("ok".to_string()) },
                        Threshold { value: 0.5, color: "yellow".to_string(), severity: Some("warning".to_string()) },
                        Threshold { value: 1.0, color: "red".to_string(), severity: Some("critical".to_string()) },
                    ],
                },
                Panel {
                    id: 2,
                    title: "Request Rate".to_string(),
                    type_: PanelType::Stat,
                    grid_pos: GridPos { x: 12, y: 0, w: 12, h: 8 },
                    targets: vec![PanelTarget {
                        prometheus_expr: "sum(rate(http_requests_total[5m]))".to_string(),
                        legend_format: "".to_string(),
                        ref_id: "B".to_string(),
                    }],
                    thresholds: vec![],
                },
            ],
            templating: vec![DashboardVariable {
                name: "namespace".to_string(),
                label: "Namespace".to_string(),
                query: "label_values(http_requests_total, namespace)".to_string(),
                current: Some("civitforge".to_string()),
                include_all: false,
            }],
            version: 1,
        }
    }

    pub fn git_operations() -> Dashboard {
        Dashboard {
            uid: "civitforge-git-ops".to_string(),
            title: "Git Operations".to_string(),
            description: "Git clone, push, and fetch metrics".to_string(),
            tags: vec!["civitforge".to_string(), "git".to_string()],
            panels: vec![
                Panel {
                    id: 1,
                    title: "Clone Duration".to_string(),
                    type_: PanelType::Graph,
                    grid_pos: GridPos { x: 0, y: 0, w: 24, h: 8 },
                    targets: vec![PanelTarget {
                        prometheus_expr: "histogram_quantile(0.99, sum(rate(git_clone_duration_seconds_bucket[5m])) by (le))".to_string(),
                        legend_format: "p99".to_string(),
                        ref_id: "A".to_string(),
                    }],
                    thresholds: vec![
                        Threshold { value: 5.0, color: "green".to_string(), severity: None },
                        Threshold { value: 30.0, color: "red".to_string(), severity: None },
                    ],
                },
            ],
            templating: vec![],
            version: 1,
        }
    }

    pub fn ci_pipeline() -> Dashboard {
        Dashboard {
            uid: "civitforge-ci-pipeline".to_string(),
            title: "CI Pipeline".to_string(),
            description: "Pipeline run duration and success metrics".to_string(),
            tags: vec!["civitforge".to_string(), "ci".to_string()],
            panels: vec![
                Panel {
                    id: 1,
                    title: "Pipeline Duration".to_string(),
                    type_: PanelType::Graph,
                    grid_pos: GridPos { x: 0, y: 0, w: 16, h: 8 },
                    targets: vec![PanelTarget {
                        prometheus_expr: "histogram_quantile(0.99, sum(rate(pipeline_run_duration_seconds_bucket[5m])) by (le))".to_string(),
                        legend_format: "p99".to_string(),
                        ref_id: "A".to_string(),
                    }],
                    thresholds: vec![],
                },
                Panel {
                    id: 2,
                    title: "Pipeline Success Rate".to_string(),
                    type_: PanelType::Stat,
                    grid_pos: GridPos { x: 16, y: 0, w: 8, h: 8 },
                    targets: vec![PanelTarget {
                        prometheus_expr: "sum(rate(pipeline_runs_total{status=\"success\"}[1h])) / sum(rate(pipeline_runs_total[1h])) * 100".to_string(),
                        legend_format: "".to_string(),
                        ref_id: "B".to_string(),
                    }],
                    thresholds: vec![
                        Threshold { value: 95.0, color: "green".to_string(), severity: None },
                        Threshold { value: 90.0, color: "yellow".to_string(), severity: None },
                        Threshold { value: 80.0, color: "red".to_string(), severity: None },
                    ],
                },
            ],
            templating: vec![],
            version: 1,
        }
    }

    pub fn ai_inference() -> Dashboard {
        Dashboard {
            uid: "civitforge-ai-inference".to_string(),
            title: "AI Inference".to_string(),
            description: "Model inference latency and throughput".to_string(),
            tags: vec!["civitforge".to_string(), "ai".to_string(), "inference".to_string()],
            panels: vec![
                Panel {
                    id: 1,
                    title: "Inference Latency".to_string(),
                    type_: PanelType::Graph,
                    grid_pos: GridPos { x: 0, y: 0, w: 24, h: 8 },
                    targets: vec![PanelTarget {
                        prometheus_expr: "histogram_quantile(0.99, sum(rate(inference_duration_seconds_bucket[5m])) by (le, model))".to_string(),
                        legend_format: "{{model}}".to_string(),
                        ref_id: "A".to_string(),
                    }],
                    thresholds: vec![],
                },
            ],
            templating: vec![DashboardVariable {
                name: "model".to_string(),
                label: "Model".to_string(),
                query: "label_values(inference_duration_seconds_count, model)".to_string(),
                current: None,
                include_all: true,
            }],
            version: 1,
        }
    }

    pub fn federation_sync() -> Dashboard {
        Dashboard {
            uid: "civitforge-federation-sync".to_string(),
            title: "Federation Sync".to_string(),
            description: "Cross-instance federation synchronization metrics".to_string(),
            tags: vec!["civitforge".to_string(), "federation".to_string()],
            panels: vec![Panel {
                id: 1,
                title: "Sync Lag".to_string(),
                type_: PanelType::Graph,
                grid_pos: GridPos {
                    x: 0,
                    y: 0,
                    w: 24,
                    h: 8,
                },
                targets: vec![PanelTarget {
                    prometheus_expr: "federation_sync_lag_seconds".to_string(),
                    legend_format: "{{instance}}".to_string(),
                    ref_id: "A".to_string(),
                }],
                thresholds: vec![
                    Threshold {
                        value: 10.0,
                        color: "green".to_string(),
                        severity: None,
                    },
                    Threshold {
                        value: 60.0,
                        color: "red".to_string(),
                        severity: None,
                    },
                ],
            }],
            templating: vec![],
            version: 1,
        }
    }
}

pub struct PredefinedAlerts;

impl PredefinedAlerts {
    pub fn high_error_rate() -> AlertRule {
        AlertRule {
            name: "HighErrorRate".to_string(),
            condition: AlertCondition {
                query: "sum(rate(http_requests_total{status=~\"5..\"}[5m])) / sum(rate(http_requests_total[5m]))".to_string(),
                operator: AlertConditionOperator::Gt,
                threshold: 5,
                labels: HashMap::from([("team".to_string(), "platform".to_string())]),
            },
            for_duration: std::time::Duration::from_secs(300),
            severity: AlertSeverity::Critical,
            annotations: HashMap::from([
                ("summary".to_string(), "High error rate detected".to_string()),
                ("description".to_string(), "Error rate exceeds 5% for more than 5 minutes".to_string()),
            ]),
            notification_channels: vec!["slack-platform".to_string(), "pagerduty".to_string()],
        }
    }

    pub fn high_latency() -> AlertRule {
        AlertRule {
            name: "HighLatency".to_string(),
            condition: AlertCondition {
                query: "histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket[5m])) by (le))".to_string(),
                operator: AlertConditionOperator::Gt,
                threshold: 1000,
                labels: HashMap::from([("team".to_string(), "platform".to_string())]),
            },
            for_duration: std::time::Duration::from_secs(300),
            severity: AlertSeverity::Warning,
            annotations: HashMap::from([
                ("summary".to_string(), "High API latency detected".to_string()),
                ("description".to_string(), "p99 latency exceeds 1s for more than 5 minutes".to_string()),
            ]),
            notification_channels: vec!["slack-platform".to_string()],
        }
    }

    pub fn pod_restart() -> AlertRule {
        AlertRule {
            name: "PodRestartLoop".to_string(),
            condition: AlertCondition {
                query: "rate(kube_pod_container_status_restarts_total[15m])".to_string(),
                operator: AlertConditionOperator::Gt,
                threshold: 0,
                labels: HashMap::from([("team".to_string(), "platform".to_string())]),
            },
            for_duration: std::time::Duration::from_secs(900),
            severity: AlertSeverity::Warning,
            annotations: HashMap::from([
                (
                    "summary".to_string(),
                    "Pod restart loop detected".to_string(),
                ),
                (
                    "description".to_string(),
                    "Container has restarted frequently in the last 15 minutes".to_string(),
                ),
            ]),
            notification_channels: vec!["slack-platform".to_string()],
        }
    }

    pub fn disk_usage() -> AlertRule {
        AlertRule {
            name: "HighDiskUsage".to_string(),
            condition: AlertCondition {
                query: "kubelet_volume_stats_used_bytes / kubelet_volume_stats_capacity_bytes"
                    .to_string(),
                operator: AlertConditionOperator::Gt,
                threshold: 85,
                labels: HashMap::from([("team".to_string(), "platform".to_string())]),
            },
            for_duration: std::time::Duration::from_secs(600),
            severity: AlertSeverity::Warning,
            annotations: HashMap::from([
                ("summary".to_string(), "High disk usage".to_string()),
                (
                    "description".to_string(),
                    "Disk usage exceeds 85%".to_string(),
                ),
            ]),
            notification_channels: vec!["slack-platform".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_creation() {
        let dashboard = Dashboard {
            uid: "test".to_string(),
            title: "Test Dashboard".to_string(),
            description: "A test".to_string(),
            tags: vec!["test".to_string()],
            panels: vec![],
            templating: vec![],
            version: 1,
        };
        assert_eq!(dashboard.uid, "test");
        assert_eq!(dashboard.title, "Test Dashboard");
        assert!(dashboard.panels.is_empty());
    }

    #[test]
    fn test_panel_creation() {
        let panel = Panel {
            id: 1,
            title: "CPU Usage".to_string(),
            type_: PanelType::Graph,
            grid_pos: GridPos {
                x: 0,
                y: 0,
                w: 12,
                h: 8,
            },
            targets: vec![PanelTarget {
                prometheus_expr: "cpu_usage".to_string(),
                legend_format: "{{pod}}".to_string(),
                ref_id: "A".to_string(),
            }],
            thresholds: vec![Threshold {
                value: 80.0,
                color: "red".to_string(),
                severity: Some("critical".to_string()),
            }],
        };
        assert_eq!(panel.id, 1);
        assert_eq!(panel.type_, PanelType::Graph);
        assert_eq!(panel.grid_pos.w, 12);
        assert_eq!(panel.targets.len(), 1);
        assert_eq!(panel.thresholds.len(), 1);
    }

    #[test]
    fn test_panel_type_display() {
        assert_eq!(PanelType::Graph.to_string(), "graph");
        assert_eq!(PanelType::Table.to_string(), "table");
        assert_eq!(PanelType::Stat.to_string(), "stat");
        assert_eq!(PanelType::Logs.to_string(), "logs");
    }

    #[test]
    fn test_grid_pos() {
        let pos = GridPos {
            x: 0,
            y: 8,
            w: 24,
            h: 4,
        };
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 8);
        assert_eq!(pos.w, 24);
        assert_eq!(pos.h, 4);
    }

    #[test]
    fn test_alert_rule_creation() {
        let rule = AlertRule {
            name: "TestAlert".to_string(),
            condition: AlertCondition {
                query: "up == 0".to_string(),
                operator: AlertConditionOperator::Eq,
                threshold: 0,
                labels: HashMap::new(),
            },
            for_duration: std::time::Duration::from_secs(60),
            severity: AlertSeverity::Critical,
            annotations: HashMap::new(),
            notification_channels: vec!["slack".to_string()],
        };
        assert_eq!(rule.name, "TestAlert");
        assert_eq!(rule.severity, AlertSeverity::Critical);
        assert_eq!(rule.notification_channels, vec!["slack"]);
    }

    #[test]
    fn test_alert_condition_operator_display() {
        assert_eq!(AlertConditionOperator::Gt.to_string(), ">");
        assert_eq!(AlertConditionOperator::Lt.to_string(), "<");
        assert_eq!(AlertConditionOperator::Eq.to_string(), "==");
        assert_eq!(AlertConditionOperator::Neq.to_string(), "!=");
    }

    #[test]
    fn test_alert_severity_display() {
        assert_eq!(AlertSeverity::Warning.to_string(), "warning");
        assert_eq!(AlertSeverity::Critical.to_string(), "critical");
    }

    #[test]
    fn test_dashboard_template() {
        let tmpl = DashboardTemplate {
            name: "api".to_string(),
            description: "API dashboard template".to_string(),
            panels: vec![],
            variables: vec![DashboardVariable {
                name: "ns".to_string(),
                label: "Namespace".to_string(),
                query: "label_values(up, namespace)".to_string(),
                current: None,
                include_all: true,
            }],
        };
        assert_eq!(tmpl.name, "api");
        assert_eq!(tmpl.variables.len(), 1);
    }

    #[test]
    fn test_alert_rule_template() {
        let tmpl = AlertRuleTemplate {
            name: "HighLatency".to_string(),
            expr: "latency > 1".to_string(),
            for_duration: std::time::Duration::from_secs(300),
            severity: AlertSeverity::Warning,
            message: "Latency is high".to_string(),
        };
        assert_eq!(tmpl.name, "HighLatency");
        assert_eq!(tmpl.severity, AlertSeverity::Warning);
    }

    #[test]
    fn test_predefined_dashboards_api_latency() {
        let dashboard = PredefinedDashboards::api_latency();
        assert_eq!(dashboard.uid, "civitforge-api-latency");
        assert_eq!(dashboard.panels.len(), 2);
        assert_eq!(dashboard.templating.len(), 1);
        assert_eq!(dashboard.tags.len(), 3);
    }

    #[test]
    fn test_predefined_dashboards_git_operations() {
        let dashboard = PredefinedDashboards::git_operations();
        assert_eq!(dashboard.uid, "civitforge-git-ops");
        assert_eq!(dashboard.panels.len(), 1);
    }

    #[test]
    fn test_predefined_dashboards_ci_pipeline() {
        let dashboard = PredefinedDashboards::ci_pipeline();
        assert_eq!(dashboard.uid, "civitforge-ci-pipeline");
        assert_eq!(dashboard.panels.len(), 2);
        let stat_panel = &dashboard.panels[1];
        assert_eq!(stat_panel.type_, PanelType::Stat);
        assert_eq!(stat_panel.thresholds.len(), 3);
    }

    #[test]
    fn test_predefined_dashboards_ai_inference() {
        let dashboard = PredefinedDashboards::ai_inference();
        assert_eq!(dashboard.uid, "civitforge-ai-inference");
        assert!(dashboard.templating[0].include_all);
    }

    #[test]
    fn test_predefined_dashboards_federation_sync() {
        let dashboard = PredefinedDashboards::federation_sync();
        assert_eq!(dashboard.uid, "civitforge-federation-sync");
        assert_eq!(dashboard.panels.len(), 1);
    }

    #[test]
    fn test_predefined_alerts_high_error_rate() {
        let alert = PredefinedAlerts::high_error_rate();
        assert_eq!(alert.name, "HighErrorRate");
        assert_eq!(alert.severity, AlertSeverity::Critical);
        assert_eq!(alert.condition.operator, AlertConditionOperator::Gt);
        assert_eq!(alert.notification_channels.len(), 2);
    }

    #[test]
    fn test_predefined_alerts_high_latency() {
        let alert = PredefinedAlerts::high_latency();
        assert_eq!(alert.name, "HighLatency");
        assert_eq!(alert.severity, AlertSeverity::Warning);
        assert!(alert.condition.threshold > 0);
    }

    #[test]
    fn test_predefined_alerts_pod_restart() {
        let alert = PredefinedAlerts::pod_restart();
        assert_eq!(alert.name, "PodRestartLoop");
        assert_eq!(alert.for_duration, std::time::Duration::from_secs(900));
    }

    #[test]
    fn test_predefined_alerts_disk_usage() {
        let alert = PredefinedAlerts::disk_usage();
        assert_eq!(alert.name, "HighDiskUsage");
        assert_eq!(alert.condition.threshold, 85);
        assert_eq!(alert.for_duration, std::time::Duration::from_secs(600));
    }

    #[test]
    fn test_panel_target() {
        let target = PanelTarget {
            prometheus_expr: "sum(rate(http_requests_total[5m]))".to_string(),
            legend_format: "total".to_string(),
            ref_id: "A".to_string(),
        };
        assert!(target.prometheus_expr.contains("rate"));
        assert_eq!(target.ref_id, "A");
    }
}
