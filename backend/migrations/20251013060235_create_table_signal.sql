-- Add migration script here
CREATE TABLE signals (
    ticker TEXT NOT NULL,
    kdj_k REAL NOT NULL,
    kdj_d REAL NOT NULL,
    boll_dist REAL NOT NULL
);
