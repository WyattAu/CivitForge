-- Remove full-text search columns and triggers
ALTER TABLE code_search_index DROP COLUMN IF EXISTS search_vector;
DROP TRIGGER IF EXISTS trg_code_search_vector_update ON code_search_index;
DROP FUNCTION IF EXISTS code_search_vector_update();

ALTER TABLE wiki_pages DROP COLUMN IF EXISTS search_vector;
DROP TRIGGER IF EXISTS trg_wiki_search_vector_update ON wiki_pages;
DROP FUNCTION IF EXISTS wiki_search_vector_update();
