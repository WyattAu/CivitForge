-- ADR-0008 track 2: flag lifecycle governance.
--
-- Feature-flag governance in production systems (Harness FME whitepaper:
-- owner, ticket linkage, default-off in production, per-environment RBAC)
-- plus the staleness signals Datadog/Flagsmith need. The staleness
-- classifier is only as good as its inputs, so the evidence is stored with
-- the flag rather than recomputed from logs:
--
--   kind                lifecycle category; sets the staleness deadline and
--                       whether the flag is exempt (permission gates never
--                       go stale).
--   owner               accountable party; enforced at creation so flags
--                       cannot be created anonymously.
--   ticket              external issue reference for the change.
--   salt                rollout-cycle identifier. Sticky within a cycle,
--                       re-drawable across cycles without a deploy
--                       (LaunchDarkly two-level randomization).
--   last_evaluated_at   last time evaluation read this flag. A flag nobody
--                       evaluates is a removal candidate, but only if we
--                       can prove nobody evaluated it -- that requires
--                       recording it, hence this column.
--   last_changed_at     distinct from updated_at in spirit: the staleness
--                       clock runs from the last *change*, so a flag still
--                       being edited has not finished its life.

ALTER TABLE feature_flags
    ADD COLUMN IF NOT EXISTS kind TEXT NOT NULL DEFAULT 'release',
    ADD COLUMN IF NOT EXISTS owner TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS ticket TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS salt TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS last_evaluated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS last_changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- Governance wants an owner on every flag. Existing rows predate the
-- requirement, so backfill with a sentinel that the API rejects on update
-- rather than inventing a human owner.
UPDATE feature_flags SET owner = 'unassigned' WHERE owner = '';

-- The staleness endpoint scans by verdict, and the audit view needs age.
CREATE INDEX IF NOT EXISTS idx_feature_flags_kind ON feature_flags (kind);
CREATE INDEX IF NOT EXISTS idx_feature_flags_last_changed ON feature_flags (last_changed_at);
CREATE INDEX IF NOT EXISTS idx_feature_flags_last_evaluated ON feature_flags (last_evaluated_at);

-- Kind must be one of the four lifecycle categories; the kit is the source
-- of truth for the wire names, and this constraint keeps a hand-written
-- INSERT from inventing a fifth. Validated is safe here because the column
-- is new with a default, so every existing row was backfilled to 'release'.
ALTER TABLE feature_flags
    ADD CONSTRAINT feature_flags_kind_check
    CHECK (kind IN ('release', 'experiment', 'operational', 'permission'));

-- Flag names are the evaluation key and appear in code, URLs, and audit
-- logs. Constrain them the way flag-kit does so a bad name cannot reach
-- production through a raw INSERT.
--
-- NOT VALID on purpose: adding this constraint *validated* aborts the whole
-- migration on any pre-existing non-conforming name, which is how the
-- deployment fails against real data (hyphenated legacy names verified this
-- against a live Postgres). NOT VALID grandfathers existing rows while still
-- enforcing the rule on every INSERT and UPDATE from here on.
ALTER TABLE feature_flags
    ADD CONSTRAINT feature_flags_name_format_check
    CHECK (name ~ '^[a-z][a-z0-9_]{0,62}$') NOT VALID;

-- Operators get the grandfathered rows as work to do, rather than a silent
-- exemption they never learn about. Fix with:
--   UPDATE feature_flags SET name = replace(name, '-', '_') WHERE name ~ '-';
--   ALTER TABLE feature_flags VALIDATE CONSTRAINT feature_flags_name_format_check;