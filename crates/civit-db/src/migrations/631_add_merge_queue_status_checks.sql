CREATE TABLE IF NOT EXISTS merge_queue_entries_v1 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    pull_request_id UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'waiting',
    merge_sha TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repository_id, pull_request_id)
);

CREATE TABLE IF NOT EXISTS merge_queue_checks_v1 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    queue_entry_id UUID NOT NULL REFERENCES merge_queue_entries_v1(id) ON DELETE CASCADE,
    check_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    output JSONB NOT NULL DEFAULT '{}',
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(queue_entry_id, check_name)
);
