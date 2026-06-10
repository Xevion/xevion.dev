-- Rich detail-page content authored in the admin TipTap editor, stored as
-- ProseMirror JSON. Nullable: only projects with a non-null value get a
-- /projects/[slug] detail page; the rest link straight to demo/GitHub.
ALTER TABLE projects ADD COLUMN detail_content JSONB;
