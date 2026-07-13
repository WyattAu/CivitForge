CREATE TABLE IF NOT EXISTS license_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    license TEXT NOT NULL,
    spdx_id TEXT NOT NULL,
    file_count INTEGER NOT NULL DEFAULT 0,
    compliant BOOLEAN NOT NULL DEFAULT true,
    issues JSONB NOT NULL DEFAULT '[]',
    scanned_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_license_reports_repo_id ON license_reports(repo_id);
CREATE INDEX IF NOT EXISTS idx_license_reports_scanned_at ON license_reports(scanned_at DESC);
