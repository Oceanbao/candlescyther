-- Add migration script here
CREATE TABLE user (
    id INTEGER PRIMARY KEY,
    user_name VARCHAR(255) UNIQUE NOT NULL,
    user_role INTEGER NOT NULL
)
