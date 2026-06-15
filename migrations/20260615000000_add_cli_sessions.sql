-- Distinguish interactive browser sessions from long-lived CLI tokens.
-- CLI tokens are bearer credentials: only a hash is stored so a database leak
-- never exposes a usable token. Browser sessions keep using the ULID id as the
-- cookie value (token_hash stays NULL for them).
ALTER TABLE sessions
    ADD COLUMN session_type TEXT NOT NULL DEFAULT 'browser'
        CHECK (session_type IN ('browser', 'cli')),
    ADD COLUMN token_hash TEXT,
    ADD COLUMN label TEXT;

-- CLI tokens are looked up by hash on every request; enforce uniqueness and
-- give the lookup an index. Browser rows (NULL token_hash) are excluded.
CREATE UNIQUE INDEX idx_sessions_token_hash
    ON sessions (token_hash)
    WHERE token_hash IS NOT NULL;
