CREATE TABLE IF NOT EXISTS apm_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_id TEXT NOT NULL,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    result TEXT NOT NULL DEFAULT 'success',
    context JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS apm_spans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_id TEXT NOT NULL,
    parent_id TEXT,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    duration_ms INTEGER NOT NULL,
    context JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_apm_transactions_transaction_id ON apm_transactions(transaction_id);
CREATE INDEX IF NOT EXISTS idx_apm_transactions_name ON apm_transactions(name);
CREATE INDEX IF NOT EXISTS idx_apm_transactions_type ON apm_transactions(type);
CREATE INDEX IF NOT EXISTS idx_apm_transactions_created_at ON apm_transactions(created_at);
CREATE INDEX IF NOT EXISTS idx_apm_spans_transaction_id ON apm_spans(transaction_id);
CREATE INDEX IF NOT EXISTS idx_apm_spans_type ON apm_spans(type);
CREATE INDEX IF NOT EXISTS idx_apm_spans_start_time ON apm_spans(start_time);
