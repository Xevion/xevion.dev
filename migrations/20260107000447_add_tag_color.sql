-- Add color column to tags table (nullable hex color without hash)
ALTER TABLE tags ADD COLUMN color VARCHAR(6) DEFAULT NULL;

-- Add check constraint for valid hex format (6 characters, 0-9a-fA-F)
ALTER TABLE tags ADD CONSTRAINT tags_color_hex_format 
  CHECK (color IS NULL OR color ~ '^[0-9a-fA-F]{6}$');

-- Create index for color lookups (optional, for future filtering)
CREATE INDEX idx_tags_color ON tags(color) WHERE color IS NOT NULL;
