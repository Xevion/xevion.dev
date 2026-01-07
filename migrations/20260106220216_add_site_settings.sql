-- Site identity settings (single row table)
CREATE TABLE site_identity (
    id INTEGER PRIMARY KEY CHECK (id = 1), -- Enforce single row
    display_name TEXT NOT NULL,
    occupation TEXT NOT NULL,
    bio TEXT NOT NULL,
    site_title TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Trigger for updated_at
CREATE TRIGGER update_site_identity_updated_at 
    BEFORE UPDATE ON site_identity
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Social links (multiple rows, extensible)
CREATE TABLE social_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    platform TEXT NOT NULL, -- Not an enum for extensibility
    label TEXT NOT NULL,
    value TEXT NOT NULL,
    icon TEXT NOT NULL, -- Icon identifier (e.g., 'simple-icons:github')
    visible BOOLEAN NOT NULL DEFAULT true,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for ordering
CREATE INDEX idx_social_links_order ON social_links(display_order ASC);

-- Trigger for updated_at
CREATE TRIGGER update_social_links_updated_at 
    BEFORE UPDATE ON social_links
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Seed default identity
INSERT INTO site_identity (id, display_name, occupation, bio, site_title)
VALUES (
    1,
    'Ryan Walters',
    'Full-Stack Software Engineer',
    'A fanatical software engineer with expertise and passion for sound, scalable and high-performance applications. I''m always working on something new.
Sometimes innovative — sometimes crazy.',
    'Xevion.dev'
);

-- Seed default social links
INSERT INTO social_links (platform, label, value, icon, visible, display_order) VALUES
    ('github', 'GitHub', 'https://github.com/Xevion', 'simple-icons:github', true, 1),
    ('linkedin', 'LinkedIn', 'https://linkedin.com/in/ryancwalters', 'simple-icons:linkedin', true, 2),
    ('discord', 'Discord', 'xevion', 'simple-icons:discord', true, 3),
    ('email', 'Email', 'your.email@example.com', 'material-symbols:mail-rounded', true, 4);
