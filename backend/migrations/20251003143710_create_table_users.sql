-- Add migration script here
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    user_name VARCHAR(255) UNIQUE NOT NULL,
    user_role VARCHAR(255) NOT NULL
);
