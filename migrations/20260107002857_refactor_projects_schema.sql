-- Drop priority column and its index
DROP INDEX IF EXISTS idx_projects_priority;
ALTER TABLE projects DROP COLUMN priority;

-- Rename title to name
ALTER TABLE projects RENAME COLUMN title TO name;

-- Add short_description field
ALTER TABLE projects ADD COLUMN short_description TEXT NOT NULL DEFAULT '';
