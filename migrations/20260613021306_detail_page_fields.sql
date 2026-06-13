-- project detail page schema additions.

-- Authored, curated fields on the project itself.
ALTER TABLE projects
    -- One curated primary label ("CLI Tool", "Web App", "Library"...). Authored,
    -- not derived from tags. Replaces the old derived "Language" field.
    ADD COLUMN project_type  TEXT,
    -- A project can be `active` AND closed-source, so this is orthogonal to
    -- `status`. When true: suppress repo link, show the closed-source callout.
    ADD COLUMN source_closed BOOLEAN NOT NULL DEFAULT false,
    -- Optional asciinema-style cast for the CLI hero. Shape:
    -- { prompt: string, lines: { t: "cmd"|"out"|"err"|"muted", text: string }[] }
    ADD COLUMN terminal_cast JSONB,
    -- Explicit per-project accent (hex). Frontend falls back to #71717a.
    ADD COLUMN accent_color  TEXT;

-- Curated "related work" — authored and ordered, explicitly NOT tag/language
-- similarity (e.g. Maestro -> Glint are both Minecraft despite different stacks).
-- Directed: a relation on A -> B does not imply B -> A.
CREATE TABLE project_relations (
    project_id         UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    related_project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    position           INT  NOT NULL,
    PRIMARY KEY (project_id, related_project_id),
    CHECK (project_id <> related_project_id)
);

-- Fetch a project's related list in authored order.
CREATE INDEX idx_project_relations_order ON project_relations (project_id, position);
