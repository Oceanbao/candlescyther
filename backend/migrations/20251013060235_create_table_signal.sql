-- Add migration script here
CREATE TABLE signals (
    ticker TEXT NOT NULL,
    kdj_k REAL NOT NULL,
    kdj_d REAL NOT NULL
);

-- macd_dif REAL NOT NULL,
-- macd_dea REAL NOT NULL
