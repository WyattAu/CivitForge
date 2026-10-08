-- ADR-0008: durable state for health-gated rollouts.
--
-- The gate in flag-kit holds the stage index and the consecutive-failure
-- streak. Both must survive a restart: losing the streak means a rollout
-- that was about to roll back gets a clean slate and promotes on the next
-- tick, and losing the stage index means an in-flight rollout silently
-- restarts from 5%.
--
-- The decision history is not optional either. An automated system that
-- changes a flag's exposure needs to answer "why did this jump from 25% to
-- 60%?" months later,  a verdict nobody can audit is a verdict nobody will
-- trust when it matters. feature_flag_events records human actions, so the
-- automated path gets its own table rather than overloading that one.

CREATE TABLE IF NOT EXISTS flag_rollouts (
    flag_id UUID PRIMARY KEY REFERENCES feature_flags(id) ON DELETE CASCADE,
    -- Index into the configured stage list-- 0 is the first stage.
    stage_index INTEGER NOT NULL DEFAULT 0 CHECK (stage_index >= 0),
    -- Consecutive failing observation windows. Reset to 0 on a healthy one.
    consecutive_failures INTEGER NOT NULL DEFAULT 0 CHECK (consecutive_failures >= 0),
    -- When the current stage was entered,  the gate's min_duration runs from
    -- here, not from process start.
    stage_started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_observed_at TIMESTAMPTZ,
    -- Last decision and its reason, so the current state is readable without
    -- scanning the event log.
    last_decision TEXT NOT NULL DEFAULT 'pending',
    last_reason TEXT NOT NULL DEFAULT '',
    last_error_rate DOUBLE PRECISION,
    last_latency_p99_ms DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS flag_rollout_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    flag_id UUID NOT NULL REFERENCES feature_flags(id) ON DELETE CASCADE,
    -- promote | hold | rollback | started | completed
    decision TEXT NOT NULL,
    -- Kit reason code (healthy, too_soon, too_few_samples,
    -- error_rate_too_high, latency_too_high, consecutive_failures).
    reason TEXT NOT NULL DEFAULT '',
    stage_index INTEGER NOT NULL,
    percentage_before INTEGER NOT NULL,
    percentage_after INTEGER NOT NULL,
    error_rate DOUBLE PRECISION,
    latency_p99_ms DOUBLE PRECISION,
    total_samples BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_flag_rollout_events_flag
    ON flag_rollout_events (flag_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_flag_rollouts_last_decision
    ON flag_rollouts (last_decision);

-- decision must be one of the gate's outcomes. Constrained so a typo in the
-- controller cannot write a value the UI cannot render.
ALTER TABLE flag_rollout_events
    ADD CONSTRAINT flag_rollout_events_decision_check
    CHECK (decision IN ('pending', 'promote', 'hold', 'rollback', 'started', 'completed'));

ALTER TABLE flag_rollouts
    ADD CONSTRAINT flag_rollouts_last_decision_check
    CHECK (last_decision IN ('pending', 'promote', 'hold', 'rollback', 'started', 'completed'));