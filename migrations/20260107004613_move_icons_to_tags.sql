-- Add icon field to tags
ALTER TABLE tags ADD COLUMN icon TEXT;

-- Drop icon field from projects
ALTER TABLE projects DROP COLUMN icon;
