-- Migration 049: Fix type mismatches between models and migrations
-- Add updated_at to pipelines table
ALTER TABLE pipelines ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
-- Fix issues.labels: JSONB -> TEXT[]
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'issues' AND column_name = 'labels' AND udt_name = 'jsonb') THEN
        ALTER TABLE issues ADD COLUMN labels_new TEXT[] NOT NULL DEFAULT '{}';
        UPDATE issues SET labels_new = ARRAY(SELECT jsonb_array_elements_text(labels));
        ALTER TABLE issues DROP COLUMN labels;
        ALTER TABLE issues RENAME COLUMN labels_new TO labels;
    END IF;
END $$;
