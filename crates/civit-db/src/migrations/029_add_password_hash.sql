-- Migration 029: Add password_hash column to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS password_hash TEXT NOT NULL DEFAULT '';

-- Create index for password lookups (already have unique on username/email)
