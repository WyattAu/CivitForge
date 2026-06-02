-- Add full-text search columns to code_search_index and wiki_pages
-- Uses PostgreSQL tsvector/tsquery for proper full-text search

ALTER TABLE code_search_index ADD COLUMN IF NOT EXISTS search_vector tsvector;

-- Create GIN index for fast full-text search
CREATE INDEX IF NOT EXISTS idx_code_search_vector ON code_search_index USING GIN(search_vector);

-- Trigger to auto-update search_vector on INSERT/UPDATE
CREATE OR REPLACE FUNCTION code_search_vector_update() RETURNS trigger AS $$
BEGIN
    NEW.search_vector :=
        setweight(to_tsvector('english', COALESCE(NEW.file_path, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(NEW.language, '')), 'B');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_code_search_vector_update ON code_search_index;
CREATE TRIGGER trg_code_search_vector_update
    BEFORE INSERT OR UPDATE ON code_search_index
    FOR EACH ROW EXECUTE FUNCTION code_search_vector_update();

-- Wiki pages: add full-text search column
ALTER TABLE wiki_pages ADD COLUMN IF NOT EXISTS search_vector tsvector;

CREATE INDEX IF NOT EXISTS idx_wiki_pages_search_vector ON wiki_pages USING GIN(search_vector);

CREATE OR REPLACE FUNCTION wiki_search_vector_update() RETURNS trigger AS $$
BEGIN
    NEW.search_vector :=
        setweight(to_tsvector('english', COALESCE(NEW.title, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(NEW.slug, '')), 'B') ||
        setweight(to_tsvector('english', COALESCE(NEW.content, '')), 'D');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_wiki_search_vector_update ON wiki_pages;
CREATE TRIGGER trg_wiki_search_vector_update
    BEFORE INSERT OR UPDATE ON wiki_pages
    FOR EACH ROW EXECUTE FUNCTION wiki_search_vector_update();
