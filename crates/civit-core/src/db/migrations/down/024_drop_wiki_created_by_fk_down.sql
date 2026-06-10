-- Migration 023 down: Re-add FK constraints on wiki created_by/author_id
-- Restores the original constraints from migration 015.

ALTER TABLE wiki_revisions ALTER COLUMN author_id SET NOT NULL;
ALTER TABLE wiki_revisions ADD CONSTRAINT wiki_revisions_author_id_fkey
    FOREIGN KEY (author_id) REFERENCES users(id);

ALTER TABLE wiki_pages ALTER COLUMN created_by SET NOT NULL;
ALTER TABLE wiki_pages ADD CONSTRAINT wiki_pages_created_by_fkey
    FOREIGN KEY (created_by) REFERENCES users(id);
