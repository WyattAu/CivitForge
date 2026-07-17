#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaDefinition {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub metric_type: SlaMetricType,
    pub target_value: f64,
    pub window_minutes: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlaMetricType {
    Uptime,
    ResponseTime,
    ErrorRate,
    Throughput,
    LatencyP99,
    LatencyP95,
    Availability,
}

impl SlaMetricType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Uptime => "Uptime",
            Self::ResponseTime => "Response Time",
            Self::ErrorRate => "Error Rate",
            Self::Throughput => "Throughput",
            Self::LatencyP99 => "Latency (p99)",
            Self::LatencyP95 => "Latency (p95)",
            Self::Availability => "Availability",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaMeasurement {
    pub id: Uuid,
    pub sla_id: Uuid,
    pub actual_value: f64,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlaStatus {
    Met,
    Breached,
    AtRisk,
    Unknown,
}

impl SlaStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Met => "Met",
            Self::Breached => "Breached",
            Self::AtRisk => "At Risk",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaCurrentStatus {
    pub sla_id: Uuid,
    pub sla_name: String,
    pub metric_type: SlaMetricType,
    pub target_value: f64,
    pub current_value: f64,
    pub status: SlaStatus,
    pub compliance_percentage: f64,
    pub last_checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaHistoricalEntry {
    pub timestamp: DateTime<Utc>,
    pub actual_value: f64,
    pub target_value: f64,
    pub status: SlaStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaBreach {
    pub id: Uuid,
    pub sla_id: Uuid,
    pub sla_name: String,
    pub metric_type: SlaMetricType,
    pub target_value: f64,
    pub actual_value: f64,
    pub detected_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i64>,
    pub severity: SlaBreachSeverity,
    pub incident_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlaBreachSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl SlaBreachSeverity {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Critical => "Critical",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaReport {
    pub id: Uuid,
    pub period: ReportPeriod,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub sla_results: Vec<SlaResult>,
    pub overall_compliance: f64,
    pub total_breaches: u32,
    pub total_incidents: u32,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportPeriod {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaResult {
    pub sla_id: Uuid,
    pub sla_name: String,
    pub metric_type: SlaMetricType,
    pub target_value: f64,
    pub actual_value: f64,
    pub uptime_percentage: f64,
    pub breach_count: u32,
    pub total_downtime_minutes: i64,
    pub status: SlaStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaDashboardData {
    pub overall_status: SlaStatus,
    pub overall_compliance: f64,
    pub active_sla_count: u32,
    pub breached_sla_count: u32,
    pub at_risk_sla_count: u32,
    pub total_breaches_this_month: u32,
    pub current_incidents: u32,
    pub sla_statuses: Vec<SlaCurrentStatus>,
    pub recent_breaches: Vec<SlaBreach>,
    pub compliance_trend: Vec<ComplianceTrendPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceTrendPoint {
    pub date: DateTime<Utc>,
    pub compliance_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaAlertConfig {
    pub id: Uuid,
    pub sla_id: Uuid,
    pub alert_type: SlaAlertType,
    pub threshold_percentage: f64,
    pub notify_emails: Vec<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlaAlertType {
    Breach,
    AtRisk,
    Recovery,
    Degraded,
}

impl SlaAlertType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Breach => "Breach",
            Self::AtRisk => "At Risk",
            Self::Recovery => "Recovery",
            Self::Degraded => "Degraded",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaIncidentCorrelation {
    pub incident_id: Uuid,
    pub sla_breach_id: Uuid,
    pub sla_name: String,
    pub correlation_type: CorrelationType,
    pub impact_duration_minutes: i64,
    pub affected_endpoints: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrelationType {
    DirectCause,
    Contributing,
    Unrelated,
}

impl Default for SlaDefinition {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            description: String::new(),
            metric_type: SlaMetricType::Uptime,
            target_value: 99.9,
            window_minutes: 1440,
            created_at: now,
            updated_at: now,
        }
    }
}

impl SlaDefinition {
    pub fn new(name: impl Into<String>, metric_type: SlaMetricType, target_value: f64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            metric_type,
            target_value,
            window_minutes: 1440,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_uptime_sla(&self) -> bool {
        matches!(
            self.metric_type,
            SlaMetricType::Uptime | SlaMetricType::Availability
        )
    }
}

pub fn default_sla_definitions() -> Vec<SlaDefinition> {
    vec![
        SlaDefinition::new("Platform Uptime", SlaMetricType::Uptime, 99.9),
        SlaDefinition::new("API Response Time", SlaMetricType::ResponseTime, 200.0),
        SlaDefinition::new("Error Rate", SlaMetricType::ErrorRate, 1.0),
        SlaDefinition::new("P99 Latency", SlaMetricType::LatencyP99, 500.0),
        SlaDefinition::new("P95 Latency", SlaMetricType::LatencyP95, 250.0),
    ]
}

pub fn compute_sla_status(target: f64, actual: f64, metric_type: SlaMetricType) -> SlaStatus {
    match metric_type {
        SlaMetricType::Uptime | SlaMetricType::Availability => {
            if actual >= target {
                SlaStatus::Met
            } else if actual >= target * 0.99 {
                SlaStatus::AtRisk
            } else {
                SlaStatus::Breached
            }
        }
        SlaMetricType::ResponseTime | SlaMetricType::LatencyP99 | SlaMetricType::LatencyP95 => {
            if actual <= target {
                SlaStatus::Met
            } else if actual <= target * 1.2 {
                SlaStatus::AtRisk
            } else {
                SlaStatus::Breached
            }
        }
        SlaMetricType::ErrorRate => {
            if actual <= target {
                SlaStatus::Met
            } else if actual <= target * 2.0 {
                SlaStatus::AtRisk
            } else {
                SlaStatus::Breached
            }
        }
        SlaMetricType::Throughput => {
            if actual >= target {
                SlaStatus::Met
            } else if actual >= target * 0.8 {
                SlaStatus::AtRisk
            } else {
                SlaStatus::Breached
            }
        }
    }
}

pub fn compute_compliance_percentage(measurements: &[SlaHistoricalEntry]) -> f64 {
    if measurements.is_empty() {
        return 0.0;
    }
    let met_count = measurements.iter().filter(|m| m.status == SlaStatus::Met).count();
    (met_count as f64 / measurements.len() as f64) * 100.0
}

pub fn detect_breach(sla: &SlaDefinition, measurement: &SlaMeasurement) -> Option<SlaBreach> {
    let status = compute_sla_status(sla.target_value, measurement.actual_value, sla.metric_type);
    if status == SlaStatus::Breached {
        Some(SlaBreach {
            id: Uuid::new_v4(),
            sla_id: sla.id,
            sla_name: sla.name.clone(),
            metric_type: sla.metric_type,
            target_value: sla.target_value,
            actual_value: measurement.actual_value,
            detected_at: measurement.measured_at,
            resolved_at: None,
            duration_minutes: None,
            severity: if measurement.actual_value < sla.target_value * 0.5 {
                SlaBreachSeverity::Critical
            } else if measurement.actual_value < sla.target_value * 0.8 {
                SlaBreachSeverity::High
            } else if measurement.actual_value < sla.target_value * 0.95 {
                SlaBreachSeverity::Medium
            } else {
                SlaBreachSeverity::Low
            },
            incident_id: None,
        })
    } else {
        None
    }
}

pub fn generate_dashboard_data(
    slas: &[SlaDefinition],
    current_statuses: &[SlaCurrentStatus],
    recent_breaches: &[SlaBreach],
    compliance_trend: Vec<ComplianceTrendPoint>,
) -> SlaDashboardData {
    let breached = current_statuses
        .iter()
        .filter(|s| s.status == SlaStatus::Breached)
        .count() as u32;
    let at_risk = current_statuses
        .iter()
        .filter(|s| s.status == SlaStatus::AtRisk)
        .count() as u32;

    let overall_status = if breached > 0 {
        SlaStatus::Breached
    } else if at_risk > 0 {
        SlaStatus::AtRisk
    } else {
        SlaStatus::Met
    };

    let overall_compliance = if current_statuses.is_empty() {
        0.0
    } else {
        current_statuses.iter().map(|s| s.compliance_percentage).sum::<f64>()
            / current_statuses.len() as f64
    };

    let current_incidents = recent_breaches
        .iter()
        .filter(|b| b.resolved_at.is_none())
        .count() as u32;

    SlaDashboardData {
        overall_status,
        overall_compliance,
        active_sla_count: slas.len() as u32,
        breached_sla_count: breached,
        at_risk_sla_count: at_risk,
        total_breaches_this_month: recent_breaches.len() as u32,
        current_incidents,
        sla_statuses: current_statuses.to_vec(),
        recent_breaches: recent_breaches.to_vec(),
        compliance_trend,
    }
}

pub fn build_report(
    period: ReportPeriod,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    measurements: &[SlaMeasurement],
    definitions: &[SlaDefinition],
) -> SlaReport {
    let mut sla_results = Vec::new();

    for sla in definitions {
        let relevant: Vec<&SlaMeasurement> = measurements
            .iter()
            .filter(|m| m.sla_id == sla.id)
            .collect();

        if relevant.is_empty() {
            continue;
        }

        let avg_value =
            relevant.iter().map(|m| m.actual_value).sum::<f64>() / relevant.len() as f64;
        let breach_count = relevant
            .iter()
            .filter(|m| {
                compute_sla_status(sla.target_value, m.actual_value, sla.metric_type)
                    == SlaStatus::Breached
            })
            .count() as u32;

        let uptime = if relevant.is_empty() {
            0.0
        } else {
            let met = relevant
                .iter()
                .filter(|m| {
                    compute_sla_status(sla.target_value, m.actual_value, sla.metric_type)
                        == SlaStatus::Met
                })
                .count();
            (met as f64 / relevant.len() as f64) * 100.0
        };

        sla_results.push(SlaResult {
            sla_id: sla.id,
            sla_name: sla.name.clone(),
            metric_type: sla.metric_type,
            target_value: sla.target_value,
            actual_value: avg_value,
            uptime_percentage: uptime,
            breach_count,
            total_downtime_minutes: 0,
            status: compute_sla_status(sla.target_value, avg_value, sla.metric_type),
        });
    }

    let overall_compliance = if sla_results.is_empty() {
        0.0
    } else {
        sla_results.iter().map(|r| r.uptime_percentage).sum::<f64>()
            / sla_results.len() as f64
    };

    let total_breaches: u32 = sla_results.iter().map(|r| r.breach_count).sum();

    SlaReport {
        id: Uuid::new_v4(),
        period,
        period_start: start,
        period_end: end,
        sla_results,
        overall_compliance,
        total_breaches,
        total_incidents: 0,
        generated_at: Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_sla_status_uptime_met() {
        assert_eq!(compute_sla_status(99.9, 99.95, SlaMetricType::Uptime), SlaStatus::Met);
    }

    #[test]
    fn test_sla_status_uptime_breached() {
        assert_eq!(compute_sla_status(99.9, 99.5, SlaMetricType::Uptime), SlaStatus::Breached);
    }

    #[test]
    fn test_sla_status_uptime_at_risk() {
        assert_eq!(compute_sla_status(99.9, 99.85, SlaMetricType::Uptime), SlaStatus::AtRisk);
    }

    #[test]
    fn test_sla_status_response_time_met() {
        assert_eq!(
            compute_sla_status(200.0, 150.0, SlaMetricType::ResponseTime),
            SlaStatus::Met
        );
    }

    #[test]
    fn test_sla_status_response_time_breached() {
        assert_eq!(
            compute_sla_status(200.0, 300.0, SlaMetricType::ResponseTime),
            SlaStatus::Breached
        );
    }

    #[test]
    fn test_sla_status_error_rate_met() {
        assert_eq!(
            compute_sla_status(1.0, 0.5, SlaMetricType::ErrorRate),
            SlaStatus::Met
        );
    }

    #[test]
    fn test_sla_status_error_rate_breached() {
        assert_eq!(
            compute_sla_status(1.0, 3.0, SlaMetricType::ErrorRate),
            SlaStatus::Breached
        );
    }

    #[test]
    fn test_compliance_percentage_empty() {
        assert_eq!(compute_compliance_percentage(&[]), 0.0);
    }

    #[test]
    fn test_compliance_percentage_all_met() {
        let entries = vec![
            SlaHistoricalEntry {
                timestamp: Utc::now(),
                actual_value: 99.95,
                target_value: 99.9,
                status: SlaStatus::Met,
            },
            SlaHistoricalEntry {
                timestamp: Utc::now(),
                actual_value: 99.98,
                target_value: 99.9,
                status: SlaStatus::Met,
            },
        ];
        assert_eq!(compute_compliance_percentage(&entries), 100.0);
    }

    #[test]
    fn test_compliance_percentage_half_met() {
        let entries = vec![
            SlaHistoricalEntry {
                timestamp: Utc::now(),
                actual_value: 99.95,
                target_value: 99.9,
                status: SlaStatus::Met,
            },
            SlaHistoricalEntry {
                timestamp: Utc::now(),
                actual_value: 99.5,
                target_value: 99.9,
                status: SlaStatus::Breached,
            },
        ];
        assert_eq!(compute_compliance_percentage(&entries), 50.0);
    }

    #[test]
    fn test_detect_breach_found() {
        let sla = SlaDefinition::new("Uptime", SlaMetricType::Uptime, 99.9);
        let measurement = SlaMeasurement {
            id: Uuid::new_v4(),
            sla_id: sla.id,
            actual_value: 99.5,
            measured_at: Utc::now(),
        };
        let breach = detect_breach(&sla, &measurement);
        assert!(breach.is_some());
        assert_eq!(breach.unwrap().severity, SlaBreachSeverity::Medium);
    }

    #[test]
    fn test_detect_breach_not_found() {
        let sla = SlaDefinition::new("Uptime", SlaMetricType::Uptime, 99.9);
        let measurement = SlaMeasurement {
            id: Uuid::new_v4(),
            sla_id: sla.id,
            actual_value: 99.95,
            measured_at: Utc::now(),
        };
        assert!(detect_breach(&sla, &measurement).is_none());
    }

    #[test]
    fn test_default_sla_definitions() {
        let defs = default_sla_definitions();
        assert_eq!(defs.len(), 5);
        assert_eq!(defs[0].metric_type, SlaMetricType::Uptime);
        assert_eq!(defs[1].metric_type, SlaMetricType::ResponseTime);
    }

    #[test]
    fn test_sla_metric_type_display() {
        assert_eq!(SlaMetricType::Uptime.display_name(), "Uptime");
        assert_eq!(SlaMetricType::LatencyP99.display_name(), "Latency (p99)");
    }

    #[test]
    fn test_sla_breach_severity_display() {
        assert_eq!(SlaBreachSeverity::Critical.display_name(), "Critical");
        assert_eq!(SlaBreachSeverity::Low.display_name(), "Low");
    }

    #[test]
    fn test_sla_alert_type_display() {
        assert_eq!(SlaAlertType::Breach.display_name(), "Breach");
        assert_eq!(SlaAlertType::Recovery.display_name(), "Recovery");
    }

    #[test]
    fn test_sla_definition_is_uptime() {
        let sla = SlaDefinition::new("Uptime", SlaMetricType::Uptime, 99.9);
        assert!(sla.is_uptime_sla());
        let sla2 = SlaDefinition::new("Error Rate", SlaMetricType::ErrorRate, 1.0);
        assert!(!sla2.is_uptime_sla());
    }

    #[test]
    fn test_generate_dashboard_data() {
        let defs = default_sla_definitions();
        let statuses = vec![SlaCurrentStatus {
            sla_id: defs[0].id,
            sla_name: defs[0].name.clone(),
            metric_type: SlaMetricType::Uptime,
            target_value: 99.9,
            current_value: 99.95,
            status: SlaStatus::Met,
            compliance_percentage: 100.0,
            last_checked_at: Utc::now(),
        }];
        let dashboard = generate_dashboard_data(&defs, &statuses, &vec![], vec![]);
        assert_eq!(dashboard.overall_status, SlaStatus::Met);
        assert_eq!(dashboard.active_sla_count, 5);
    }

    #[test]
    fn test_build_report_empty_measurements() {
        let defs = default_sla_definitions();
        let report = build_report(
            ReportPeriod::Daily,
            Utc::now() - Duration::days(1),
            Utc::now(),
            &[],
            &defs,
        );
        assert_eq!(report.sla_results.len(), 0);
        assert_eq!(report.total_breaches, 0);
    }
}
