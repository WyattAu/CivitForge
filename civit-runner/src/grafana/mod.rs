#![forbid(unsafe_code)]

pub mod dashboards;

pub use dashboards::{
    AlertCondition, AlertConditionOperator, AlertRule, AlertRuleTemplate, AlertSeverity, Dashboard,
    DashboardTemplate, GridPos, Panel, PanelTarget, PanelType, PredefinedAlerts,
    PredefinedDashboards,
};
