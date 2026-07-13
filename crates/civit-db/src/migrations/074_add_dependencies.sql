CREATE TABLE IF NOT EXISTS dependencies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    ecosystem TEXT NOT NULL,
    latest_version TEXT,
    is_outdated BOOLEAN NOT NULL DEFAULT false,
    has_vulnerabilities BOOLEAN NOT NULL DEFAULT false,
    vulnerability_count INTEGER NOT NULL DEFAULT 0,
    last_scanned_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, name, ecosystem)
);

CREATE TABLE IF NOT EXISTS vulnerability_advisories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dependency_id UUID NOT NULL REFERENCES dependencies(id) ON DELETE CASCADE,
    severity TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    url TEXT,
    patched_version TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_dependencies_repo_id ON dependencies(repo_id);
CREATE INDEX idx_dependencies_ecosystem ON dependencies(ecosystem);
CREATE INDEX idx_dependencies_outdated ON dependencies(is_outdated);
CREATE INDEX idx_dependencies_vulnerable ON dependencies(has_vulnerabilities);
CREATE INDEX idx_vulnerability_advisories_dependency_id ON vulnerability_advisories(dependency_id);
CREATE INDEX idx_vulnerability_advisories_severity ON vulnerability_advisories(severity);
