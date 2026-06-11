-- Add content snapshot to wiki revisions for diff support
ALTER TABLE wiki_revisions ADD COLUMN IF NOT EXISTS content_snapshot TEXT NOT NULL DEFAULT '';
