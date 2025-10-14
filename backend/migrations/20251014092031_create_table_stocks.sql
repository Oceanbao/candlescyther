-- Add migration script here
CREATE TABLE stocks (
    ticker TEXT NOT NULL PRIMARY KEY
);

-- Essential indexes for common query patterns
CREATE INDEX idx_stocks_ticker ON stocks (ticker);
