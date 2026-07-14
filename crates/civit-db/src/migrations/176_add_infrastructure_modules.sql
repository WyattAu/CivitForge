CREATE TABLE IF NOT EXISTS infrastructure_modules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    provider TEXT NOT NULL,
    module_type TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}',
    version TEXT NOT NULL DEFAULT '1.0.0',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS infrastructure_module_deps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    module_id UUID NOT NULL REFERENCES infrastructure_modules(id),
    dependency_id UUID NOT NULL REFERENCES infrastructure_modules(id),
    version_constraint TEXT NOT NULL DEFAULT '*',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(module_id, dependency_id)
);