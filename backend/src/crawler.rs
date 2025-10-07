use serde::{Deserialize, Serialize};

// url2text handles text results.
pub async fn url2text(url: &str) -> Result<String, anyhow::Error> {
    let mut resp = ureq::get(url).call()?;

    let text = resp.body_mut().read_to_string()?;

    if resp.status().is_success() {
        Ok(text)
    } else {
        anyhow::bail!(
            "HTTP request failed with status: {} and message: {}",
            resp.status(),
            text
        )
    }
}

// parse_price_eastmoney takes text results from eastmoney
// and returns the right struct.
// parse_price_eastmoney(text) -> Result<PriceEastmoney, anyhow::Error>
// - define PriceEastmoney
// - define Error

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawPriceEastmoney {
    #[serde(rename = "data")]
    pub data: RawPriceEastmoneyData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawPriceEastmoneyData {
    #[serde(rename = "code")]
    pub code: String,
    #[serde(rename = "market")]
    pub market: i32,
    #[serde(rename = "klines")]
    pub klines: Vec<String>,
}

pub fn parse_raw_price_eastmoney(raw: &str) -> Option<RawPriceEastmoney> {
    let mut start_index = raw.find('(')?;
    start_index += 1;
    let end_index = raw.find(')')?;

    let parsed = String::from(&raw[start_index..end_index]);

    let result: Result<RawPriceEastmoney, serde_json::Error> = serde_json::from_str(&parsed);

    match result {
        Ok(res) => Some(res),
        Err(e) => {
            tracing::error!("Failed to parse_price_eastmoney {e:#?}");
            None
        }
    }
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
        high: parts[2].parse::<f64>()?,
        low: parts[3].parse::<f64>()?,
        close: parts[4].parse::<f64>()?,
        volume: parts[5].parse::<f64>()?,
        value: parts[6].parse::<f64>()?,
        volatility: parts[7].parse::<f64>()?,
        pchange: parts[8].parse::<f64>()?,
        change: parts[9].parse::<f64>()?,
        turnover: parts[10].parse::<f64>()?,
    };

    Ok(kline)
}

#[cfg(test)]
mod tests {
    use crate::crawler::{
        RawPriceEastmoney, RawPriceEastmoneyData, parse_kline_eastmoney, parse_raw_price_eastmoney,
        url2text,
    };

    const DEMO_PRICE_EASTMONEY_GOOD: &str = r#"jQuery35105424247560587396_1758630789935({"rc":0,"rt":17,"svr":177617930,"lt":2,"full":0,"dlmkts":"","data":{"code":"APP","market":105,"name":"Applovin Corp-A","decimal":3,"dktotal":1125,"preKPrice":80.0,"klines":["2021-04-16,70.000,61.000,71.510,58.650,15643711,1034038718.000,16.08,-23.75,-19.000,4.37","2021-04-23,60.000,58.500,62.950,55.705,13380547,802760598.000,11.88,-4.10,-2.500,3.74","2021-04-30,58.770,58.010,61.110,57.650,2313034,136641797.000,5.91,-0.84,-0.490,0.65","2021-05-07,58.530,57.260,60.410,54.720,3922270,226305381.000,9.81,-1.29,-0.750,1.10","2021-05-14,59.210,57.260,59.210,49.410,7027414,375163594.000,17.11,0.00,0.000,1.93","2021-05-21,56.170,68.350,70.170,55.825,4603785,298832284.000,25.05,19.37,11.090,1.26"]}});"#;
    const DEMO_PRICE_EASTMONEY_BAD: &str = r#"jQuery35105424247560587396_1758630789935("rc":0,"rt":17,"svr":177617930,"lt":2,"full":0,"dlmkts":"","data":{"code":"APP","market":105,"name":"Applovin Corp-A","decimal":3,"dktotal":1125,"preKPrice":80.0,"klines":["2021-04-16,70.000,61.000,71.510,58.650,15643711,1034038718.000,16.08,-23.75,-19.000,4.37","2021-04-23,60.000,58.500,62.950,55.705,13380547,802760598.000,11.88,-4.10,-2.500,3.74","2021-04-30,58.770,58.010,61.110,57.650,2313034,136641797.000,5.91,-0.84,-0.490,0.65","2021-05-07,58.530,57.260,60.410,54.720,3922270,226305381.000,9.81,-1.29,-0.750,1.10","2021-05-14,59.210,57.260,59.210,49.410,7027414,375163594.000,17.11,0.00,0.000,1.93","2021-05-21,56.170,68.350,70.170,55.825,4603785,298832284.000,25.05,19.37,11.090,1.26"]}});"#;
    const DEMO_META_EASTMONEY_GOOD: &str = r#"jQuery35105571137681219451_1708499614785({"rc":0,"rt":4,"svr":177622158,"lt":2,"full":1,"dlmkts":"8,10,128","data":{"f55":1.768942283,"f57":"TSLA","f58":"特斯拉","f59":3,"f62":2,"f84":3325150886.0,"f85":3325150886.0,"f92":23.2512757,"f105":0.0,"f107":105,"f116":1473208100042.3,"f117":1473208100042.3,"f152":2,"f162":"-","f167":1905,"f173":2.1,"f183":41831000000.0,"f184":0.0,"f185":0.0,"f186":0.0,"f187":0.0,"f188":0.392752417028,"f189":20100629,"f190":0.0}});"#;
    const DEMO_META_EASTMONEY_BAD: &str = r#"jQuery35105571137681219451_1708499614785("rc":0,"rt":4,"svr":177622158,"lt":2,"full":1,"dlmkts":"8,10,128","data":{"f55":1.768942283,"f57":"TSLA","f58":"特斯拉","f59":3,"f62":2,"f84":3325150886.0,"f85":3325150886.0,"f92":23.2512757,"f105":0.0,"f107":105,"f116":1473208100042.3,"f117":1473208100042.3,"f152":2,"f162":"-","f167":1905,"f173":2.1,"f183":41831000000.0,"f184":0.0,"f185":0.0,"f186":0.0,"f187":0.0,"f188":0.392752417028,"f189":20100629,"f190":0.0}});"#;
    const DEMO_KLINE_EASTMONEY_GOOD: &str =
        "2021-04-16,70.000,61.000,71.510,58.650,15643711,1034038718.000,16.08,-23.75,-19.000,4.37";
    const DEMO_KLINE_EASTMONEY_BAD: &str =
        "2021-04-1670.000,61.000,71.510,58.650,15643711,1034038718.000,16.08,-23.75,-19.000,4.37";

    #[tokio::test]
    async fn test_url2text() {
        let url = "https://dummyjson.com/test";
        let text = url2text(url).await;

        match text {
            Ok(txt) => {
                assert_eq!(txt, "{\"status\":\"ok\",\"method\":\"GET\"}");
            }
            Err(e) => {
                panic!("Expected Ok, but got err: {e:?}");
            }
        }

        let url = "https://dummyjson.com/tes";
        let text = url2text(url).await;

        match text {
            Ok(txt) => {
                panic!("Expected Err, but got Ok: {txt:?}");
            }
            Err(e) => {
                assert_eq!(e.to_string(), "http status: 404");
            }
        }
    }

    #[test]
    fn test_parse_price_eastmoney() {
        let result = parse_raw_price_eastmoney(DEMO_PRICE_EASTMONEY_GOOD);

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

        assert!(result.is_some());
        assert_eq!(result.unwrap().data, expect.data);

        let result = parse_raw_price_eastmoney(DEMO_PRICE_EASTMONEY_BAD);

        assert!(result.is_none());
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
}
