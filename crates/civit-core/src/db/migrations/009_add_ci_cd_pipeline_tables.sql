-- CivitForge Phase 9: CI/CD Pipeline Backend
-- Migration 009
-- Adds runners, pipeline definitions (multi-job), pipeline runs,
-- run jobs, run steps, and runner auth tokens.

-- -----------------------------------------------------------------------
-- Runners: self-hosted CI/CD runner instances
-- -----------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS runners (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    -- Runner scope: 'global', 'org', 'repo'
    scope VARCHAR(20) NOT NULL DEFAULT 'global',
    -- FK to owning entity (NULL for global runners)
    org_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    repo_id UUID REFERENCES repositories(id) ON DELETE CASCADE,
    -- Runner labels for job matching (e.g. ["linux", "amd64", "docker"])
    labels JSONB NOT NULL DEFAULT '[]',
    -- Runner status
    status VARCHAR(20) NOT NULL DEFAULT 'offline',
    -- Last heartbeat timestamp
    last_seen_at TIMESTAMPTZ,
    -- Token for runner auth (hashed)
    token_hash VARCHAR(128) NOT NULL UNIQUE,
    -- Group for distributed runners (runners in same group share load)
    runner_group VARCHAR(255),
    -- Who registered this runner
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Exactly one of org_id/repo_id must be set for scoped runners
    CONSTRAINT chk_runner_scope CHECK (
        (scope = 'global' AND org_id IS NULL AND repo_id IS NULL)
        OR (scope = 'org' AND org_id IS NOT NULL AND repo_id IS NULL)
        OR (scope = 'repo' AND repo_id IS NOT NULL AND org_id IS NULL)
    )
);

-- -----------------------------------------------------------------------
-- Pipeline definitions: parsed YAML stored per repository
-- -----------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS pipeline_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    -- File path in repo (e.g. '.civit/pipeline.yaml')
    yaml_path VARCHAR(512) NOT NULL DEFAULT '.civit/pipeline.yaml',
    -- Ref (branch/tag SHA) this definition was parsed from
    ref_name VARCHAR(255) NOT NULL,
    commit_sha VARCHAR(64) NOT NULL,
    -- Raw YAML content
    yaml_content TEXT NOT NULL,
    -- Schema version
    version VARCHAR(10) NOT NULL DEFAULT '1',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- No UNIQUE on (repo_id, ref_name) since different refs can have different configs
    UNIQUE(repo_id, ref_name)
);

-- -----------------------------------------------------------------------
-- Pipeline jobs: individual jobs within a pipeline definition
-- -----------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS pipeline_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    definition_id UUID NOT NULL REFERENCES pipeline_definitions(id) ON DELETE CASCADE,
    -- Job name from YAML
    name VARCHAR(255) NOT NULL,
    -- Job ordering: serial index within the definition
    job_index INT NOT NULL,
    -- Dependencies (other job names in same definition)
    needs JSONB NOT NULL DEFAULT '[]',
    -- Runner target labels
    runs_on JSONB,
    -- Timeout (ISO 8601 duration string, e.g. "30m")
    timeout VARCHAR(20),
    -- Condition (CEL expression)
    condition TEXT,
    -- Services (sidecar containers) as JSON
    services JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(definition_id, name)
);

-- -----------------------------------------------------------------------
-- Pipeline job steps: individual steps within a job
-- -----------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS pipeline_job_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id UUID NOT NULL REFERENCES pipeline_jobs(id) ON DELETE CASCADE,
    -- Step ordering within job
    step_index INT NOT NULL,
    -- Step name
    name VARCHAR(255) NOT NULL,
    -- Step type: 'run' (shell commands), 'uses' (action)
    step_type VARCHAR(20) NOT NULL DEFAULT 'run',
    -- Shell commands (for run type)
    commands JSONB,
    -- Action identifier (for uses type)
    action VARCHAR(255),
    -- Action parameters (for uses type)
    action_params JSONB,
    -- Container image override
    image VARCHAR(512),
    -- Working directory
    workdir VARCHAR(512) NOT NULL DEFAULT '',
    -- Environment variables (non-secret)
    env JSONB NOT NULL DEFAULT '{}',
    -- Secret names to inject
    secrets JSONB NOT NULL DEFAULT '[]',
    -- Continue on error
    continue_on_error BOOLEAN NOT NULL DEFAULT false,
    -- Step-level timeout
    timeout VARCHAR(20),
    -- Step condition
    condition TEXT,
    -- Checkout config
    checkout_config JSONB,
    -- Cache config
    cache_config JSONB,
    -- Artifact config
    artifact_config JSONB,
    -- Retry config
    retry_config JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(job_id, step_index)
);

