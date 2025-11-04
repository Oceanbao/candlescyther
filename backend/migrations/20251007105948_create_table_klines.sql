-- Add migration script here
CREATE TABLE klines (
    k_ticker TEXT NOT NULL,
    k_date INTEGER NOT NULL,
    k_open REAL NOT NULL,
    k_high REAL NOT NULL,
    k_low REAL NOT NULL,
    k_close REAL NOT NULL,
    k_volume REAL NOT NULL,
    k_value REAL NOT NULL,
    PRIMARY KEY (k_ticker, k_date)
);

-- Essential indexes for common query patterns
CREATE INDEX idx_klines_ticker_date ON klines (k_ticker, k_date);
CREATE INDEX idx_klines_date ON klines (k_date);
CREATE INDEX idx_klines_ticker ON klines (k_ticker);
