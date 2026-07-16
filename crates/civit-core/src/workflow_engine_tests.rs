#![cfg(test)]

use super::workflow_engine::*;
use chrono::Utc;
use uuid::Uuid;

// --- Workflow struct creation and serialization ---

#[test]
fn test_workflow_struct_creation() {
    let wf = Workflow {
        id: Uuid::new_v4(),
        name: "test-workflow".into(),
        description: "desc".into(),
        trigger_type: "push".into(),
        trigger_config: serde_json::json!({"branch": "main"}),
        steps: serde_json::json!([{"name": "step1"}]),
        enabled: true,
        created_at: Utc::now(),
    };
    assert_eq!(wf.name, "test-workflow");
    assert!(wf.enabled);
}

#[test]
fn test_workflow_serialization_roundtrip() {
    let wf = Workflow {
        id: Uuid::new_v4(),
        name: "wf1".into(),
        description: "d".into(),
        trigger_type: "cron".into(),
        trigger_config: serde_json::json!({}),
        steps: serde_json::json!([]),
        enabled: false,
        created_at: Utc::now(),
    };
    let json = serde_json::to_string(&wf).unwrap();
    let deserialized: Workflow = serde_json::from_str(&json).unwrap();
    assert_eq!(wf.name, deserialized.name);
    assert_eq!(wf.enabled, deserialized.enabled);
}

// --- WorkflowTrigger ---

#[test]
fn test_workflow_trigger_creation() {
    let trigger = WorkflowTrigger {
        id: Uuid::new_v4(),
        workflow_id: Uuid::new_v4(),
        trigger_type: "webhook".into(),
        trigger_config: serde_json::json!({"url": "/hook"}),
        enabled: true,
        created_at: Utc::now(),
    };
    assert_eq!(trigger.trigger_type, "webhook");
    assert!(trigger.enabled);
}

// --- WorkflowAction ---

#[test]
fn test_workflow_action_creation() {
    let action = WorkflowAction {
        id: Uuid::new_v4(),
        workflow_id: Uuid::new_v4(),
        action_type: "notify".into(),
        action_config: serde_json::json!({"channel": "#ops"}),
        order_index: 0,
        enabled: true,
        created_at: Utc::now(),
    };
    assert_eq!(action.action_type, "notify");
    assert_eq!(action.order_index, 0);
}

// --- WorkflowRun ---

#[test]
fn test_workflow_run_creation() {
    let run = WorkflowRun {
        id: Uuid::new_v4(),
        workflow_id: Uuid::new_v4(),
        status: "running".into(),
        current_step: 0,
        total_steps: 3,
        started_at: Utc::now(),
        completed_at: None,
    };
    assert_eq!(run.status, "running");
    assert_eq!(run.total_steps, 3);
    assert!(run.completed_at.is_none());
}

#[test]
fn test_workflow_run_completed() {
    let run = WorkflowRun {
        id: Uuid::new_v4(),
        workflow_id: Uuid::new_v4(),
        status: "completed".into(),
        current_step: 3,
        total_steps: 3,
        started_at: Utc::now(),
        completed_at: Some(Utc::now()),
    };
    assert!(run.completed_at.is_some());
}

// --- CreateWorkflow / UpdateWorkflow ---

#[test]
fn test_create_workflow_input() {
    let input = CreateWorkflow {
        name: "deploy".into(),
        description: Some("Deploy pipeline".into()),
        trigger_type: "push".into(),
        trigger_config: Some(serde_json::json!({"branch": "main"})),
        steps: Some(serde_json::json!([{"name": "build"}, {"name": "test"}])),
        enabled: Some(true),
    };
    assert_eq!(input.name, "deploy");
    assert!(input.description.is_some());
    assert!(input.trigger_config.is_some());
}

#[test]
fn test_update_workflow_input_defaults() {
    let input = UpdateWorkflow {
        name: None,
        description: None,
        trigger_type: None,
        trigger_config: None,
        steps: None,
        enabled: None,
    };
    assert!(input.name.is_none());
    assert!(input.enabled.is_none());
}

