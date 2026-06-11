ALTER TABLE wiki_pages DROP COLUMN IF EXISTS git_synced;
ALTER TABLE wiki_revisions DROP COLUMN IF EXISTS is_git_commit;
