use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::domain::{
    model::{Kline, Signal, Stock},
    repository::DomainRepository,
};

#[derive(Clone)]
pub struct SqliteDomainRepository {
    pub pool: SqlitePool,
}

impl SqliteDomainRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DomainRepository for SqliteDomainRepository {
    async fn create_stock(&self, stock: Stock) -> Result<(), anyhow::Error> {
        sqlx::query!(
            "INSERT INTO stocks (ticker, realname, market, total_cap, pe, pb, revenue, net, margin, debt)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            stock.ticker,
            stock.realname,
            stock.market,
            stock.total_cap,
            stock.pe,
            stock.pb,
            stock.revenue,
            stock.net,
            stock.margin,
            stock.debt,
        )
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_stock(&self, ticker: &str) -> Result<Stock, anyhow::Error> {
        let stock = sqlx::query_as!(
            Stock,
            r#"
            SELECT *
            FROM stocks
            WHERE ticker = ?
            LIMIT 1
        "#,
            ticker
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(stock)
    }

    async fn get_stock_all(&self) -> Result<Vec<Stock>, anyhow::Error> {
        let stocks = sqlx::query_as!(
            Stock,
            r#"
            SELECT *
            FROM stocks
        "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(stocks)
    }

    async fn delete_stocks(&self, tickers: &[&str]) -> Result<(), anyhow::Error> {
        for ticker in tickers.iter() {
            let is_us = is_us(ticker);
            sqlx::query!("DELETE FROM stocks WHERE ticker = ?", ticker)
                .execute(&self.pool)
                .await?;
            if is_us {
                sqlx::query!("DELETE FROM klines_us WHERE k_ticker = ?", ticker)
                    .execute(&self.pool)
                    .await?;
                sqlx::query!("DELETE FROM signals_us WHERE ticker = ?", ticker)
                    .execute(&self.pool)
                    .await?;
            } else {
                sqlx::query!("DELETE FROM klines WHERE k_ticker = ?", ticker)
                    .execute(&self.pool)
                    .await?;
                sqlx::query!("DELETE FROM signals WHERE ticker = ?", ticker)
                    .execute(&self.pool)
                    .await?;
            }
        }

        Ok(())
    }

    // NOTE: Restricted to a single ticker.
    async fn create_klines(&self, ticker: &str, klines: &[Kline]) -> Result<(), anyhow::Error> {
        let is_us = is_us(ticker);

        // NOTE: Clear all records per tickers.
        if is_us {
            sqlx::query!("DELETE FROM klines_us WHERE k_ticker = ?", ticker)
                .execute(&self.pool)
                .await?;
        } else {
            sqlx::query!("DELETE FROM klines WHERE k_ticker = ?", ticker)
                .execute(&self.pool)
                .await?;
        }

        let batch_size = 5000;
        let chunks: Vec<Vec<Kline>> = klines
            .chunks(batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        for chunk in chunks {
            let tx = self.pool.begin().await?;

            // Batch commit.
            if is_us {
                for kline in chunk {
                    sqlx::query!(
                "INSERT INTO klines_us (k_ticker, k_date, k_open, k_high, k_low, k_close, k_volume, k_value) 
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                kline.k_ticker,
                kline.k_date,
                kline.k_open,
                kline.k_high,
                kline.k_low,
                kline.k_close,
                kline.k_volume,
                kline.k_value,
            )
            .execute(&self.pool)
            .await?;
                }
            } else {
                for kline in chunk {
                    sqlx::query!(
                        "INSERT INTO klines (k_ticker, k_date, k_open, k_high, k_low, k_close, k_volume, k_value) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                        kline.k_ticker,
                        kline.k_date,
                        kline.k_open,
                        kline.k_high,
                        kline.k_low,
                        kline.k_close,
                        kline.k_volume,
                        kline.k_value,
                    )
                        .execute(&self.pool)
                    .await?;
                }
            }

            tx.commit().await?;
        }

        Ok(())
    }

    async fn get_klines(&self, ticker: &str) -> Result<Vec<Kline>, anyhow::Error> {
        let is_us = is_us(ticker);

        if is_us {
            let klines = sqlx::query_as!(
                Kline,
                r#"
            SELECT *
            FROM klines_us
            WHERE k_ticker = ?
        "#,
                ticker
            )
            .fetch_all(&self.pool)
            .await?;

            Ok(klines)
        } else {
            let klines = sqlx::query_as!(
                Kline,
                r#"
            SELECT *
            FROM klines
            WHERE k_ticker = ?
        "#,
                ticker
            )
            .fetch_all(&self.pool)
            .await?;

            Ok(klines)
        }
    }

    async fn create_signals(&self, signal: Signal) -> Result<(), anyhow::Error> {
        let is_us = is_us(&signal.ticker);

        if is_us {
            sqlx::query_as!(
                Signal,
                "INSERT INTO signals_us (ticker, kdj_k, kdj_d) VALUES (?, ?, ?)",
                signal.ticker,
                signal.kdj_k,
                signal.kdj_d,
            )
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query_as!(
                Signal,
                "INSERT INTO signals (ticker, kdj_k, kdj_d) VALUES (?, ?, ?)",
                signal.ticker,
                signal.kdj_k,
                signal.kdj_d,
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn get_signals(&self, ticker: &str) -> Result<Signal, anyhow::Error> {
        let signal = sqlx::query_as!(
            Signal,
            r#"
            SELECT *
            FROM signals
            WHERE ticker = ?
        "#,
            ticker
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(signal)
    }

    async fn get_signals_all(&self) -> Result<Vec<Signal>, anyhow::Error> {
        let signals = sqlx::query_as!(
            Signal,
            r#"
            SELECT *
            FROM signals
        "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(signals)
    }

    async fn get_signals_all_us(&self) -> Result<Vec<Signal>, anyhow::Error> {
        let signals = sqlx::query_as!(
            Signal,
            r#"
            SELECT *
            FROM signals_us
        "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(signals)
    }
}

fn is_us(ticker: &str) -> bool {
    let idx = ticker.find('.').unwrap();
    let num = &ticker[..idx];
    let n = num.parse::<u32>().unwrap();
    (100..=110).contains(&n)
}

// FIX: more test coverage no need network call.
#[cfg(test)]
mod tests {
    use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

    use crate::{
        domain::{model::Kline, repository::DomainRepository},
        infra::storage::repo_domain_sqlite::SqliteDomainRepository,
    };

    async fn setup_test_db() -> Result<SqlitePool, sqlx::Error> {
        // Using a unique database URL for each test run enhances isolation.
        // In a real scenario, you might generate a unique name or use an in-memory database.
        let database_url = "sqlite::memory:";
        let pool = SqlitePoolOptions::new().connect(database_url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(pool)
    }

    pub fn generate_sequential_klines(count: usize, ticker: &str, start_date: i64) -> Vec<Kline> {
        let mut klines = Vec::with_capacity(count);

        for i in 0..count {
            let base_price = 100.0 + (i as f64 * 0.1); // Slowly increasing base price
            let open = base_price;
            let close = open - (i as f64 * 0.2);

            let high = open.max(close);
            let low = open.min(close);

            let volume = 100000.0 + (i as f64 * 0.1);
            let value = volume * (open + close) / 2.0;

            klines.push(Kline {
                k_ticker: ticker.to_string(),
                k_date: start_date + i as i64,
                k_open: open,
                k_high: high,
                k_low: low,
                k_close: close,
                k_volume: volume,
                k_value: value,
            });
        }

        klines
    }

    #[tokio::test]
    async fn test_create_klines() {
        let pool = setup_test_db().await.unwrap();

        let tickers = vec!["105.AAPL", "105.GOOGL", "105.MSFT"];
        let mut klines: Vec<Vec<Kline>> = vec![];
        for ticker in &tickers {
            let kline = generate_sequential_klines(8000, ticker, 20200101);
            klines.push(kline);
        }

        let repo = SqliteDomainRepository::new(pool.clone());

        for (idx, ticker) in tickers.iter().enumerate() {
            repo.create_klines(ticker, &klines[idx]).await.unwrap();
        }

        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) from klines_us")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count, 24000);

        // Check for clearing function.
        for (idx, ticker) in tickers.iter().enumerate() {
            repo.create_klines(ticker, &klines[idx]).await.unwrap();
        }

        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) from klines_us")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count, 24000);

        let klines_aapl = repo.get_klines("105.AAPL").await.unwrap();

        assert_eq!(klines_aapl.len(), 8000);
        assert_eq!(klines_aapl[0].k_ticker, "105.AAPL");
        assert_eq!(klines_aapl[1].k_ticker, "105.AAPL");
        assert_eq!(klines_aapl[0].k_date, 20200101);
        assert_eq!(klines_aapl[1].k_date, 20200102);
    }
}
