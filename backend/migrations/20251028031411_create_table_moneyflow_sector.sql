-- Add migration script here
CREATE TABLE moneyflow_sector (
    date_time TEXT NOT NULL,
    ticker TEXT NOT NULL,
    realname TEXT NOT NULL,
    lead_value REAL NOT NULL,
    lead_share REAL NOT NULL,
    super_value REAL NOT NULL,
    super_share REAL NOT NULL,
    large_value REAL NOT NULL,
    large_share REAL NOT NULL,
    mid_value REAL NOT NULL,
    mid_share REAL NOT NULL,
    small_value REAL NOT NULL,
    small_share REAL NOT NULL
);

CREATE INDEX idx_moneyflow_sector_datetime_ticker
ON moneyflow_sector (date_time, ticker);

-- Working trigger
CREATE TRIGGER maintain_30_records_per_ticker
AFTER INSERT ON moneyflow_sector
BEGIN
    DELETE FROM moneyflow_sector
    WHERE ticker = NEW.ticker
    AND date_time = (
        SELECT date_time
        FROM moneyflow_sector
        WHERE ticker = NEW.ticker
        ORDER BY date_time
        LIMIT 1 OFFSET 30
    );
END;
