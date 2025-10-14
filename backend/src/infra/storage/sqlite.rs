use std::collections::HashSet;

use sqlx::{Pool, Sqlite};

use crate::domain::model::Kline;

// TODO: This is specific to kline insert with elaborate steps.
// May want to split and redesign.
pub async fn insert_klines(pool: &Pool<Sqlite>, klines: Vec<Kline>) -> Result<(), sqlx::Error> {
    if klines.is_empty() {
        return Ok(());
    }

    // NOTE: Extract all tickers.
    let tickers: Vec<String> = klines
        .iter()
        .map(|k| k.k_ticker.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    // NOTE: Clear all records per tickers.
    for ticker in &tickers {
        let _ = sqlx::query!("DELETE FROM klines WHERE k_ticker = ?", ticker)
            .execute(pool)
            .await?;
    }

    let batch_size = 5000;
    let chunks: Vec<Vec<Kline>> = klines
        .chunks(batch_size)
        .map(|chunk| chunk.to_vec())
        .collect();

    for chunk in chunks {
        let mut tx = pool.begin().await?;

        // Batch commit.
        for kline in chunk {
            sqlx::query!(
                "INSERT INTO klines (k_ticker, k_date, k_open, k_high, k_low, k_close, k_volume, k_value) 
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
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
    }

    // FIX: remove this when stock crawl is done.
    let mut tx = pool.begin().await?;
    for ticker in &tickers {
        match sqlx::query!(
            "INSERT INTO stocks (ticker) 
                 VALUES (?)",
            ticker,
        )
        .execute(&mut *tx)
        .await
        {
            Ok(_) => {}
            Err(e) => match e {
                sqlx::Error::Database(e) => {
                    let msg = e.message();
                    if e.code().unwrap_or_default() == "1555" && msg.contains("UNIQUE constraint") {
                        tracing::debug!("Ticker already exists.")
                    }
                }
                _ => {
                    tracing::debug!("Failed to write to `stocks`: {}", e.to_string());
                }
            },
        }
    }
    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

    use crate::{
        domain::model::Stock,
        infra::storage::sqlite::{Kline, insert_klines},
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
    async fn test_insert_klines() {
        let pool = setup_test_db().await.unwrap();

        let tickers = vec!["105.AAPL", "105.GOOGL", "105.MSFT"];
        let mut klines: Vec<Kline> = vec![];
        for ticker in tickers {
            let kline = generate_sequential_klines(8000, ticker, 20200101);
            klines.extend(kline);
        }

        insert_klines(&pool, klines.clone()).await.unwrap();

        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) from klines")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count, 24000);

        // Check for clearing function.
        insert_klines(&pool, klines).await.unwrap();

        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) from klines")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count, 24000);

        let results: Vec<Kline> = sqlx::query_as(
            "SELECT k_ticker, k_date, k_open, k_high, k_low, k_close, k_volume, k_value FROM klines LIMIT 2"
        )
            .fetch_all(&pool)
            .await
            .unwrap();

        assert_eq!(results[0].k_ticker, "105.AAPL");
        assert_eq!(results[1].k_ticker, "105.AAPL");
        assert_eq!(results[0].k_date, 20200101);
        assert_eq!(results[1].k_date, 20200102);

        let results: Vec<Stock> = sqlx::query_as("SELECT ticker FROM stocks")
            .fetch_all(&pool)
            .await
            .unwrap();

        assert_eq!(results.len(), 3);
    }
}
