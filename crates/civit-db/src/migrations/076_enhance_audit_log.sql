ALTER TABLE audit_events ADD COLUMN IF NOT EXISTS request_id UUID;
CREATE INDEX IF NOT EXISTS idx_audit_events_request_id ON audit_events(request_id);