#[test]
fn test_update_workflow_input_with_values() {
    let input = UpdateWorkflow {
        name: Some("new-name".into()),
        description: Some("new-desc".into()),
        trigger_type: Some("cron".into()),
        trigger_config: Some(serde_json::json!({"schedule": "0 * * * *"})),
        steps: Some(serde_json::json!([])),
        enabled: Some(false),
    };
    assert_eq!(input.name.unwrap(), "new-name");
    assert!(!input.enabled.unwrap());
}

// --- WorkflowStepResult ---

#[test]
fn test_workflow_step_result() {
    let result = WorkflowStepResult {
        step_index: 1,
        status: "completed".into(),
        output: Some(serde_json::json!({"key": "value"})),
        error: None,
        started_at: Utc::now(),
        completed_at: Some(Utc::now()),
    };
    assert_eq!(result.step_index, 1);
    assert!(result.error.is_none());
}

#[test]
fn test_workflow_step_result_with_error() {
    let result = WorkflowStepResult {
        step_index: 0,
        status: "failed".into(),
        output: None,
        error: Some("timeout".into()),
        started_at: Utc::now(),
        completed_at: None,
    };
    assert!(result.error.is_some());
    assert_eq!(result.error.unwrap(), "timeout");
}

// --- ActionChainResult ---

#[test]
fn test_action_chain_result() {
    let result = ActionChainResult {
        action_id: Uuid::new_v4(),
        action_type: "deploy".into(),
        status: "completed".into(),
        output: Some(serde_json::json!({"deployed": true})),
        error: None,
        started_at: Utc::now(),
        completed_at: Some(Utc::now()),
    };
    assert_eq!(result.action_type, "deploy");
}

// --- WorkflowTemplate ---

#[test]
fn test_workflow_template_creation() {
    let template = WorkflowTemplate {
        id: Uuid::new_v4(),
        name: "ci-template".into(),
        description: "CI/CD pipeline".into(),
        template_type: "ci".into(),
        config: serde_json::json!({"steps": []}),
        is_public: true,
        author_id: Some(Uuid::new_v4()),
        usage_count: 0,
        created_at: Utc::now(),
    };
    assert!(template.is_public);
    assert_eq!(template.usage_count, 0);
}

// --- WorkflowTemplateV2 ---

#[test]
fn test_workflow_template_v2_creation() {
    let template = WorkflowTemplateV2 {
        id: Uuid::new_v4(),
        name: "ml-pipeline".into(),
        description: "ML training".into(),
        template_type: "ml".into(),
        config: serde_json::json!({}),
        is_public: false,
        author_id: None,
        usage_count: 42,
        rating: 4.5,
        created_at: Utc::now(),
    };
    assert_eq!(template.rating, 4.5);
    assert_eq!(template.usage_count, 42);
}

// --- WorkflowTemplateReview ---

#[test]
fn test_workflow_template_review() {
    let review = WorkflowTemplateReview {
        id: Uuid::new_v4(),
        template_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        rating: 5,
        review: "Excellent template".into(),
        created_at: Utc::now(),
    };
    assert_eq!(review.rating, 5);
}

// --- WorkflowTemplateReviewV2 (helpful_count) ---

#[test]
fn test_workflow_template_review_v2() {
    let review = WorkflowTemplateReviewV2 {
        id: Uuid::new_v4(),
        template_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        rating: 4,
        review: "Good but could be better".into(),
        helpful_count: 12,
        created_at: Utc::now(),
    };
    assert_eq!(review.helpful_count, 12);
}

// --- WorkflowTemplateAnalytics ---

#[test]
fn test_workflow_template_analytics() {
    let analytics = WorkflowTemplateAnalytics {
        template_id: Uuid::new_v4(),
        total_usage: 150,
        avg_rating: 4.2,
        total_reviews: 30,
    };
    assert_eq!(analytics.total_usage, 150);
    assert!((analytics.avg_rating - 4.2).abs() < f64::EPSILON);
}

