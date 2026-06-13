-- The initial schema's trigger bumped updated_at on EVERY update, so background
-- GitHub syncs (last_github_activity) polluted it. Re-guard it to fire only on
-- editorial-column changes, making updated_at a true "last edited" signal.

DROP TRIGGER IF EXISTS update_projects_updated_at ON projects;

CREATE TRIGGER update_projects_updated_at
    BEFORE UPDATE ON projects
    FOR EACH ROW
    WHEN (
        OLD.slug IS DISTINCT FROM NEW.slug
        OR OLD.name IS DISTINCT FROM NEW.name
        OR OLD.short_description IS DISTINCT FROM NEW.short_description
        OR OLD.description IS DISTINCT FROM NEW.description
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
