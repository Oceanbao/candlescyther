-- Add migration script here
CREATE TABLE stocks (
    ticker TEXT NOT NULL PRIMARY KEY,
    realname TEXT NOT NULL,
    market INT NOT NULL,
    total_cap REAL,
    pe REAL,
    pb REAL,
    revenue REAL,
    net REAL,
    margin REAL,
    debt REAL
);

-- Essential indexes for common query patterns
CREATE INDEX idx_stocks_ticker ON stocks (ticker);