// --- WorkflowTemplateRecommendation ---

#[test]
fn test_workflow_template_recommendation() {
    let rec = WorkflowTemplateRecommendation {
        template_id: Uuid::new_v4(),
        recommendation_type: "featured_candidate".into(),
        description: "High rating and usage".into(),
        confidence: 0.85,
        suggested_changes: serde_json::json!({"action": "feature"}),
    };
    assert!(rec.confidence > 0.8);
    assert_eq!(rec.recommendation_type, "featured_candidate");
}

// --- WorkflowExecution ---

#[test]
fn test_workflow_execution_creation() {
    let execution = WorkflowExecution {
        id: Uuid::new_v4(),
        workflow_id: Uuid::new_v4(),
        trigger_id: Some(Uuid::new_v4()),
        status: "running".into(),
        input: serde_json::json!({"key": "val"}),
        output: serde_json::json!({}),
        started_at: Utc::now(),
        completed_at: None,
    };
    assert_eq!(execution.status, "running");
    assert!(execution.completed_at.is_none());
}

// --- WorkflowExecutionStep ---

#[test]
fn test_workflow_execution_step() {
    let step = WorkflowExecutionStep {
        id: Uuid::new_v4(),
        execution_id: Uuid::new_v4(),
        action_id: Uuid::new_v4(),
        status: "completed".into(),
        input: serde_json::json!({}),
        output: serde_json::json!({"result": "ok"}),
        error: None,
        started_at: Utc::now(),
        completed_at: Some(Utc::now()),
    };
    assert!(step.error.is_none());
}

// --- WorkflowExecutionStats ---

#[test]
fn test_workflow_execution_stats() {
    let stats = WorkflowExecutionStats {
        workflow_id: Uuid::new_v4(),
        total_executions: 100,
        successful_executions: 95,
        failed_executions: 5,
        average_execution_time_ms: 150.0,
        last_execution_time_ms: Some(120.0),
        success_rate: 95.0,
    };
    assert_eq!(stats.success_rate, 95.0);
    assert_eq!(stats.failed_executions, 5);
}

// --- CreateWorkflowTemplate / UpdateWorkflowTemplate ---

#[test]
fn test_create_workflow_template_input() {
    let input = CreateWorkflowTemplate {
        name: "deploy-template".into(),
        description: Some("Deploy pipeline template".into()),
        template_type: "deploy".into(),
        config: Some(serde_json::json!({"steps": [{"name": "build"}]})),
        is_public: Some(true),
        author_id: Some(Uuid::new_v4()),
    };
    assert_eq!(input.name, "deploy-template");
}

#[test]
fn test_update_workflow_template_input() {
    let input = UpdateWorkflowTemplate {
        name: Some("updated-name".into()),
        description: None,
        template_type: Some("ci".into()),
        config: None,
        is_public: Some(false),
    };
    assert_eq!(input.name.unwrap(), "updated-name");
    assert!(!input.is_public.unwrap());
}

// --- CreateWorkflowExecution ---

#[test]
fn test_create_workflow_execution_input() {
    let input = CreateWorkflowExecution {
        workflow_id: Uuid::new_v4(),
        trigger_id: Some(Uuid::new_v4()),
        input: Some(serde_json::json!({"param": 42})),
    };
    assert!(input.trigger_id.is_some());
}

// --- WorkflowTemplateV3 (with helpful_count reviews) ---

#[test]
fn test_workflow_template_v3_creation() {
    let template = WorkflowTemplateV3 {
        id: Uuid::new_v4(),
        name: "v3-template".into(),
        description: "Template V3".into(),
        template_type: "generic".into(),
        config: serde_json::json!({}),
        is_public: true,
        author_id: None,
        usage_count: 10,
        rating: 4.0,
        created_at: Utc::now(),
    };
    assert_eq!(template.usage_count, 10);
}

// --- V4/V5/V6/V7/V8 template structs ---

