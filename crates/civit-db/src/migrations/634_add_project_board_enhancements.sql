CREATE TABLE IF NOT EXISTS project_boards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    owner_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS project_board_columns_v1 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id UUID NOT NULL REFERENCES project_boards(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    position INTEGER NOT NULL DEFAULT 0,
    color TEXT NOT NULL DEFAULT '#808080',
    wip_limit INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_project_board_columns_board ON project_board_columns_v1(board_id);

CREATE TABLE IF NOT EXISTS project_board_cards_v1 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id UUID NOT NULL REFERENCES project_boards(id) ON DELETE CASCADE,
    column_id UUID NOT NULL REFERENCES project_board_columns_v1(id),
    issue_id UUID REFERENCES issues(id),
    position INTEGER NOT NULL DEFAULT 0,
    assignee_id UUID REFERENCES users(id),
    labels JSONB NOT NULL DEFAULT '[]',
    due_date TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_project_board_cards_board ON project_board_cards_v1(board_id);
CREATE INDEX IF NOT EXISTS idx_project_board_cards_column ON project_board_cards_v1(column_id);
CREATE INDEX IF NOT EXISTS idx_project_board_cards_issue ON project_board_cards_v1(issue_id);
CREATE INDEX IF NOT EXISTS idx_project_board_cards_assignee ON project_board_cards_v1(assignee_id);

CREATE TABLE IF NOT EXISTS project_board_card_movements_v1 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    card_id UUID NOT NULL REFERENCES project_board_cards_v1(id) ON DELETE CASCADE,
    from_column_id UUID REFERENCES project_board_columns_v1(id),
    to_column_id UUID NOT NULL REFERENCES project_board_columns_v1(id),
    moved_by UUID NOT NULL REFERENCES users(id),
    moved_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_project_board_card_movements_card ON project_board_card_movements_v1(card_id);
