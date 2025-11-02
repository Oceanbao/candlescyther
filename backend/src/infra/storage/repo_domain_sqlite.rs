use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::{
    domain::{
        model::{Kline, Signal, Stock},
        repository::DomainRepository,
    },
    infra::data::moneyflow::MoneyflowEastmoney,
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
        let tx = self.pool.begin().await?;

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

        tx.commit().await?;

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
                "INSERT INTO signals_us (ticker, kdj_k, kdj_d, boll_dist) VALUES (?, ?, ?, ?)",
                signal.ticker,
                signal.kdj_k,
                signal.kdj_d,
                signal.boll_dist,
            )
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query_as!(
                Signal,
                "INSERT INTO signals (ticker, kdj_k, kdj_d, boll_dist) VALUES (?, ?, ?, ?)",
                signal.ticker,
                signal.kdj_k,
                signal.kdj_d,
                signal.boll_dist,
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
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

    async fn create_mf_sector(&self, flows: &[MoneyflowEastmoney]) -> Result<(), anyhow::Error> {
        let tx = self.pool.begin().await?;

        for flow in flows.iter() {
            sqlx::query!(
                "INSERT INTO moneyflow_sector (date_time, ticker, realname, lead_value, lead_share, super_value, super_share, large_value, large_share, mid_value, mid_share, small_value, small_share) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                flow.date_time,
                flow.ticker,
                flow.realname,
                flow.lead_value,
                flow.lead_share,
                flow.super_value,
                flow.super_share,
                flow.large_value,
                flow.large_share,
                flow.mid_value,
                flow.mid_share,
                flow.small_value,
                flow.small_share,
            ).execute(&self.pool)
                .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn get_mf_sector(&self) -> Result<Vec<MoneyflowEastmoney>, anyhow::Error> {
        let flows = sqlx::query_as!(MoneyflowEastmoney, "SELECT * FROM moneyflow_sector")
            .fetch_all(&self.pool)
            .await?;

        Ok(flows)
    }

    async fn delete_mf_sector(&self) -> Result<(), anyhow::Error> {
        sqlx::query!("DELETE FROM moneyflow_sector")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn create_signals_sector(&self, signal: Signal) -> Result<(), anyhow::Error> {
        sqlx::query!("DELETE FROM signals_sector WHERE ticker = ?", signal.ticker)
            .execute(&self.pool)
            .await?;

        sqlx::query_as!(
            Signal,
            "INSERT INTO signals_sector (ticker, kdj_k, kdj_d, boll_dist) VALUES (?, ?, ?, ?)",
            signal.ticker,
            signal.kdj_k,
            signal.kdj_d,
            signal.boll_dist,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_signals_all_sector(&self) -> Result<Vec<Signal>, anyhow::Error> {
        let signals = sqlx::query_as!(
            Signal,
            r#"
            SELECT *
            FROM signals_sector
        "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(signals)
    }

    async fn get_sector_tickers(&self) -> Result<Vec<Stock>, anyhow::Error> {
        let stock = sqlx::query_as!(
            Stock,
            r#"
            SELECT *
            FROM stocks
            WHERE ticker LIKE '90.%'
        "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(stock)
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
    use chrono::{Duration, NaiveDate};
    use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

    use crate::{
        domain::{model::Kline, repository::DomainRepository},
        infra::{
            data::{
                moneyflow::{MoneyflowEastmoney, RawMoneyflowEastmoney, create_moneyflow},
                service::parse_raw_eastmoney,
            },
            storage::repo_domain_sqlite::SqliteDomainRepository,
        },
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

    #[tokio::test]
    async fn test_create_ml_sector() {
        let pool = setup_test_db().await.unwrap();

        let repo = SqliteDomainRepository::new(pool.clone());

        const RAW_MONEYFLOW_SECTOR_EASTMONEY: &str = r#"jQuery112301083078708820121_1761565635682({"rc":0,"rt":6,"svr":181669432,"lt":1,"full":1,"dlmkts":"","data":{"total":86,"diff":[{"f1":2,"f2":35063.87,"f3":1.9,"f12":"BK0459","f13":90,"f14":"电子元件","f62":2672467968.0,"f66":2872657664.0,"f69":2.02,"f72":-200189696.0,"f75":-0.14,"f78":-1933110528.0,"f81":-1.36,"f84":-768382720.0,"f87":-0.54,"f124":1761550782,"f184":1.88,"f204":"胜宏科技","f205":"300476","f206":0},{"f1":2,"f2":18444.16,"f3":1.56,"f12":"BK0448","f13":90,"f14":"通信设备","f62":2602910976.0,"f66":3761489408.0,"f69":2.88,"f72":-1158578432.0,"f75":-0.89,"f78":-1900750336.0,"f81":-1.45,"f84":-711574272.0,"f87":-0.54,"f124":1761550782,"f184":1.99,"f204":"恒宝股份","f205":"002104","f206":0},{"f1":2,"f2":1987.6,"f3":1.54,"f12":"BK1037","f13":90,"f14":"消费电子","f62":2465783296.0,"f66":1903972096.0,"f69":1.94,"f72":561811200.0,"f75":0.57,"f78":-1500482304.0,"f81":-1.53,"f84":-998753792.0,"f87":-1.02,"f124":1761550782,"f184":2.52,"f204":"工业富联","f205":"601138","f206":1},{"f1":2,"f2":3109.46,"f3":2.39,"f12":"BK1027","f13":90,"f14":"小金属","f62":2056189696.0,"f66":2037394432.0,"f69":4.12,"f72":18795264.0,"f75":0.04,"f78":-770170368.0,"f81":-1.56,"f84":-1281168128.0,"f87":-2.59,"f124":1761550782,"f184":4.16,"f204":"厦门钨业","f205":"600549","f206":1},{"f1":2,"f2":2027.18,"f3":2.13,"f12":"BK1036","f13":90,"f14":"半导体","f62":1888555776.0,"f66":2031833856.0,"f69":0.83,"f72":-143278080.0,"f75":-0.06,"f78":-980680704.0,"f81":-0.4,"f84":-914381824.0,"f87":-0.38,"f124":1761550782,"f184":0.78,"f204":"兆易创新","f205":"603986","f206":1},{"f1":2,"f2":38790.99,"f3":0.83,"f12":"BK0465","f13":90,"f14":"化学制药","f62":1096540608.0,"f66":999148992.0,"f69":2.37,"f72":97391616.0,"f75":0.23,"f78":-981284352.0,"f81":-2.33,"f84":-143647744.0,"f87":-0.34,"f124":1761550782,"f184":2.6,"f204":"向日葵","f205":"300111","f206":0},{"f1":2,"f2":18347.21,"f3":1.36,"f12":"BK0428","f13":90,"f14":"电力行业","f62":1026856192.0,"f66":849184512.0,"f69":2.29,"f72":177671680.0,"f75":0.48,"f78":-1001579264.0,"f81":-2.7,"f84":-25276928.0,"f87":-0.07,"f124":1761550782,"f184":2.77,"f204":"中国核电","f205":"601985","f206":1},{"f1":2,"f2":9024.85,"f3":1.92,"f12":"BK0479","f13":90,"f14":"钢铁行业","f62":997690944.0,"f66":1004895040.0,"f69":5.74,"f72":-7204096.0,"f75":-0.04,"f78":-414445312.0,"f81":-2.37,"f84":-578170880.0,"f87":-3.3,"f124":1761550782,"f184":5.7,"f204":"新兴铸管","f205":"000778","f206":0},{"f1":2,"f2":33663.93,"f3":0.89,"f12":"BK0457","f13":90,"f14":"电网设备","f62":697530368.0,"f66":628551680.0,"f69":1.33,"f72":68978688.0,"f75":0.15,"f78":-144529664.0,"f81":-0.3,"f84":-603417088.0,"f87":-1.27,"f124":1761550782,"f184":1.47,"f204":"中国西电","f205":"601179","f206":1},{"f1":2,"f2":3500.02,"f3":0.93,"f12":"BK0910","f13":90,"f14":"专用设备","f62":645576192.0,"f66":684069888.0,"f69":0.86,"f72":-38493696.0,"f75":-0.05,"f78":-413467904.0,"f81":-0.52,"f84":-281076992.0,"f87":-0.35,"f124":1761550782,"f184":0.81,"f204":"英维克","f205":"002837","f206":0},{"f1":2,"f2":28302.66,"f3":0.58,"f12":"BK0425","f13":90,"f14":"工程建设","f62":616678496.0,"f66":506754144.0,"f69":1.8,"f72":109924352.0,"f75":0.39,"f78":-155837440.0,"f81":-0.55,"f84":-463444992.0,"f87":-1.65,"f124":1761550782,"f184":2.19,"f204":"中国电建","f205":"601669","f206":1},{"f1":2,"f2":1506.72,"f3":0.82,"f12":"BK0474","f13":90,"f14":"保险","f62":502335744.0,"f66":388679376.0,"f69":4.07,"f72":113656368.0,"f75":1.19,"f78":-406719424.0,"f81":-4.26,"f84":-95616336.0,"f87":-1.0,"f124":1761550782,"f184":5.26,"f204":"中国平安","f205":"601318","f206":1},{"f1":2,"f2":11688.06,"f3":1.27,"f12":"BK0484","f13":90,"f14":"贸易行业","f62":371387936.0,"f66":285093952.0,"f69":2.37,"f72":86293984.0,"f75":0.72,"f78":-119965136.0,"f81":-1.0,"f84":-258886784.0,"f87":-2.15,"f124":1761550782,"f184":3.09,"f204":"中电港","f205":"001287","f206":0},{"f1":2,"f2":47942.24,"f3":1.41,"f12":"BK0546","f13":90,"f14":"玻璃玻纤","f62":293145184.0,"f66":168121440.0,"f69":2.3,"f72":125023744.0,"f75":1.71,"f78":-94673024.0,"f81":-1.3,"f84":-179209440.0,"f87":-2.45,"f124":1761550782,"f184":4.01,"f204":"中材科技","f205":"002080","f206":0},{"f1":2,"f2":12567.57,"f3":0.51,"f12":"BK0539","f13":90,"f14":"综合行业","f62":284617232.0,"f66":321809104.0,"f69":4.11,"f72":-37191872.0,"f75":-0.47,"f78":-37467264.0,"f81":-0.48,"f84":-247149968.0,"f87":-3.15,"f124":1761550782,"f184":3.63,"f204":"东阳光","f205":"600673","f206":1},{"f1":2,"f2":12772.6,"f3":0.9,"f12":"BK0437","f13":90,"f14":"煤炭行业","f62":272668576.0,"f66":288082608.0,"f69":1.53,"f72":-15414032.0,"f75":-0.08,"f78":-38485504.0,"f81":-0.2,"f84":-234182912.0,"f87":-1.24,"f124":1761550782,"f184":1.45,"f204":"郑州煤电","f205":"600121","f206":1},{"f1":2,"f2":1087.13,"f3":2.34,"f12":"BK0729","f13":90,"f14":"船舶制造","f62":257691648.0,"f66":266805024.0,"f69":3.05,"f72":-9113376.0,"f75":-0.1,"f78":-195964128.0,"f81":-2.24,"f84":-61727520.0,"f87":-0.71,"f124":1761550782,"f184":2.94,"f204":"中国船舶","f205":"600150","f206":1},{"f1":2,"f2":2196.43,"f3":3.08,"f12":"BK1039","f13":90,"f14":"电子化学品","f62":255217344.0,"f66":585644992.0,"f69":2.33,"f72":-330427648.0,"f75":-1.31,"f78":-314919168.0,"f81":-1.25,"f84":59701760.0,"f87":0.24,"f124":1761550782,"f184":1.01,"f204":"晶瑞电材","f205":"300655","f206":0},{"f1":2,"f2":1350.48,"f3":0.97,"f12":"BK0730","f13":90,"f14":"农药兽药","f62":206520080.0,"f66":77003872.0,"f69":0.83,"f72":129516208.0,"f75":1.4,"f78":39617120.0,"f81":0.43,"f84":-255658768.0,"f87":-2.76,"f124":1761550782,"f184":2.23,"f204":"联化科技","f205":"002250","f206":0},{"f1":2,"f2":23845.93,"f3":1.06,"f12":"BK0478","f13":90,"f14":"有色金属","f62":189264384.0,"f66":111172096.0,"f69":0.18,"f72":78092288.0,"f75":0.12,"f78":443991040.0,"f81":0.71,"f84":-640158976.0,"f87":-1.02,"f124":1761550782,"f184":0.3,"f204":"紫金矿业","f205":"601899","f206":1},{"f1":2,"f2":15733.54,"f3":0.79,"f12":"BK0433","f13":90,"f14":"农牧饲渔","f62":184318400.0,"f66":299851392.0,"f69":1.79,"f72":-115532992.0,"f75":-0.69,"f78":-91829760.0,"f81":-0.55,"f84":-106748672.0,"f87":-0.64,"f124":1761550782,"f184":1.1,"f204":"牧原股份","f205":"002714","f206":0},{"f1":2,"f2":30942.83,"f3":1.0,"f12":"BK0538","f13":90,"f14":"化学制品","f62":141105904.0,"f66":212118000.0,"f69":0.49,"f72":-71012096.0,"f75":-0.16,"f78":-174632704.0,"f81":-0.4,"f84":47441152.0,"f87":0.11,"f124":1761550782,"f184":0.33,"f204":"天赐材料","f205":"002709","f206":0},{"f1":2,"f2":7908.95,"f3":0.17,"f12":"BK0422","f13":90,"f14":"物流行业","f62":134002704.0,"f66":189793424.0,"f69":1.76,"f72":-55790720.0,"f75":-0.52,"f78":-103790080.0,"f81":-0.96,"f84":-33989888.0,"f87":-0.32,"f124":1761550782,"f184":1.25,"f204":"胜通能源","f205":"001331","f206":0},{"f1":2,"f2":30530.19,"f3":0.97,"f12":"BK0424","f13":90,"f14":"水泥建材","f62":106185504.0,"f66":49538272.0,"f69":0.8,"f72":56647232.0,"f75":0.92,"f78":-48269104.0,"f81":-0.78,"f84":-57916384.0,"f87":-0.94,"f124":1761550782,"f184":1.72,"f204":"海螺水泥","f205":"600585","f206":1},{"f1":2,"f2":1285.58,"f3":1.16,"f12":"BK0727","f13":90,"f14":"医疗服务","f62":93195760.0,"f66":327629040.0,"f69":1.67,"f72":-234433280.0,"f75":-1.19,"f78":-302119168.0,"f81":-1.54,"f84":208923648.0,"f87":1.06,"f124":1761550782,"f184":0.47,"f204":"药明康德","f205":"603259","f206":1},{"f1":2,"f2":1024.65,"f3":0.29,"f12":"BK1041","f13":90,"f14":"医疗器械","f62":88950640.0,"f66":-221685904.0,"f69":-0.99,"f72":310636544.0,"f75":1.38,"f78":17168640.0,"f81":0.08,"f84":-114670592.0,"f87":-0.51,"f124":1761550782,"f184":0.4,"f204":"福瑞股份","f205":"300049","f206":0},{"f1":2,"f2":1128.72,"f3":1.07,"f12":"BK1016","f13":90,"f14":"汽车服务","f62":70827612.0,"f66":125612796.0,"f69":7.37,"f72":-54785184.0,"f75":-3.21,"f78":-76294032.0,"f81":-4.48,"f84":521312.0,"f87":0.03,"f124":1761550782,"f184":4.16,"f204":"漳州发展","f205":"000753","f206":0},{"f1":2,"f2":15298.11,"f3":-0.17,"f12":"BK0451","f13":90,"f14":"房地产开发","f62":58891120.0,"f66":-36992144.0,"f69":-0.16,"f72":95883264.0,"f75":0.41,"f78":-26480384.0,"f81":-0.11,"f84":-32410624.0,"f87":-0.14,"f124":1761550782,"f184":0.25,"f204":"首开股份","f205":"600376","f206":1},{"f1":2,"f2":1635.22,"f3":1.39,"f12":"BK0731","f13":90,"f14":"化肥行业","f62":46755136.0,"f66":212334640.0,"f69":2.25,"f72":-165579504.0,"f75":-1.76,"f78":-253192816.0,"f81":-2.68,"f84":206437680.0,"f87":2.19,"f124":1761550782,"f184":0.5,"f204":"盐湖股份","f205":"000792","f206":0},{"f1":2,"f2":483.59,"f3":0.49,"f12":"BK0734","f13":90,"f14":"珠宝首饰","f62":45087792.0,"f66":-30447968.0,"f69":-1.42,"f72":75535760.0,"f75":3.51,"f78":-2759760.0,"f81":-0.13,"f84":-42328016.0,"f87":-1.97,"f124":1761550782,"f184":2.1,"f204":"潮宏基","f205":"002345","f206":0},{"f1":2,"f2":693.44,"f3":0.33,"f12":"BK0738","f13":90,"f14":"多元金融","f62":41639344.0,"f66":86104544.0,"f69":1.26,"f72":-44465200.0,"f75":-0.65,"f78":-74005520.0,"f81":-1.08,"f84":32366160.0,"f87":0.47,"f124":1761550782,"f184":0.61,"f204":"中油资本","f205":"000617","f206":0},{"f1":2,"f2":715.47,"f3":2.17,"f12":"BK1015","f13":90,"f14":"能源金属","f62":36233600.0,"f66":38813056.0,"f69":0.16,"f72":-2579456.0,"f75":-0.01,"f78":-186657280.0,"f81":-0.78,"f84":150423552.0,"f87":0.63,"f124":1761550782,"f184":0.15,"f204":"华友钴业","f205":"603799","f206":1},{"f1":2,"f2":1759.77,"f3":0.83,"f12":"BK1020","f13":90,"f14":"非金属材料","f62":36116352.0,"f66":161883776.0,"f69":0.8,"f72":-125767424.0,"f75":-0.62,"f78":-47178752.0,"f81":-0.23,"f84":-10470656.0,"f87":-0.05,"f124":1761550782,"f184":0.18,"f204":"菲利华","f205":"300395","f206":0},{"f1":2,"f2":2632.39,"f3":0.34,"f12":"BK1030","f13":90,"f14":"电机","f62":27743792.0,"f66":182289472.0,"f69":1.05,"f72":-154545680.0,"f75":-0.89,"f78":-322093056.0,"f81":-1.85,"f84":232761344.0,"f87":1.34,"f124":1761550782,"f184":0.16,"f204":"卧龙电驱","f205":"600580","f206":1},{"f1":2,"f2":692.49,"f3":0.59,"f12":"BK0740","f13":90,"f14":"教育","f62":20278388.0,"f66":7033636.0,"f69":0.31,"f72":13244752.0,"f75":0.59,"f78":-39129168.0,"f81":-1.73,"f84":18850768.0,"f87":0.83,"f124":1761550782,"f184":0.9,"f204":"中公教育","f205":"002607","f206":0},{"f1":2,"f2":19150.0,"f3":0.8,"f12":"BK0454","f13":90,"f14":"塑料制品","f62":16523088.0,"f66":-34563472.0,"f69":-0.21,"f72":51086560.0,"f75":0.31,"f78":165054208.0,"f81":1.01,"f84":-211354368.0,"f87":-1.29,"f124":1761550782,"f184":0.1,"f204":"佛塑科技","f205":"000973","f206":0},{"f1":2,"f2":1235.4,"f3":0.4,"f12":"BK0733","f13":90,"f14":"包装材料","f62":12034057.0,"f66":471449.0,"f69":0.01,"f72":11562608.0,"f75":0.31,"f78":-27117552.0,"f81":-0.73,"f84":12477664.0,"f87":0.34,"f124":1761550782,"f184":0.33,"f204":"王子新材","f205":"002735","f206":0},{"f1":2,"f2":18673.43,"f3":0.44,"f12":"BK0482","f13":90,"f14":"商业百货","f62":-32728768.0,"f66":73189520.0,"f69":0.65,"f72":-105918288.0,"f75":-0.94,"f78":9347488.0,"f81":0.08,"f84":23381264.0,"f87":0.21,"f124":1761550782,"f184":-0.29,"f204":"国光连锁","f205":"605188","f206":1},{"f1":2,"f2":1128.47,"f3":0.61,"f12":"BK1042","f13":90,"f14":"医药商业","f62":-43339544.0,"f66":-4437864.0,"f69":-0.11,"f72":-38901680.0,"f75":-0.94,"f78":-34779520.0,"f81":-0.84,"f84":78119072.0,"f87":1.89,"f124":1761550782,"f184":-1.05,"f204":"九州通","f205":"600998","f206":1},{"f1":2,"f2":10602.94,"f3":0.23,"f12":"BK0470","f13":90,"f14":"造纸印刷","f62":-46049120.0,"f66":-80690000.0,"f69":-1.38,"f72":34640880.0,"f75":0.59,"f78":79723008.0,"f81":1.37,"f84":-42126336.0,"f87":-0.72,"f124":1761550782,"f184":-0.79,"f204":"中顺洁柔","f205":"002511","f206":0},{"f1":2,"f2":533.72,"f3":0.2,"f12":"BK0728","f13":90,"f14":"环保行业","f62":-67351584.0,"f66":-20234016.0,"f69":-0.1,"f72":-47117568.0,"f75":-0.24,"f78":142688768.0,"f81":0.72,"f84":-81471232.0,"f87":-0.41,"f124":1761550782,"f184":-0.34,"f204":"高能环境","f205":"603588","f206":1},{"f1":2,"f2":6204.4,"f3":0.2,"f12":"BK0464","f13":90,"f14":"石油行业","f62":-99684464.0,"f66":242880368.0,"f69":3.3,"f72":-342564832.0,"f75":-4.66,"f78":70292512.0,"f81":0.96,"f84":29391968.0,"f87":0.4,"f124":1761550782,"f184":-1.36,"f204":"中国石化","f205":"600028","f206":1},{"f1":2,"f2":1715.29,"f3":0.88,"f12":"BK1018","f13":90,"f14":"橡胶制品","f62":-117049728.0,"f66":-45177648.0,"f69":-0.6,"f72":-71872080.0,"f75":-0.95,"f78":-7609728.0,"f81":-0.1,"f84":86729680.0,"f87":1.15,"f124":1761550782,"f184":-1.55,"f204":"科创新源","f205":"300731","f206":0},{"f1":2,"f2":1223.82,"f3":-0.47,"f12":"BK1045","f13":90,"f14":"房地产服务","f62":-122941878.0,"f66":-69009094.0,"f69":-2.69,"f72":-53932784.0,"f75":-2.11,"f78":-15535600.0,"f81":-0.61,"f84":138477472.0,"f87":5.41,"f124":1761550782,"f184":-4.8,"f204":"我爱我家","f205":"000560","f206":0},{"f1":2,"f2":9115.6,"f3":0.01,"f12":"BK0427","f13":90,"f14":"公用事业","f62":-132585264.0,"f66":-82856112.0,"f69":-2.36,"f72":-49729152.0,"f75":-1.42,"f78":29656448.0,"f81":0.84,"f84":102928816.0,"f87":2.93,"f124":1761550782,"f184":-3.77,"f204":"海天股份","f205":"603759","f206":1},{"f1":2,"f2":16999.7,"f3":0.11,"f12":"BK0471","f13":90,"f14":"化纤行业","f62":-132674803.0,"f66":-112412947.0,"f69":-2.54,"f72":-20261856.0,"f75":-0.46,"f78":34904880.0,"f81":0.79,"f84":92093280.0,"f87":2.08,"f124":1761550782,"f184":-3.0,"f204":"恒力石化","f205":"600346","f206":1},{"f1":2,"f2":754.04,"f3":0.58,"f12":"BK0725","f13":90,"f14":"装修装饰","f62":-159712048.0,"f66":-180949088.0,"f69":-2.99,"f72":21237040.0,"f75":0.35,"f78":144701184.0,"f81":2.39,"f84":55498560.0,"f87":0.92,"f124":1761550782,"f184":-2.64,"f204":"中铁装配","f205":"300374","f206":0},{"f1":2,"f2":1880.31,"f3":0.25,"f12":"BK1028","f13":90,"f14":"燃气","f62":-159932336.0,"f66":-177324928.0,"f69":-2.51,"f72":17392592.0,"f75":0.25,"f78":-31213168.0,"f81":-0.44,"f84":182760304.0,"f87":2.59,"f124":1761550782,"f184":-2.27,"f204":"首华燃气","f205":"300483","f206":0},{"f1":2,"f2":11887.93,"f3":1.16,"f12":"BK0429","f13":90,"f14":"交运设备","f62":-173683152.0,"f66":-154603040.0,"f69":-1.47,"f72":-19080112.0,"f75":-0.18,"f78":75509008.0,"f81":0.72,"f84":94802976.0,"f87":0.9,"f124":1761550782,"f184":-1.65,"f204":"宗申动力","f205":"001696","f206":0},{"f1":2,"f2":5272.9,"f3":0.05,"f12":"BK0420","f13":90,"f14":"航空机场","f62":-186943328.0,"f66":-205286688.0,"f69":-4.64,"f72":18343360.0,"f75":0.41,"f78":56613232.0,"f81":1.28,"f84":130330080.0,"f87":2.94,"f124":1761550782,"f184":-4.22,"f204":"南方航空","f205":"600029","f206":1}]}});"#;

        let raw_moneyflow: RawMoneyflowEastmoney =
            parse_raw_eastmoney(RAW_MONEYFLOW_SECTOR_EASTMONEY).unwrap();
        let moneyflow = create_moneyflow(raw_moneyflow);

        let total = 40;
        let mut moneyflow_batch: Vec<Vec<MoneyflowEastmoney>> = vec![];
        for day in 1..total {
            let mut mf_copy = moneyflow.clone();
            for x in mf_copy.iter_mut() {
                let mut new_dt = NaiveDate::parse_from_str(&x.date_time, "%Y-%m-%d").unwrap();
                new_dt += Duration::days(day);
                x.date_time = new_dt.to_string();
            }
            moneyflow_batch.push(mf_copy);
        }

        for mf in moneyflow_batch {
            repo.create_mf_sector(&mf).await.unwrap();
        }

        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) from moneyflow_sector")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count, 930);

        let ticker_excluded = "90.BK0420";
        let records = repo.get_mf_sector().await.unwrap();

        assert_eq!(records.len(), 930);

        let filtered: Vec<MoneyflowEastmoney> = records
            .into_iter()
            .filter(|item| item.ticker == ticker_excluded)
            .collect();

        assert_eq!(filtered.len(), 0);
    }
}
