CREATE TABLE IF NOT EXISTS service_mesh_services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    endpoint TEXT NOT NULL,
    protocol TEXT NOT NULL DEFAULT 'http',
    health_check_url TEXT,
    status TEXT NOT NULL DEFAULT 'healthy',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS service_mesh_routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    path TEXT NOT NULL,
    service_id UUID NOT NULL REFERENCES service_mesh_services(id),
    weight INTEGER NOT NULL DEFAULT 100,
    headers JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