-- -----------------------------------------------------------------------
-- Pipeline runs: execution instances triggered by events
-- -----------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS pipeline_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    definition_id UUID NOT NULL REFERENCES pipeline_definitions(id) ON DELETE CASCADE,
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    -- Who/what triggered this run
    trigger VARCHAR(50) NOT NULL DEFAULT 'push',
    -- Branch/tag ref that triggered
    ref_name VARCHAR(255),
    commit_sha VARCHAR(64) NOT NULL,
    -- Event context (JSON: branch, tag, changed_files, etc.)
    event_context JSONB NOT NULL DEFAULT '{}',
    -- Run status: pending, queued, running, success, failure, canceled, skipped
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    -- Concurrency group (from YAML)
    concurrency_group VARCHAR(255),
    -- Workflow dispatch inputs (if manual trigger)
    dispatch_inputs JSONB,
    -- Who triggered (user_id or NULL for system events)
    triggered_by UUID REFERENCES users(id),
    -- Assigned runner (NULL if not yet assigned)
    runner_id UUID REFERENCES runners(id),
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    queued_at TIMESTAMPTZ,
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ
);

-- -----------------------------------------------------------------------
-- Pipeline run jobs: per-job execution within a run
-- -----------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS pipeline_run_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id UUID NOT NULL REFERENCES pipeline_runs(id) ON DELETE CASCADE,
    job_id UUID NOT NULL REFERENCES pipeline_jobs(id),
    -- Job name (denormalized for fast queries)
    name VARCHAR(255) NOT NULL,
    -- Job status
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    -- Assigned runner
    runner_id UUID REFERENCES runners(id),
    -- Job-level outputs
    outputs JSONB NOT NULL DEFAULT '{}',
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    UNIQUE(run_id, job_id)
);

-- -----------------------------------------------------------------------
-- Pipeline run steps: per-step execution within a run job
-- -----------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS pipeline_run_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    run_job_id UUID NOT NULL REFERENCES pipeline_run_jobs(id) ON DELETE CASCADE,
    step_id UUID NOT NULL REFERENCES pipeline_job_steps(id),
    -- Step name (denormalized)
    name VARCHAR(255) NOT NULL,
    -- Step index within job
    step_index INT NOT NULL,
    -- Step status
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    -- Container image used
    image VARCHAR(512),
    -- Exit code
    exit_code INT,
    -- Step output / logs (stored in text, large outputs go to object storage)
    output TEXT,
    -- Started timestamp
    started_at TIMESTAMPTZ,
    -- Finished timestamp
    finished_at TIMESTAMPTZ,
    UNIQUE(run_job_id, step_id)
);

-- -----------------------------------------------------------------------
-- Indexes for performance
-- -----------------------------------------------------------------------
CREATE INDEX IF NOT EXISTS idx_pipeline_defs_repo_ref ON pipeline_definitions(repo_id, ref_name);
CREATE INDEX IF NOT EXISTS idx_pipeline_runs_def ON pipeline_runs(definition_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_runs_repo ON pipeline_runs(repo_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_runs_status ON pipeline_runs(status);
CREATE INDEX IF NOT EXISTS idx_pipeline_runs_trigger ON pipeline_runs(trigger);
CREATE INDEX IF NOT EXISTS idx_runners_scope_org ON runners(scope, org_id);
CREATE INDEX IF NOT EXISTS idx_runners_scope_repo ON runners(scope, repo_id);
CREATE INDEX IF NOT EXISTS idx_runners_status ON runners(status);
CREATE INDEX IF NOT EXISTS idx_runners_group ON runners(runner_group);
CREATE INDEX IF NOT EXISTS idx_pipeline_jobs_def ON pipeline_jobs(definition_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_job_steps_job ON pipeline_job_steps(job_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_run_jobs_run ON pipeline_run_jobs(run_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_run_jobs_status ON pipeline_run_jobs(status);
CREATE INDEX IF NOT EXISTS idx_pipeline_run_steps_run_job ON pipeline_run_steps(run_job_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_run_steps_status ON pipeline_run_steps(status);
