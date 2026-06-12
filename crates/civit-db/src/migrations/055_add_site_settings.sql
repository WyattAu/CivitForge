-- Migration 055: Site settings
CREATE TABLE IF NOT EXISTS site_settings (
    id INTEGER PRIMARY KEY DEFAULT 1,
    site_name VARCHAR(255) NOT NULL DEFAULT 'CivitForge',
    site_description TEXT NOT NULL DEFAULT '',
    footer_text TEXT NOT NULL DEFAULT '',
    logo_url TEXT NOT NULL DEFAULT '',
    contact_email VARCHAR(255) NOT NULL DEFAULT '',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT site_settings_single_row CHECK (id = 1)
);

INSERT INTO site_settings (id, site_name, site_description, footer_text, logo_url, contact_email)
VALUES (1, 'CivitForge', '', '', '', '')
ON CONFLICT (id) DO NOTHING;
