-- Tags table
CREATE TABLE tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for tags
CREATE INDEX idx_tags_slug ON tags(slug);

-- Case-insensitive unique constraint on name
CREATE UNIQUE INDEX idx_tags_name_lower ON tags(LOWER(name));

-- Project-Tag junction table
CREATE TABLE project_tags (
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (project_id, tag_id)
);

-- Indexes for project_tags
CREATE INDEX idx_project_tags_project_id ON project_tags(project_id);
CREATE INDEX idx_project_tags_tag_id ON project_tags(tag_id);

-- Tag cooccurrence matrix
CREATE TABLE tag_cooccurrence (
    tag_a UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    tag_b UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (tag_a, tag_b),
    CHECK (tag_a < tag_b)
);

-- Index for reverse lookups
CREATE INDEX idx_tag_cooccurrence_tag_b ON tag_cooccurrence(tag_b, tag_a);
