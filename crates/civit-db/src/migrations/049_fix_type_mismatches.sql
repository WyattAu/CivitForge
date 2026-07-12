-- Migration 049: Fix type mismatches between models and migrations
-- Add updated_at to pipelines table
ALTER TABLE pipelines ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
-- Fix issues.labels: JSONB -> TEXT[] (handled by application logic if needed)
-- Note: This migration previously had PL/pgSQL that was incompatible with the migration runner.
-- The labels column type conversion is handled at the application level.
