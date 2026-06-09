CREATE TABLE IF NOT EXISTS secret_scan_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scan_id VARCHAR(64) NOT NULL,
    repo_owner VARCHAR(255) NOT NULL,
    repo_name VARCHAR(255) NOT NULL,
    total_secrets INTEGER NOT NULL DEFAULT 0,
    scanned_files INTEGER NOT NULL DEFAULT 0,
    scan_json TEXT NOT NULL DEFAULT '[]',
    scanned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_secret_scan_results_repo ON secret_scan_results(repo_owner, repo_name);
CREATE INDEX IF NOT EXISTS idx_secret_scan_results_scan_id ON secret_scan_results(scan_id);

CREATE TABLE IF NOT EXISTS slsa_attestations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_owner VARCHAR(255) NOT NULL,
    repo_name VARCHAR(255) NOT NULL,
    pipeline_run_id VARCHAR(64) NOT NULL,
    attestation_id VARCHAR(64) NOT NULL,
    provenance_json TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_slsa_attestations_repo ON slsa_attestations(repo_owner, repo_name);
CREATE INDEX IF NOT EXISTS idx_slsa_attestations_run ON slsa_attestations(pipeline_run_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_slsa_attestations_unique ON slsa_attestations(repo_owner, repo_name, attestation_id);
