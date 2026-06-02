-- 007_add_permissions_tables.sql
-- Phase 8: Full RBAC permission system with deny-overrides and branch protection.

-- Role assignments: who has what role on which resource
CREATE TABLE IF NOT EXISTS member_roles (
    id          BIGSERIAL PRIMARY KEY,
    user_id     BIGINT NOT NULL,
    org_id      BIGINT,
    repo_id     BIGINT,
    role        VARCHAR(20) NOT NULL CHECK (role IN ('owner', 'admin', 'maintainer', 'developer', 'reporter', 'guest')),
    created_by  BIGINT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Exactly one of org_id or repo_id must be set per assignment
    CONSTRAINT chk_member_scope CHECK (
        (org_id IS NOT NULL AND repo_id IS NULL) OR
        (org_id IS NULL AND repo_id IS NOT NULL)
    ),

    -- One user has at most one role per resource
    UNIQUE(user_id, org_id),
    UNIQUE(user_id, repo_id)
);

CREATE INDEX idx_member_roles_user ON member_roles (user_id);
CREATE INDEX idx_member_roles_org ON member_roles (org_id);
CREATE INDEX idx_member_roles_repo ON member_roles (repo_id);
CREATE INDEX idx_member_roles_user_role ON member_roles (user_id, org_id, role);
CREATE INDEX idx_member_roles_user_role_repo ON member_roles (user_id, repo_id, role);

-- Per-repo permission overrides (deny always wins over grant)
CREATE TABLE IF NOT EXISTS repo_policies (
    id          BIGSERIAL PRIMARY KEY,
    repo_id     BIGINT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    role        VARCHAR(20) NOT NULL CHECK (role IN ('owner', 'admin', 'maintainer', 'developer', 'reporter', 'guest')),
    resource    VARCHAR(30) NOT NULL,
    action      VARCHAR(30) NOT NULL,
    effect      VARCHAR(10) NOT NULL CHECK (effect IN ('grant', 'deny')),
    created_by  BIGINT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(repo_id, role, resource, action)
);

CREATE INDEX idx_repo_policies_repo ON repo_policies (repo_id);

-- Branch protection rules
CREATE TABLE IF NOT EXISTS branch_protections (
    id                  BIGSERIAL PRIMARY KEY,
    repo_id             BIGINT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    pattern             VARCHAR(255) NOT NULL,
    push_restricted     BOOLEAN NOT NULL DEFAULT false,
    allowed_roles       JSONB NOT NULL DEFAULT '[]'::jsonb,
    required_reviews   INTEGER,
    require_ci          BOOLEAN NOT NULL DEFAULT false,
    force_push_allowed  BOOLEAN NOT NULL DEFAULT false,
    created_by          BIGINT NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_branch_protections_repo ON branch_protections (repo_id);
CREATE INDEX idx_branch_protections_pattern ON branch_protections (repo_id, pattern);

-- Pipeline variables (encrypted at rest with AES-256-GCM, per-repo key)
CREATE TABLE IF NOT EXISTS pipeline_variables (
    id          BIGSERIAL PRIMARY KEY,
    repo_id     BIGINT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name        VARCHAR(100) NOT NULL,
    value_enc   BYTEA NOT NULL,      -- AES-256-GCM encrypted value
    nonce       BYTEA NOT NULL,      -- GCM nonce
    masked      BOOLEAN NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(repo_id, name)
);

CREATE INDEX idx_pipeline_variables_repo ON pipeline_variables (repo_id);
