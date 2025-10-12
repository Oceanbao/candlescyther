use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, prelude::FromRow};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Kline {
    pub k_ticker: String,
    pub k_date: i32,
    pub k_open: f64,
    pub k_high: f64,
    pub k_low: f64,
    pub k_close: f64,
    pub k_volume: f64,
    pub k_value: f64,
}

pub async fn insert_klines(pool: &Pool<Sqlite>, klines: Vec<Kline>) -> Result<(), sqlx::Error> {
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
                "INSERT INTO kline (k_ticker, k_date, k_open, k_high, k_low, k_close, k_volume, k_value) 
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

    use crate::infra::storage::repository::{Kline, insert_klines};

    async fn setup_test_db() -> Result<SqlitePool, sqlx::Error> {
        // Using a unique database URL for each test run enhances isolation.
        // In a real scenario, you might generate a unique name or use an in-memory database.
        let database_url = "sqlite::memory:";
        let pool = SqlitePoolOptions::new().connect(database_url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(pool)
    }

    pub fn generate_sequential_klines(count: usize, ticker: &str, start_date: i32) -> Vec<Kline> {
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
                k_date: start_date + i as i32,
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

        insert_klines(&pool, klines).await.unwrap();

        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) from kline")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count, 24000);

        let results: Vec<Kline> = sqlx::query_as(
            "SELECT k_ticker, k_date, k_open, k_high, k_low, k_close, k_volume, k_value FROM kline LIMIT 2"
        )
            .fetch_all(&pool)
            .await
            .unwrap();

        assert_eq!(results[0].k_ticker, "105.AAPL");
        assert_eq!(results[1].k_ticker, "105.AAPL");
        assert_eq!(results[0].k_date, 20200101);
        assert_eq!(results[1].k_date, 20200102);
    }
}
