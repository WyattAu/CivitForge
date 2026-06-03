-- Migration 023: Drop FK constraints on wiki created_by/author_id
-- These columns may use nil UUID for system actions (no user context),
-- so a NOT NULL FK to users(id) is incorrect.

ALTER TABLE wiki_pages DROP CONSTRAINT IF EXISTS wiki_pages_created_by_fkey;
ALTER TABLE wiki_pages ALTER COLUMN created_by DROP NOT NULL;

ALTER TABLE wiki_revisions DROP CONSTRAINT IF EXISTS wiki_revisions_author_id_fkey;
ALTER TABLE wiki_revisions ALTER COLUMN author_id DROP NOT NULL;
