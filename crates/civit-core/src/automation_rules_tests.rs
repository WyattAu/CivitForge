#![cfg(test)]

use super::automation_rules::*;
use crate::shared_types::ExecutionResult;
use chrono::Utc;
use uuid::Uuid;

// --- AutomationRule ---

#[test]
fn test_automation_rule_creation() {
    let rule = AutomationRule {
        id: Uuid::new_v4(),
        repo_id: Uuid::new_v4(),
        name: "auto-deploy".into(),
        trigger_type: "push".into(),
        conditions: serde_json::json!({"branch": "main"}),
        actions: serde_json::json!([{"type": "deploy"}]),
        enabled: true,
        created_at: Utc::now(),
    };
    assert_eq!(rule.name, "auto-deploy");
    assert!(rule.enabled);
}

#[test]
fn test_automation_rule_serialization_roundtrip() {
    let rule = AutomationRule {
        id: Uuid::new_v4(),
        repo_id: Uuid::new_v4(),
        name: "test".into(),
        trigger_type: "pr".into(),
        conditions: serde_json::json!({}),
        actions: serde_json::json!([]),
        enabled: false,
        created_at: Utc::now(),
    };
    let json = serde_json::to_string(&rule).unwrap();
    let deserialized: AutomationRule = serde_json::from_str(&json).unwrap();
    assert_eq!(rule.name, deserialized.name);
    assert_eq!(rule.enabled, deserialized.enabled);
}

// --- AutomationRuleV2 (priority) ---

#[test]
fn test_automation_rule_v2_creation() {
    let rule = AutomationRuleV2 {
        id: Uuid::new_v4(),
        repo_id: Uuid::new_v4(),
        name: "v2-rule".into(),
        trigger_type: "push".into(),
        conditions: serde_json::json!({"branch": "main"}),
        actions: serde_json::json!([{"type": "notify"}]),
        priority: 10,
        enabled: true,
        created_at: Utc::now(),
    };
    assert_eq!(rule.priority, 10);
}

// --- AutomationRuleV3 ---

#[test]
fn test_automation_rule_v3_creation() {
    let rule = AutomationRuleV3 {
        id: Uuid::new_v4(),
        repo_id: Uuid::new_v4(),
        name: "v3-rule".into(),
        trigger_type: "push".into(),
        conditions: serde_json::json!({}),
        actions: serde_json::json!([]),
        priority: 5,
        enabled: true,
        last_run_at: Some(Utc::now()),
        run_count: 42,
        created_at: Utc::now(),
    };
    assert_eq!(rule.run_count, 42);
    assert!(rule.last_run_at.is_some());
}

// --- AutomationRuleV4 (success_rate) ---

#[test]
fn test_automation_rule_v4_creation() {
    let rule = AutomationRuleV4 {
        id: Uuid::new_v4(),
        repo_id: Uuid::new_v4(),
        name: "v4-rule".into(),
        trigger_type: "push".into(),
        conditions: serde_json::json!({}),
        actions: serde_json::json!([]),
        priority: 0,
        enabled: true,
        last_run_at: None,
        run_count: 0,
        success_rate: 0.0,
        created_at: Utc::now(),
    };
    assert_eq!(rule.success_rate, 0.0);
}

// --- AutomationRuleV5 (avg_execution_time_ms) ---

#[test]
fn test_automation_rule_v5_creation() {
    let rule = AutomationRuleV5 {
        id: Uuid::new_v4(),
        repo_id: Uuid::new_v4(),
        name: "v5-rule".into(),
        trigger_type: "push".into(),
        conditions: serde_json::json!({}),
        actions: serde_json::json!([]),
        priority: 3,
        enabled: true,
        last_run_at: None,
        run_count: 10,
        success_rate: 85.5,
        avg_execution_time_ms: 150,
        created_at: Utc::now(),
    };
    assert_eq!(rule.avg_execution_time_ms, 150);
}

// --- CreateAutomationRule variants ---

#[test]
fn test_create_automation_rule() {
    let input = CreateAutomationRule {
        repo_id: Uuid::new_v4(),
        name: "new-rule".into(),
        trigger_type: "push".into(),
        conditions: None,
        actions: None,
        enabled: None,
    };
    assert_eq!(input.name, "new-rule");
}

