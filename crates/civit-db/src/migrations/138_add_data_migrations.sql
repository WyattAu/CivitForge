-- CivitForge Phase 138: Data Migration
-- Migration 138
-- Adds tables for tracking data migrations with progress and rollback support.

CREATE TABLE IF NOT EXISTS data_migrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source TEXT NOT NULL,
    destination TEXT NOT NULL,
    migration_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    progress DOUBLE PRECISION NOT NULL DEFAULT 0,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_data_migrations_status ON data_migrations(status);
CREATE INDEX IF NOT EXISTS idx_data_migrations_type ON data_migrations(migration_type);
CREATE INDEX IF NOT EXISTS idx_data_migrations_started ON data_migrations(started_at DESC);
