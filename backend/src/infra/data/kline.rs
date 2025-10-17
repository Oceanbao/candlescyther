use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::{
    domain::model::Kline,
    infra::data::service::{parse_raw_eastmoney, url2text},
};

// crawl_kline_eastmoney(url) -> Result<Vec<Kline>, Error>
// url2text(url) -> raw
// parse_raw_price_eastmoney(raw) -> RawPriceEastmoney
// parse_kline_eastmoney(RawPriceEastmoney) -> KlineEastmoney
// create_kline_eastmoney(RawPriceEastmoney) -> Vec<Kline>
pub async fn crawl_kline_eastmoney(url: &str) -> Result<Vec<Kline>, anyhow::Error> {
    let raw = url2text(url).await?;
    let raw_price: Result<RawPriceEastmoney, _> = parse_raw_eastmoney(&raw);

    match raw_price {
        Ok(res) => create_kline_eastmoney(res),
        Err(e) => anyhow::bail!(e.to_string()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawPriceEastmoney {
    #[serde(rename = "data")]
    data: RawPriceEastmoneyData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct RawPriceEastmoneyData {
    #[serde(rename = "code")]
    pub code: String,
    #[serde(rename = "market")]
    pub market: i32,
    #[serde(rename = "klines")]
    pub klines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineEastmoney {
    #[serde(rename = "date")]
    pub date: String,

    #[serde(rename = "open")]
    pub open: f64,

    #[serde(rename = "high")]
    pub high: f64,

    #[serde(rename = "low")]
    pub low: f64,

    #[serde(rename = "close")]
    pub close: f64,

    #[serde(rename = "volume")]
    pub volume: f64,

    #[serde(rename = "value")]
    pub value: f64,

    #[serde(rename = "volatility")]
    pub volatility: f64,

    #[serde(rename = "pchange")]
    pub pchange: f64,

    #[serde(rename = "change")]
    pub change: f64,

    #[serde(rename = "turnover")]
    pub turnover: f64,
}

pub fn parse_kline_eastmoney(input: &str) -> Result<KlineEastmoney, anyhow::Error> {
    let parts: Vec<&str> = input.split(',').collect();

    if parts.len() != 11 {
        anyhow::bail!(format!("Expected 11 fields, got {}", parts.len()));
    }

    let kline = KlineEastmoney {
        date: parts[0].to_string(),
        open: parts[1].parse::<f64>()?,
        close: parts[2].parse::<f64>()?,
        high: parts[3].parse::<f64>()?,
        low: parts[4].parse::<f64>()?,
        volume: parts[5].parse::<f64>()?,
        value: parts[6].parse::<f64>()?,
        volatility: parts[7].parse::<f64>()?,
        pchange: parts[8].parse::<f64>()?,
        change: parts[9].parse::<f64>()?,
        turnover: parts[10].parse::<f64>()?,
    };

    Ok(kline)
}

pub fn create_kline_eastmoney(
    price_eastmoney: RawPriceEastmoney,
) -> Result<Vec<Kline>, anyhow::Error> {
    let mut klines: Vec<Kline> = vec![];
    for kline_raw in price_eastmoney.data.klines {
        let kline = parse_kline_eastmoney(&kline_raw)?;
        klines.push(Kline {
            k_ticker: format!(
                "{}.{}",
                price_eastmoney.data.market, price_eastmoney.data.code
            ),
            k_date: date_string_to_i32(&kline.date)?,
            k_open: kline.open,
            k_high: kline.high,
            k_low: kline.low,
            k_close: kline.close,
            k_volume: kline.volume,
            k_value: kline.value,
        });
    }

    Ok(klines)
}

fn date_string_to_i32(date_str: &str) -> Result<i64, anyhow::Error> {
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;

    let year = date.year() as i64;
    let month = date.month() as i64;
    let day = date.day() as i64;

    Ok(year * 10000 + month * 100 + day)
}

#[cfg(test)]
mod tests {
    use crate::infra::data::kline::{
        RawPriceEastmoney, RawPriceEastmoneyData, crawl_kline_eastmoney, create_kline_eastmoney,
        parse_kline_eastmoney, parse_raw_eastmoney,
    };

    const DEMO_PRICE_EASTMONEY_GOOD: &str = r#"jQuery35105424247560587396_1758630789935({"rc":0,"rt":17,"svr":177617930,"lt":2,"full":0,"dlmkts":"","data":{"code":"APP","market":105,"name":"Applovin Corp-A","decimal":3,"dktotal":1125,"preKPrice":80.0,"klines":["2021-04-16,70.000,61.000,71.510,58.650,15643711,1034038718.000,16.08,-23.75,-19.000,4.37","2021-04-23,60.000,58.500,62.950,55.705,13380547,802760598.000,11.88,-4.10,-2.500,3.74","2021-04-30,58.770,58.010,61.110,57.650,2313034,136641797.000,5.91,-0.84,-0.490,0.65","2021-05-07,58.530,57.260,60.410,54.720,3922270,226305381.000,9.81,-1.29,-0.750,1.10","2021-05-14,59.210,57.260,59.210,49.410,7027414,375163594.000,17.11,0.00,0.000,1.93","2021-05-21,56.170,68.350,70.170,55.825,4603785,298832284.000,25.05,19.37,11.090,1.26"]}});"#;
    const DEMO_PRICE_EASTMONEY_BAD: &str = r#"jQuery35105424247560587396_1758630789935("rc":0,"rt":17,"svr":177617930,"lt":2,"full":0,"dlmkts":"","data":{"code":"APP","market":105,"name":"Applovin Corp-A","decimal":3,"dktotal":1125,"preKPrice":80.0,"klines":["2021-04-16,70.000,61.000,71.510,58.650,15643711,1034038718.000,16.08,-23.75,-19.000,4.37","2021-04-23,60.000,58.500,62.950,55.705,13380547,802760598.000,11.88,-4.10,-2.500,3.74","2021-04-30,58.770,58.010,61.110,57.650,2313034,136641797.000,5.91,-0.84,-0.490,0.65","2021-05-07,58.530,57.260,60.410,54.720,3922270,226305381.000,9.81,-1.29,-0.750,1.10","2021-05-14,59.210,57.260,59.210,49.410,7027414,375163594.000,17.11,0.00,0.000,1.93","2021-05-21,56.170,68.350,70.170,55.825,4603785,298832284.000,25.05,19.37,11.090,1.26"]}});"#;
    const DEMO_KLINE_EASTMONEY_GOOD: &str =
        "2021-04-16,70.000,61.000,71.510,58.650,15643711,1034038718.000,16.08,-23.75,-19.000,4.37";
    const DEMO_KLINE_EASTMONEY_BAD: &str =
        "2021-04-1670.000,61.000,71.510,58.650,15643711,1034038718.000,16.08,-23.75,-19.000,4.37";

    #[test]
    fn test_parse_price_eastmoney() {
        let result: Result<RawPriceEastmoney, _> = parse_raw_eastmoney(DEMO_PRICE_EASTMONEY_GOOD);

        let expect = RawPriceEastmoney {
            data: RawPriceEastmoneyData {
                code: "APP".to_string(),
                market: 105,
                klines: vec![
                    "2021-04-16,70.000,61.000,71.510,58.650,15643711,1034038718.000,16.08,-23.75,-19.000,4.37".to_string(),
                    "2021-04-23,60.000,58.500,62.950,55.705,13380547,802760598.000,11.88,-4.10,-2.500,3.74".to_string(),
                    "2021-04-30,58.770,58.010,61.110,57.650,2313034,136641797.000,5.91,-0.84,-0.490,0.65".to_string(),
                    "2021-05-07,58.530,57.260,60.410,54.720,3922270,226305381.000,9.81,-1.29,-0.750,1.10".to_string(),
                    "2021-05-14,59.210,57.260,59.210,49.410,7027414,375163594.000,17.11,0.00,0.000,1.93".to_string(),
                    "2021-05-21,56.170,68.350,70.170,55.825,4603785,298832284.000,25.05,19.37,11.090,1.26".to_string(),
                ],
            },
        };

        assert!(result.is_ok());
        assert_eq!(result.unwrap().data, expect.data);

        let result: Result<RawPriceEastmoney, _> = parse_raw_eastmoney(DEMO_PRICE_EASTMONEY_BAD);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_kline_eastmoney() {
        let result = parse_kline_eastmoney(DEMO_KLINE_EASTMONEY_GOOD);

        assert!(result.is_ok());
        let kline = result.unwrap();
        assert_eq!(kline.date, "2021-04-16");
        assert_eq!(kline.open, 70.000);

        let result = parse_kline_eastmoney(DEMO_KLINE_EASTMONEY_BAD);

        assert!(result.is_err());
    }

    #[test]
    fn test_create_kline_eastmoney() {
        let price_eastmoney: Result<RawPriceEastmoney, _> =
            parse_raw_eastmoney(DEMO_PRICE_EASTMONEY_GOOD);

        let result = create_kline_eastmoney(price_eastmoney.unwrap());

        assert!(result.is_ok());

        let result = result.unwrap();
        let first = result.first().unwrap();

        assert_eq!(first.k_ticker, "105.APP");
        assert_eq!(first.k_date, 20210416);
    }

    #[tokio::test]
    #[ignore = "network call to eastmoney"]
    async fn test_crawl_kline_eastmoney() {
        // 105.TSLA 20110126 - 20110202 1D
        let url = "https://54.push2his.eastmoney.com/api/qt/stock/kline/get?cb=jQuery35106707668456928451_1695010059469&secid=105.TSLA&ut=fa5fd1943c7b386f172d6893dbfba10b&fields1=f1%2Cf2%2Cf3%2Cf4%2Cf5%2Cf6&fields2=f51%2Cf52%2Cf53%2Cf54%2Cf55%2Cf56%2Cf57%2Cf58%2Cf59%2Cf60%2Cf61&klt=101&fqt=1&beg=0&end=20110202&lmt=1200&_=1695010059524";

        let result = crawl_kline_eastmoney(url).await;

        assert!(result.is_ok());

        let klines = result.unwrap();

        let first = klines.first().unwrap();
        let last = klines.last().unwrap();

        assert_eq!(first.k_ticker, "105.TSLA");
        assert_eq!(first.k_date, 20110126);
        assert_eq!(first.k_open, 1.647);
        assert_eq!(first.k_close, 1.650);
        assert_eq!(first.k_high, 1.659);
        assert_eq!(first.k_low, 1.607);
        assert_eq!(first.k_volume, 1078933.0);
        assert_eq!(first.k_value, 0.0);

        assert_eq!(last.k_ticker, "105.TSLA");
        assert_eq!(last.k_date, 20110202);
        assert_eq!(last.k_open, 1.611);
        assert_eq!(last.k_close, 1.596);
        assert_eq!(last.k_high, 1.612);
        assert_eq!(last.k_low, 1.577);
        assert_eq!(last.k_volume, 569472.0);
        assert_eq!(last.k_value, 0.0);
    }
}