#[test]
fn test_create_automation_rule_v2() {
    let input = CreateAutomationRuleV2 {
        repo_id: Uuid::new_v4(),
        name: "v2".into(),
        trigger_type: "pr".into(),
        conditions: Some(serde_json::json!({"status": "open"})),
        actions: Some(serde_json::json!([{"type": "review"}])),
        priority: Some(5),
        enabled: Some(true),
    };
    assert_eq!(input.priority.unwrap(), 5);
}

#[test]
fn test_create_automation_rule_v3() {
    let input = CreateAutomationRuleV3 {
        repo_id: Uuid::new_v4(),
        name: "v3".into(),
        trigger_type: "push".into(),
        conditions: None,
        actions: None,
        priority: None,
        enabled: Some(true),
    };
    assert!(input.enabled.unwrap());
}

#[test]
fn test_create_automation_rule_v4() {
    let input = CreateAutomationRuleV4 {
        repo_id: Uuid::new_v4(),
        name: "v4".into(),
        trigger_type: "push".into(),
        conditions: None,
        actions: None,
        priority: None,
        enabled: None,
    };
    assert_eq!(input.name, "v4");
}

#[test]
fn test_create_automation_rule_v5() {
    let input = CreateAutomationRuleV5 {
        repo_id: Uuid::new_v4(),
        name: "v5".into(),
        trigger_type: "push".into(),
        conditions: None,
        actions: None,
        priority: Some(10),
        enabled: Some(false),
    };
    assert!(!input.enabled.unwrap());
}

// --- UpdateAutomationRule variants ---

#[test]
fn test_update_automation_rule() {
    let input = UpdateAutomationRule {
        name: Some("updated".into()),
        trigger_type: None,
        conditions: None,
        actions: None,
        enabled: Some(false),
    };
    assert_eq!(input.name.unwrap(), "updated");
    assert!(!input.enabled.unwrap());
}

#[test]
fn test_update_automation_rule_v2() {
    let input = UpdateAutomationRuleV2 {
        name: None,
        trigger_type: Some("cron".into()),
        conditions: None,
        actions: None,
        priority: Some(100),
        enabled: None,
    };
    assert_eq!(input.priority.unwrap(), 100);
}

// --- RuleExecutionRecord ---

#[test]
fn test_rule_execution_record() {
    let record = RuleExecutionRecord {
        id: Uuid::new_v4(),
        rule_id: Uuid::new_v4(),
        status: ExecutionResult::Matched,
        matched_conditions: vec!["branch".into(), "status".into()],
        failed_conditions: vec![],
        actions_executed: vec!["deploy".into(), "notify".into()],
        error: None,
        executed_at: Utc::now(),
    };
    assert_eq!(record.status, ExecutionResult::Matched);
    assert_eq!(record.matched_conditions.len(), 2);
    assert_eq!(record.actions_executed.len(), 2);
}

#[test]
fn test_rule_execution_record_with_error() {
    let record = RuleExecutionRecord {
        id: Uuid::new_v4(),
        rule_id: Uuid::new_v4(),
        status: "error".into(),
        matched_conditions: vec![],
        failed_conditions: vec!["branch".into()],
        actions_executed: vec![],
        error: Some("timeout".into()),
        executed_at: Utc::now(),
    };
    assert!(record.error.is_some());
    assert_eq!(record.failed_conditions.len(), 1);
}

// --- RuleTestResult ---

#[test]
fn test_rule_test_result() {
    let result = RuleTestResult {
        rule_id: Uuid::new_v4(),
        matched: true,
        conditions_met: vec!["a".into()],
        conditions_failed: vec![],
        actions_executed: vec!["deploy".into()],
    };
    assert!(result.matched);
    assert!(result.conditions_failed.is_empty());
}

#[test]
fn test_rule_test_result_not_matched() {
    let result = RuleTestResult {
        rule_id: Uuid::new_v4(),
        matched: false,
        conditions_met: vec![],
        conditions_failed: vec!["branch".into(), "event".into()],
        actions_executed: vec![],
    };
    assert!(!result.matched);
    assert_eq!(result.conditions_failed.len(), 2);
}

// --- RulePerformanceMetrics ---

#[test]
fn test_rule_performance_metrics() {
    let metrics = RulePerformanceMetrics {
        rule_id: Uuid::new_v4(),
        total_runs: 100,
        successful_runs: 85,
        failed_runs: 15,
        average_execution_time_ms: 120.5,
        last_execution_time_ms: Some(95.0),
        success_rate: 85.0,
    };
    assert_eq!(metrics.total_runs, 100);
    assert!((metrics.success_rate - 85.0).abs() < f64::EPSILON);
}

