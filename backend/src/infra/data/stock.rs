use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    domain::model::Stock,
    infra::data::service::{parse_raw_eastmoney, url2text},
};

pub struct UrlStockEastmoney(String);

impl UrlStockEastmoney {
    pub fn new(ticker: &str) -> Self {
        let url = format!(
            "https://push2.eastmoney.com/api/qt/stock/get?invt=2&fltt=1&cb=jQuery35105046193517115448_1760700941707&fields=f57%2Cf58%2Cf105%2Cf107%2Cf116%2Cf164%2Cf167%2Cf183%2Cf187%2Cf188&secid={}&ut=fa5fd1943c7b386f172d6893dbfba10b&wbp2u=%7C0%7C0%7C0%7Cweb&dect=1&_=1760700941708",
            ticker
        );
        UrlStockEastmoney(url)
    }
}

/// Crawl stock meta from `eastmoney api`.
pub async fn crawl_stock_eastmoney(url: UrlStockEastmoney) -> Result<Stock, anyhow::Error> {
    let raw = url2text(&url.0).await?;
    let raw_stock: Result<RawStockEastmoney, _> = parse_raw_eastmoney(&raw);

    match raw_stock {
        Ok(res) => Ok(create_stock_eastmoney(res)),
        Err(e) => anyhow::bail!(e.to_string()),
    }
}

/// NOTE: can change in the future if [Stock] diverges from [RawStockEastmoney].
fn create_stock_eastmoney(raw_stock: RawStockEastmoney) -> Stock {
    let data = &raw_stock.data;
    Stock {
        ticker: format!("{}.{}", data.market, data.ticker),
        realname: data.name.clone(),
        market: data.market,
        pe: data.pe,
        pb: data.pb,
        total_cap: data.totalcap,
        revenue: data.revenue,
        net: data.net,
        margin: data.margin,
        debt: data.debt,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawStockEastmoney {
    #[serde(rename = "data")]
    pub data: RawStockEastmoneyData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawStockEastmoneyData {
    #[serde(rename = "f57")]
    pub ticker: String,
    #[serde(rename = "f107")]
    pub market: i64,
    #[serde(rename = "f58")]
    pub name: String,
    #[serde(rename = "f105", deserialize_with = "deserialize_optional_float")]
    pub net: Option<f64>,
    #[serde(rename = "f116", deserialize_with = "deserialize_optional_float")]
    pub totalcap: Option<f64>,
    #[serde(rename = "f164", deserialize_with = "deserialize_optional_float")]
    pub pe: Option<f64>,
    #[serde(rename = "f167", deserialize_with = "deserialize_optional_float")]
    pub pb: Option<f64>,
    #[serde(rename = "f183", deserialize_with = "deserialize_optional_float")]
    pub revenue: Option<f64>,
    #[serde(rename = "f187", deserialize_with = "deserialize_optional_float")]
    pub margin: Option<f64>,
    #[serde(rename = "f188", deserialize_with = "deserialize_optional_float")]
    pub debt: Option<f64>,
}

// Custom deserializer that handles null, "-", and valid numbers for f64.
fn deserialize_optional_float<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<serde_json::Value>::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::String(s)) if s == "-" || s.is_empty() => Ok(None),
        Some(serde_json::Value::String(s)) => {
            s.parse::<f64>().map(Some).map_err(serde::de::Error::custom)
        }
        Some(serde_json::Value::Number(n)) => Ok(n.as_f64()),
        Some(other) => Err(serde::de::Error::custom(format!(
            "expected null, \"-\", or number, found: {}",
            other
        ))),
    }
}

#[cfg(test)]
mod tests {
    use crate::infra::data::{
        service::parse_raw_eastmoney,
        stock::{
            RawStockEastmoney, RawStockEastmoneyData, UrlStockEastmoney, crawl_stock_eastmoney,
        },
    };

    const DEMO_STOCK_EASTMONEY_GOOD: &str = r#"jQuery35105046193517115448_1760700941707({"rc":0,"rt":4,"svr":183640384,"lt":1,"full":1,"dlmkts":"","data":{"f57":"TSLA","f58":"特斯拉","f105":0.0,"f107":105,"f116":1442516957364.52,"f164":24524,"f167":1866,"f183":41831000000.0,"f187":0.0,"f188":0.392752417028}});"#;
    const DEMO_STOCK_EASTMONEY_BAD: &str = r#"jQuery35105046193517115448_1760700941707({"rc":0,"rt":4,"svr":183640384,"lt":1,"full":1,"dlmkts":"","data":{"f57":"TSLA","f58":"特斯拉","f105":0.0,"f107":105,"f116":!,"f164":24524,"f167":1866,"f183":41831000000.0,"f187":0.0,"f188":0.392752417028}});"#;

    #[test]
    fn test_parse_raw_stock_eastmoney() {
        let result: Result<RawStockEastmoney, _> = parse_raw_eastmoney(DEMO_STOCK_EASTMONEY_GOOD);

        let expect = RawStockEastmoney {
            data: RawStockEastmoneyData {
                ticker: "TSLA".to_string(),
                name: "特斯拉".to_string(),
                market: 105,
                pe: Some(24524_f64),
                pb: Some(1866_f64),
                revenue: Some(41831000000.0),
                margin: Some(0.0),
                net: Some(0.0),
                totalcap: Some(1442516957364.52),
                debt: Some(0.392752417028),
            },
        };

        assert!(result.is_ok());
        assert_eq!(result.unwrap().data, expect.data);

        let result: Result<RawStockEastmoney, _> = parse_raw_eastmoney(DEMO_STOCK_EASTMONEY_BAD);

        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore = "network call to eastmoney"]
    async fn test_crawl_stock_eastmoney() {
        let url = UrlStockEastmoney::new("105.TSLA");

        let stock = crawl_stock_eastmoney(url).await;

        assert!(stock.is_ok());

        let stock = stock.unwrap();

        assert_eq!(stock.ticker, "105.TSLA");
        assert_eq!(stock.market, 105);
        assert_eq!(stock.realname, "特斯拉");
    }
}
