-- Collapse the project text fields to just `short_description` (the card
-- tagline) plus the rich `detail_content` page. The middle plain-text
-- `description` column is removed; any project that had a `description` but no
-- detail page yet has it lifted into `detail_content` as a ProseMirror doc so
-- nothing is lost. Blank-line-separated chunks become paragraphs.
UPDATE projects
SET detail_content = jsonb_build_object(
    'type', 'doc',
    'content', coalesce(
        (
            SELECT jsonb_agg(
                jsonb_build_object(
                    'type', 'paragraph',
                    'content', jsonb_build_array(
                        jsonb_build_object('type', 'text', 'text', para)
                    )
                )
            )
            FROM (
                SELECT trim(chunk) AS para
                FROM regexp_split_to_table(projects.description, E'\n\\s*\n') AS chunk
            ) chunks
            WHERE para <> ''
        ),
        '[]'::jsonb
    )
)
WHERE detail_content IS NULL
  AND coalesce(trim(description), '') <> '';

-- The updated_at trigger's WHEN clause references `description`, which both
-- creates a dependency that blocks the drop and would dangle afterwards.
-- Recreate it without that column before dropping it.
DROP TRIGGER IF EXISTS update_projects_updated_at ON projects;

CREATE TRIGGER update_projects_updated_at
    BEFORE UPDATE ON projects
    FOR EACH ROW
    WHEN (
        OLD.slug IS DISTINCT FROM NEW.slug
        OR OLD.name IS DISTINCT FROM NEW.name
        OR OLD.short_description IS DISTINCT FROM NEW.short_description
        OR OLD.status IS DISTINCT FROM NEW.status
        OR OLD.github_repo IS DISTINCT FROM NEW.github_repo
        OR OLD.demo_url IS DISTINCT FROM NEW.demo_url
        OR OLD.detail_content IS DISTINCT FROM NEW.detail_content
        OR OLD.project_type IS DISTINCT FROM NEW.project_type
        OR OLD.source_closed IS DISTINCT FROM NEW.source_closed
        OR OLD.terminal_cast IS DISTINCT FROM NEW.terminal_cast
        OR OLD.accent_color IS DISTINCT FROM NEW.accent_color
    )
    EXECUTE FUNCTION update_updated_at_column();

ALTER TABLE projects DROP COLUMN description;