#[test]
fn test_rule_performance_metrics_zero_runs() {
    let metrics = RulePerformanceMetrics {
        rule_id: Uuid::new_v4(),
        total_runs: 0,
        successful_runs: 0,
        failed_runs: 0,
        average_execution_time_ms: 0.0,
        last_execution_time_ms: None,
        success_rate: 0.0,
    };
    assert_eq!(metrics.total_runs, 0);
    assert!(metrics.last_execution_time_ms.is_none());
}

// --- RuleRecommendation ---

#[test]
fn test_rule_recommendation() {
    let rec = RuleRecommendation {
        rule_id: Uuid::new_v4(),
        recommendation_type: "low_success_rate".into(),
        description: "Rule success rate is below 50%".into(),
        confidence: 0.85,
        suggested_changes: serde_json::json!({"action": "review_conditions"}),
    };
    assert_eq!(rec.recommendation_type, "low_success_rate");
    assert!(rec.confidence > 0.8);
}

#[test]
fn test_rule_recommendation_high_performance() {
    let rec = RuleRecommendation {
        rule_id: Uuid::new_v4(),
        recommendation_type: "high_performance".into(),
        description: "Rule is performing well".into(),
        confidence: 0.9,
        suggested_changes: serde_json::json!({"action": "increase_priority"}),
    };
    assert_eq!(rec.recommendation_type, "high_performance");
}

// --- V6-V10 rule structs ---

#[test]
fn test_automation_rule_v6() {
    let rule = AutomationRuleV6 {
        id: Uuid::new_v4(),
        repo_id: Uuid::new_v4(),
        name: "v6".into(),
        trigger_type: "push".into(),
        conditions: serde_json::json!({}),
        actions: serde_json::json!([]),
        priority: 1,
        enabled: true,
        last_run_at: None,
        run_count: 0,
        success_rate: 0.0,
        avg_execution_time_ms: 0,
        created_at: Utc::now(),
    };
    assert_eq!(rule.name, "v6");
}

#[test]
fn test_automation_rule_v7() {
    let rule = AutomationRuleV7 {
        id: Uuid::new_v4(),
        repo_id: Uuid::new_v4(),
        name: "v7".into(),
        trigger_type: "push".into(),
        conditions: serde_json::json!({}),
        actions: serde_json::json!([]),
        priority: 1,
        enabled: true,
        last_run_at: None,
        run_count: 0,
        success_rate: 0.0,
        avg_execution_time_ms: 0,
        created_at: Utc::now(),
    };
    assert_eq!(rule.name, "v7");
}

#[test]
fn test_automation_rule_v8() {
    let rule = AutomationRuleV8 {
        id: Uuid::new_v4(),
        repo_id: Uuid::new_v4(),
        name: "v8".into(),
        trigger_type: "push".into(),
        conditions: serde_json::json!({}),
        actions: serde_json::json!([]),
        priority: 1,
        enabled: true,
        last_run_at: None,
        run_count: 0,
        success_rate: 0.0,
        avg_execution_time_ms: 0,
        created_at: Utc::now(),
    };
    assert_eq!(rule.name, "v8");
}

#[test]
fn test_automation_rule_v9() {
    let rule = AutomationRuleV9 {
        id: Uuid::new_v4(),
        repo_id: Uuid::new_v4(),
        name: "v9".into(),
        trigger_type: "push".into(),
        conditions: serde_json::json!({}),
        actions: serde_json::json!([]),
        priority: 1,
        enabled: true,
        last_run_at: None,
        run_count: 0,
        success_rate: 0.0,
        avg_execution_time_ms: 0,
        created_at: Utc::now(),
    };
    assert_eq!(rule.name, "v9");
}

#[test]
fn test_automation_rule_v10() {
    let rule = AutomationRuleV10 {
        id: Uuid::new_v4(),
        repo_id: Uuid::new_v4(),
        name: "v10".into(),
        trigger_type: "push".into(),
        conditions: serde_json::json!({}),
        actions: serde_json::json!([]),
        priority: 1,
        enabled: true,
        last_run_at: None,
        run_count: 0,
        success_rate: 0.0,
        avg_execution_time_ms: 0,
        created_at: Utc::now(),
    };
    assert_eq!(rule.name, "v10");
}

// --- Create/Update for V6-V10 ---