#[test]
fn test_workflow_template_v4() {
    let t = WorkflowTemplateV4 {
        id: Uuid::new_v4(),
        name: "v4".into(),
        description: "d".into(),
        template_type: "t".into(),
        config: serde_json::json!({}),
        is_public: false,
        author_id: None,
        usage_count: 0,
        rating: 0.0,
        created_at: Utc::now(),
    };
    assert_eq!(t.name, "v4");
}

#[test]
fn test_workflow_template_v5() {
    let t = WorkflowTemplateV5 {
        id: Uuid::new_v4(),
        name: "v5".into(),
        description: "d".into(),
        template_type: "t".into(),
        config: serde_json::json!({}),
        is_public: true,
        author_id: None,
        usage_count: 5,
        rating: 3.5,
        created_at: Utc::now(),
    };
    assert_eq!(t.usage_count, 5);
}

#[test]
fn test_workflow_template_v6() {
    let t = WorkflowTemplateV6 {
        id: Uuid::new_v4(),
        name: "v6".into(),
        description: "d".into(),
        template_type: "t".into(),
        config: serde_json::json!({}),
        is_public: false,
        author_id: None,
        usage_count: 0,
        rating: 0.0,
        created_at: Utc::now(),
    };
    assert!(!t.is_public);
}

#[test]
fn test_workflow_template_v7() {
    let t = WorkflowTemplateV7 {
        id: Uuid::new_v4(),
        name: "v7".into(),
        description: "d".into(),
        template_type: "t".into(),
        config: serde_json::json!({}),
        is_public: true,
        author_id: None,
        usage_count: 0,
        rating: 0.0,
        created_at: Utc::now(),
    };
    assert!(t.is_public);
}

// --- WorkflowTemplateUsage ---

#[test]
fn test_workflow_template_usage() {
    let usage = WorkflowTemplateUsage {
        id: Uuid::new_v4(),
        template_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        used_at: Utc::now(),
    };
    assert_eq!(usage.template_id, usage.template_id);
}

// --- All review V3-V6 ---

#[test]
fn test_workflow_template_review_v3() {
    let r = WorkflowTemplateReviewV3 {
        id: Uuid::new_v4(),
        template_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        rating: 3,
        review: "ok".into(),
        helpful_count: 5,
        created_at: Utc::now(),
    };
    assert_eq!(r.helpful_count, 5);
}

#[test]
fn test_workflow_template_review_v4() {
    let r = WorkflowTemplateReviewV4 {
        id: Uuid::new_v4(),
        template_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        rating: 4,
        review: "good".into(),
        helpful_count: 10,
        created_at: Utc::now(),
    };
    assert_eq!(r.rating, 4);
}

#[test]
fn test_workflow_template_review_v5() {
    let r = WorkflowTemplateReviewV5 {
        id: Uuid::new_v4(),
        template_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        rating: 5,
        review: "great".into(),
        helpful_count: 20,
        created_at: Utc::now(),
    };
    assert_eq!(r.rating, 5);
}

#[test]
fn test_workflow_template_review_v6() {
    let r = WorkflowTemplateReviewV6 {
        id: Uuid::new_v4(),
        template_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        rating: 2,
        review: "meh".into(),
        helpful_count: 0,
        created_at: Utc::now(),
    };
    assert_eq!(r.helpful_count, 0);
}

// --- Create input variants ---

#[test]
fn test_create_workflow_template_v2_input() {
    let input = CreateWorkflowTemplateV2 {
        name: "v2-tpl".into(),
        description: Some("desc".into()),
        template_type: "deploy".into(),
        config: None,
        is_public: None,
        author_id: None,
    };
    assert_eq!(input.name, "v2-tpl");
}

#[test]
fn test_update_workflow_template_v2_input() {
    let input = UpdateWorkflowTemplateV2 {
        name: Some("new".into()),
        description: None,
        template_type: None,
        config: None,
        is_public: Some(true),
    };
    assert!(input.is_public.unwrap());
}
