-- Add migration script here
CREATE TABLE jobs (
    id INTEGER PRIMARY KEY,
    job_status TEXT NOT NULL DEFAULT 'pending',
    job_type TEXT NOT NULL,
    payload TEXT NOT NULL, -- This will store JSON as text
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    error_message TEXT
);

-- Essential indexes for common query patterns
CREATE INDEX idx_jobs_created_at ON jobs (created_at);