#[test]
fn test_create_automation_rule_v6() {
    let input = CreateAutomationRuleV6 {
        repo_id: Uuid::new_v4(),
        name: "v6".into(),
        trigger_type: "push".into(),
        conditions: None,
        actions: None,
        priority: None,
        enabled: None,
    };
    assert_eq!(input.name, "v6");
}

#[test]
fn test_create_automation_rule_v7() {
    let input = CreateAutomationRuleV7 {
        repo_id: Uuid::new_v4(),
        name: "v7".into(),
        trigger_type: "push".into(),
        conditions: None,
        actions: None,
        priority: None,
        enabled: None,
    };
    assert_eq!(input.name, "v7");
}

#[test]
fn test_update_automation_rule_v3() {
    let input = UpdateAutomationRuleV3 {
        name: Some("v3-updated".into()),
        trigger_type: None,
        conditions: None,
        actions: None,
        priority: None,
        enabled: None,
    };
    assert_eq!(input.name.unwrap(), "v3-updated");
}

#[test]
fn test_update_automation_rule_v4() {
    let input = UpdateAutomationRuleV4 {
        name: None,
        trigger_type: Some("cron".into()),
        conditions: None,
        actions: None,
        priority: Some(7),
        enabled: Some(true),
    };
    assert_eq!(input.priority.unwrap(), 7);
}

#[test]
fn test_update_automation_rule_v5() {
    let input = UpdateAutomationRuleV5 {
        name: None,
        trigger_type: None,
        conditions: Some(serde_json::json!({"env": "prod"})),
        actions: None,
        priority: None,
        enabled: Some(true),
    };
    assert!(input.conditions.is_some());
}

#[test]
fn test_update_automation_rule_v6() {
    let input = UpdateAutomationRuleV6 {
        name: Some("v6-updated".into()),
        trigger_type: None,
        conditions: None,
        actions: None,
        priority: None,
        enabled: None,
    };
    assert_eq!(input.name.unwrap(), "v6-updated");
}

#[test]
fn test_update_automation_rule_v7() {
    let input = UpdateAutomationRuleV7 {
        name: None,
        trigger_type: None,
        conditions: None,
        actions: Some(serde_json::json!([{"type": "deploy"}])),
        priority: Some(20),
        enabled: None,
    };
    assert_eq!(input.priority.unwrap(), 20);
}

// --- V13 create/update ---

#[test]
fn test_create_automation_rule_v13() {
    let input = CreateAutomationRuleV13 {
        repo_id: Uuid::new_v4(),
        name: "v13".into(),
        trigger_type: "push".into(),
        conditions: None,
        actions: None,
        priority: None,
        enabled: None,
    };
    assert_eq!(input.name, "v13");
}

#[test]
fn test_update_automation_rule_v13() {
    let input = UpdateAutomationRuleV13 {
        name: Some("v13-updated".into()),
        trigger_type: None,
        conditions: None,
        actions: None,
        priority: None,
        enabled: None,
    };
    assert_eq!(input.name.unwrap(), "v13-updated");
}

// --- V9/V10 create/update ---

#[test]
fn test_create_automation_rule_v9() {
    let input = CreateAutomationRuleV9 {
        repo_id: Uuid::new_v4(),
        name: "v9".into(),
        trigger_type: "push".into(),
        conditions: None,
        actions: None,
        priority: None,
        enabled: None,
    };
    assert_eq!(input.name, "v9");
}

#[test]
fn test_create_automation_rule_v10() {
    let input = CreateAutomationRuleV10 {
        repo_id: Uuid::new_v4(),
        name: "v10".into(),
        trigger_type: "push".into(),
        conditions: None,
        actions: None,
        priority: None,
        enabled: None,
    };
    assert_eq!(input.name, "v10");
}

#[test]
fn test_update_automation_rule_v9() {
    let input = UpdateAutomationRuleV9 {
        name: Some("v9-up".into()),
        trigger_type: None,
        conditions: None,
        actions: None,
        priority: None,
        enabled: None,
    };
    assert_eq!(input.name.unwrap(), "v9-up");
}

#[test]
fn test_update_automation_rule_v10() {
    let input = UpdateAutomationRuleV10 {
        name: None,
        trigger_type: Some("pr".into()),
        conditions: None,
        actions: None,
        priority: None,
        enabled: None,
    };
    assert_eq!(input.trigger_type.unwrap(), "pr");
}
