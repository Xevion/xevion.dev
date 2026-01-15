-- Project media table for carousel support
-- Each project can have multiple images/videos with ordering

CREATE TYPE media_type AS ENUM ('image', 'video');

CREATE TABLE project_media (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    display_order INT NOT NULL DEFAULT 0,
    media_type media_type NOT NULL,
    original_filename TEXT NOT NULL,
    r2_base_path TEXT NOT NULL,
    variants JSONB NOT NULL,
    width INT,
    height INT,
    size_bytes BIGINT NOT NULL,
    blurhash TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    UNIQUE (project_id, display_order)
);

CREATE INDEX idx_project_media_project_id ON project_media(project_id);
