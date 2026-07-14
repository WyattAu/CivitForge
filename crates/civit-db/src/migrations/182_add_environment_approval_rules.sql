-- CivitForge Phase 182: Environment Approval Rules & Approvals
-- Migration 182
-- Adds approval rule management, approval tracking, and auto-approval.

CREATE TABLE IF NOT EXISTS environment_approval_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id UUID NOT NULL REFERENCES pipeline_environments(id) ON DELETE CASCADE,
    required_approvers INTEGER NOT NULL DEFAULT 1,
    approver_groups TEXT[] NOT NULL DEFAULT '{}',
    auto_approve_after_hours INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS environment_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    environment_id UUID NOT NULL REFERENCES pipeline_environments(id),
    deployment_id UUID NOT NULL,
    approver_id UUID NOT NULL REFERENCES users(id),
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_environment_approval_rules_env_id ON environment_approval_rules(environment_id);
CREATE INDEX IF NOT EXISTS idx_environment_approvals_env_id ON environment_approvals(environment_id);
CREATE INDEX IF NOT EXISTS idx_environment_approvals_deployment_id ON environment_approvals(deployment_id);
CREATE INDEX IF NOT EXISTS idx_environment_approvals_approver_id ON environment_approvals(approver_id);
CREATE INDEX IF NOT EXISTS idx_environment_approvals_status ON environment_approvals(status);
