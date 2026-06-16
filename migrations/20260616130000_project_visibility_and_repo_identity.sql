-- Split public visibility out of the status enum, name the closed-source flag
-- "private", and anchor each repo to GitHub's stable numeric id.
--
-- Before: `status = 'hidden'` doubled as "don't show this project", conflating
-- visibility with the activity/development state the other variants describe.
-- After: `hidden` is its own boolean (overall public visibility) and `status`
-- only carries activity state (active | maintained | archived).

-- 1. Public-visibility flag, independent of activity state.
ALTER TABLE projects ADD COLUMN hidden BOOLEAN NOT NULL DEFAULT false;

-- 2. Carry existing hidden-status projects over: they become hidden, and their
--    activity state lands on `active` (the prior status was lost to 'hidden', so
--    the author re-triages from there).
UPDATE projects SET hidden = true, status = 'active' WHERE status = 'hidden';

-- 3. `source_closed` is exactly the "Private" concept (source code is private →
--    hide the repo link, keep syncing). Rename it to match the vocabulary.
ALTER TABLE projects RENAME COLUMN source_closed TO private;

-- 4. Stable repo identity. GitHub's numeric repo id survives renames AND
--    owner/transfer changes, so it anchors a project to its repo even when the
--    "owner/repo" string drifts. NULL until the first successful API resolve.
ALTER TABLE projects ADD COLUMN github_repo_id BIGINT;

-- 5. Drop the updated_at trigger before touching the status column: its WHEN
--    clause references `status`, which otherwise blocks the column-type rewrite
--    below ("cannot alter type of a column used in a trigger definition"). It is
--    recreated in step 7.
DROP TRIGGER IF EXISTS update_projects_updated_at ON projects;

-- 6. Drop 'hidden' from the enum. Postgres can't remove an enum value in place,
--    so recreate the type. Existing rows were migrated off 'hidden' in step 2,
--    making the text round-trip safe. The status index is rebuilt automatically
--    by the column rewrite.
ALTER TABLE projects ALTER COLUMN status DROP DEFAULT;
ALTER TYPE project_status RENAME TO project_status_old;
CREATE TYPE project_status AS ENUM ('active', 'maintained', 'archived');
ALTER TABLE projects
    ALTER COLUMN status TYPE project_status USING status::text::project_status;
ALTER TABLE projects ALTER COLUMN status SET DEFAULT 'active';
DROP TYPE project_status_old;

-- 7. Recreate the updated_at trigger:
--    * source_closed → private, plus the new `hidden` toggle (both editorial).
--    * DROP github_repo from the allowlist: it is now also written by background
--      sync (canonical-name healing on rename), so per the existing
--      "sync-written columns are exempt" convention it must not bump updated_at.
CREATE TRIGGER update_projects_updated_at
    BEFORE UPDATE ON projects
    FOR EACH ROW
    WHEN (
        OLD.slug IS DISTINCT FROM NEW.slug
        OR OLD.name IS DISTINCT FROM NEW.name
        OR OLD.short_description IS DISTINCT FROM NEW.short_description
        OR OLD.status IS DISTINCT FROM NEW.status
        OR OLD.hidden IS DISTINCT FROM NEW.hidden
        OR OLD.private IS DISTINCT FROM NEW.private
        OR OLD.demo_url IS DISTINCT FROM NEW.demo_url
        OR OLD.detail_content IS DISTINCT FROM NEW.detail_content
        OR OLD.project_type IS DISTINCT FROM NEW.project_type
        OR OLD.terminal_cast IS DISTINCT FROM NEW.terminal_cast
        OR OLD.accent_color IS DISTINCT FROM NEW.accent_color
    )
    EXECUTE FUNCTION update_updated_at_column();
