CREATE TABLE IF NOT EXISTS code_intelligence_symbols (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    file_path TEXT NOT NULL,
    line_number INTEGER NOT NULL,
    column_number INTEGER NOT NULL DEFAULT 0,
    signature TEXT,
    documentation TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS code_intelligence_references (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol_id UUID NOT NULL REFERENCES code_intelligence_symbols(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    line_number INTEGER NOT NULL,
    column_number INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_code_intelligence_symbols_repo_id ON code_intelligence_symbols(repo_id);
CREATE INDEX idx_code_intelligence_symbols_name ON code_intelligence_symbols(name);
CREATE INDEX idx_code_intelligence_references_symbol_id ON code_intelligence_references(symbol_id);