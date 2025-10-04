-- Add migration script here
CREATE TABLE logging (
    id INTEGER PRIMARY KEY,
    log_timestamp TEXT NOT NULL,
    log_level TEXT NOT NULL,
    log_target TEXT NOT NULL
)
