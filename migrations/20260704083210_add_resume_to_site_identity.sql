ALTER TABLE site_identity
    ADD COLUMN resume_r2_key TEXT,
    ADD COLUMN resume_uploaded_at TIMESTAMPTZ;
