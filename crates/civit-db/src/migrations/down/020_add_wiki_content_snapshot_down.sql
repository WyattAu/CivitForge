-- Remove content snapshot from wiki revisions
ALTER TABLE wiki_revisions DROP COLUMN IF EXISTS content_snapshot;
