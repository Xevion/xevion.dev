-- The single settings row must exist before any page renders; the Postgres import overwrites it.
INSERT OR IGNORE INTO `site_identity` (`id`, `display_name`, `occupation`, `bio`, `site_title`, `updated_at`)
VALUES (
  1,
  'Ryan Walters',
  'Full-Stack Software Engineer',
  'A fanatical software engineer with expertise and passion for sound, scalable and high-performance applications. I''m always working on something new.
Sometimes innovative — sometimes crazy.',
  'Xevion.dev',
  unixepoch()
);
