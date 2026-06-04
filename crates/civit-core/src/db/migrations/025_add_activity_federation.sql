-- Migration 025: Activity feed + Federation tables

-- Activity events for the platform activity feed
CREATE TABLE IF NOT EXISTS activity_events (
    id          BIGSERIAL PRIMARY KEY,
    actor_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    action      VARCHAR(64) NOT NULL,
    -- action values: 'push', 'create_repo', 'delete_repo', 'open_issue', 'close_issue',
    --                'open_pr', 'merge_pr', 'create_wiki', 'edit_wiki', 'fork_repo',
    --                'star_repo', 'comment', 'join_org', 'leave_org'
    resource_type VARCHAR(32) NOT NULL,
    -- resource_type values: 'repo', 'issue', 'pr', 'wiki', 'org', 'comment', 'pipeline'
    resource_id   UUID,
    repo_id       UUID REFERENCES repositories(id) ON DELETE SET NULL,
    org_id        UUID REFERENCES organizations(id) ON DELETE SET NULL,
    description   TEXT NOT NULL DEFAULT '',
    metadata      JSONB NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_activity_events_actor ON activity_events(actor_id);
CREATE INDEX IF NOT EXISTS idx_activity_events_repo ON activity_events(repo_id);
CREATE INDEX IF NOT EXISTS idx_activity_events_org ON activity_events(org_id);
CREATE INDEX IF NOT EXISTS idx_activity_events_created ON activity_events(created_at DESC);

-- Federation actors (remote instances)
CREATE TABLE IF NOT EXISTS federation_actors (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_uri       VARCHAR(512) NOT NULL UNIQUE,
    inbox_url       VARCHAR(512) NOT NULL,
    outbox_url      VARCHAR(512) NOT NULL,
    public_key_id   VARCHAR(512),
    public_key_pem  TEXT,
    domain          VARCHAR(256) NOT NULL,
    actor_type      VARCHAR(32) NOT NULL DEFAULT 'Application',
    username        VARCHAR(128),
    display_name    VARCHAR(256),
    first_seen_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_fetched_at TIMESTAMPTZ,
    is_blocked      BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS idx_federation_actors_domain ON federation_actors(domain);
CREATE INDEX IF NOT EXISTS idx_federation_actors_uri ON federation_actors(actor_uri);

-- Federation activities (inbound + outbound)
CREATE TABLE IF NOT EXISTS federation_activities (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    activity_type   VARCHAR(64) NOT NULL,
    -- ActivityPub types: Create, Update, Delete, Follow, Undo, Accept, Reject, Add, Like, Announce
    actor_uri       VARCHAR(512) NOT NULL,
    object_type     VARCHAR(64),
    object_uri      VARCHAR(512),
    target_uri      VARCHAR(512),
    direction       VARCHAR(8) NOT NULL DEFAULT 'inbound',
    -- direction: 'inbound' or 'outbound'
    raw_json        JSONB NOT NULL DEFAULT '{}',
    status          VARCHAR(16) NOT NULL DEFAULT 'pending',
    -- status: pending, processing, delivered, failed, accepted, rejected
    local_resource_id UUID,
    processing_result TEXT,
    retry_count     INTEGER NOT NULL DEFAULT 0,
    next_retry_at   TIMESTAMPTZ,
    idempotency_key VARCHAR(128) UNIQUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at    TIMSTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_fed_activities_type ON federation_activities(activity_type);
CREATE INDEX IF NOT EXISTS idx_fed_activities_status ON federation_activities(status);
CREATE INDEX IF NOT EXISTS idx_fed_activities_actor ON federation_activities(actor_uri);
CREATE INDEX IF NOT EXISTS idx_fed_activities_direction ON federation_activities(direction);
CREATE INDEX IF NOT EXISTS idx_fed_activities_created ON federation_activities(created_at DESC);
