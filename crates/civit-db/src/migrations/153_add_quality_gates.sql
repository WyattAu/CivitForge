-- Migration 153: Code Quality Gates
-- Adds quality gates with enforcement and finding tracking.

CREATE TABLE IF NOT EXISTS quality_gates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    conditions JSONB NOT NULL DEFAULT '{}',
    actions JSONB NOT NULL DEFAULT '[]',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS quality_gate_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    gate_id UUID NOT NULL REFERENCES quality_gates(id) ON DELETE CASCADE,
    pr_id UUID REFERENCES pull_requests(id),
    status TEXT NOT NULL DEFAULT 'pending',
    findings JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_quality_gates_repo ON quality_gates(repo_id);
CREATE INDEX IF NOT EXISTS idx_quality_gate_results_gate ON quality_gate_results(gate_id);
CREATE INDEX IF NOT EXISTS idx_quality_gate_results_pr ON quality_gate_results(pr_id);
CREATE INDEX IF NOT EXISTS idx_quality_gate_results_status ON quality_gate_results(status);
