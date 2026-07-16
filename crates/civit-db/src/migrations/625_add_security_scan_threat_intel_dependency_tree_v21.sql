-- Security Scanning v24: threat intelligence v21, dependency tree analysis v21
CREATE TABLE IF NOT EXISTS security_scan_threat_intelligence_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cve_id TEXT NOT NULL,
    severity TEXT NOT NULL,
    description TEXT NOT NULL,
    affected_packages TEXT[] NOT NULL DEFAULT '{}',
    fix_available BOOLEAN NOT NULL DEFAULT false,
    published_at TIMESTAMPTZ,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_threat_intelligence_v21_cve ON security_scan_threat_intelligence_v21(cve_id);
CREATE INDEX IF NOT EXISTS idx_threat_intelligence_v21_severity ON security_scan_threat_intelligence_v21(severity);
CREATE INDEX IF NOT EXISTS idx_threat_intelligence_v21_fetched ON security_scan_threat_intelligence_v21(fetched_at DESC);
CREATE INDEX IF NOT EXISTS idx_threat_intelligence_v21_packages ON security_scan_threat_intelligence_v21 USING GIN(affected_packages);

CREATE TABLE IF NOT EXISTS security_scan_dependency_tree_v21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id),
    package_name TEXT NOT NULL,
    version TEXT NOT NULL,
    parent_package TEXT,
    dependency_type TEXT NOT NULL DEFAULT 'direct',
    depth INTEGER NOT NULL DEFAULT 0,
    scanned_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_dependency_tree_v21_repo ON security_scan_dependency_tree_v21(repo_id);
CREATE INDEX IF NOT EXISTS idx_dependency_tree_v21_package ON security_scan_dependency_tree_v21(package_name);
CREATE INDEX IF NOT EXISTS idx_dependency_tree_v21_parent ON security_scan_dependency_tree_v21(parent_package);
CREATE INDEX IF NOT EXISTS idx_dependency_tree_v21_repo_package ON security_scan_dependency_tree_v21(repo_id, package_name);
