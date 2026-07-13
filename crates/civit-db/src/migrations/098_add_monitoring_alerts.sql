CREATE TABLE IF NOT EXISTS monitoring_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    alert_type TEXT NOT NULL,
    condition TEXT NOT NULL,
    threshold DOUBLE PRECISION NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_monitoring_alerts_repo_id ON monitoring_alerts(repo_id);
CREATE INDEX IF NOT EXISTS idx_monitoring_alerts_alert_type ON monitoring_alerts(alert_type);

CREATE TABLE IF NOT EXISTS monitoring_incidents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_id UUID NOT NULL REFERENCES monitoring_alerts(id) ON DELETE CASCADE,
    severity TEXT NOT NULL,
    message TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open',
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_monitoring_incidents_alert_id ON monitoring_incidents(alert_id);
CREATE INDEX IF NOT EXISTS idx_monitoring_incidents_status ON monitoring_incidents(status);
CREATE INDEX IF NOT EXISTS idx_monitoring_incidents_severity ON monitoring_incidents(severity);
