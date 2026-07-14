-- CivitForge Phase 136: Database Backup and Recovery
-- Migration 136
-- Adds tables for tracking database backups and recovery points.

CREATE TABLE IF NOT EXISTS database_backups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    backup_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    file_path TEXT,
    file_size_bytes BIGINT NOT NULL DEFAULT 0,
    checksum TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS database_recovery_points (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    backup_id UUID NOT NULL REFERENCES database_backups(id),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_database_backups_status ON database_backups(status);
CREATE INDEX IF NOT EXISTS idx_database_backups_type ON database_backups(backup_type);
CREATE INDEX IF NOT EXISTS idx_database_backups_started ON database_backups(started_at DESC);
CREATE INDEX IF NOT EXISTS idx_database_recovery_points_backup_id ON database_recovery_points(backup_id);
