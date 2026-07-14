CREATE TABLE IF NOT EXISTS intrusion_detections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    detection_type TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'medium',
    source_ip INET NOT NULL,
    target TEXT NOT NULL,
    message TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS intrusion_detection_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    detection_type TEXT NOT NULL,
    pattern TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'medium',
    action TEXT NOT NULL DEFAULT 'alert',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS intrusion_incidents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    detection_id UUID NOT NULL REFERENCES intrusion_detections(id) ON DELETE CASCADE,
    response_action TEXT NOT NULL,
    response_data JSONB NOT NULL DEFAULT '{}',
    resolved BOOLEAN NOT NULL DEFAULT false,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_intrusion_detections_status ON intrusion_detections(status);
CREATE INDEX IF NOT EXISTS idx_intrusion_detections_severity ON intrusion_detections(severity);
CREATE INDEX IF NOT EXISTS idx_intrusion_detections_source_ip ON intrusion_detections(source_ip);
CREATE INDEX IF NOT EXISTS idx_intrusion_detection_rules_detection_type ON intrusion_detection_rules(detection_type);
CREATE INDEX IF NOT EXISTS idx_intrusion_incidents_detection_id ON intrusion_incidents(detection_id);
