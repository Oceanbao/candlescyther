-- Add migration script here
CREATE TABLE logs (
    id INTEGER PRIMARY KEY,
    log_timestamp TEXT NOT NULL,
    log_level INTEGER NOT NULL,
    log_target TEXT NOT NULL,
    log_message TEXT NOT NULL,
    log_line INTEGER NOT NULL
);

-- Optional: Add an index for faster time-based lookups, critical for logs
-- CREATE INDEX idx_log_time ON log_entries (timestamp_ms);

-- Optional: Add an index for filtering by log level
-- CREATE INDEX idx_log_level ON log_entries (level);
CREATE INDEX idx_logs_timestamp ON logs (log_timestamp);
