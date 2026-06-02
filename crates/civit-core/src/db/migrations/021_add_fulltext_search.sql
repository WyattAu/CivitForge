-- 021: Full-text search columns and GIN indexes for code_search_index and wiki_pages.

ALTER TABLE code_search_index ADD COLUMN IF NOT EXISTS search_vector tsvector;

CREATE INDEX IF NOT EXISTS idx_code_search_vector ON code_search_index USING GIN(search_vector);

ALTER TABLE wiki_pages ADD COLUMN IF NOT EXISTS search_vector tsvector;

CREATE INDEX IF NOT EXISTS idx_wiki_pages_search_vector ON wiki_pages USING GIN(search_vector);
