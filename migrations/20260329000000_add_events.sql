CREATE TYPE event_level AS ENUM ('info', 'warning', 'error');

CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL,
    level event_level NOT NULL DEFAULT 'info',
    entity_type TEXT,
    entity_id UUID,
    actor TEXT,
    message TEXT NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_events_created_at ON events (created_at DESC);
CREATE INDEX idx_events_filters ON events (level, entity_type, event_type);
