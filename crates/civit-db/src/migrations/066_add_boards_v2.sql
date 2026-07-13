ALTER TABLE board_cards ADD COLUMN IF NOT EXISTS priority INTEGER NOT NULL DEFAULT 0;
ALTER TABLE board_cards ADD COLUMN IF NOT EXISTS due_date TIMESTAMPTZ;
ALTER TABLE board_cards ADD COLUMN IF NOT EXISTS sort_order INTEGER NOT NULL DEFAULT 0;

CREATE TABLE IF NOT EXISTS board_card_labels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    card_id UUID NOT NULL REFERENCES board_cards(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    color TEXT NOT NULL DEFAULT '#3b82f6',
    UNIQUE(card_id, label)
);

CREATE TABLE IF NOT EXISTS board_card_assignees (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    card_id UUID NOT NULL REFERENCES board_cards(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(card_id, user_id)
);
