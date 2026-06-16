-- Track GitHub sync health separately from activity.
--
-- `last_github_activity` records WHEN the repo last had activity; it says nothing
-- about whether syncing still works. These two columns close that gap:
--   github_synced_at  — timestamp of the last successful poll (NULL until first)
--   github_sync_error — most recent sync failure, cleared on success (NULL = healthy)
-- so the admin UI can distinguish "quiet repo" from "sync has been broken".
--
-- No trigger change is needed: update_projects_updated_at fires only WHEN one of an
-- explicit allowlist of editorial columns changes. These new columns are absent
-- from that list, so background sync writes never bump updated_at (same reason
-- last_github_activity is already exempt).
ALTER TABLE projects
    ADD COLUMN github_synced_at TIMESTAMPTZ,
    ADD COLUMN github_sync_error TEXT;
