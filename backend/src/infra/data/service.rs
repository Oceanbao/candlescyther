use serde::Deserialize;

// url2text GET url and returns text results.
pub async fn url2text(url: &str) -> Result<String, anyhow::Error> {
    match ureq::get(url).call() {
        Ok(mut resp) => {
            let text = resp.body_mut().read_to_string()?;
            Ok(text)
        }
        Err(ureq::Error::StatusCode(code)) => {
            anyhow::bail!("HTTP error: {code}",)
        }
        Err(e) => {
            anyhow::bail!("Non-HTTP error: {e}",)
        }
    }
}

// Parse eastmoney jquery string results into T.
// NOTE: T impl Deserialize for any lifetime 'a. Most flexible works for owned or ref data.
pub fn parse_raw_eastmoney<T: for<'a> Deserialize<'a>>(raw: &str) -> Result<T, anyhow::Error> {
    let mut start_index = raw.find('(').ok_or(anyhow::anyhow!("( not found"))?;
    start_index += 1;
    let end_index = raw.find(')').ok_or(anyhow::anyhow!(") not found"))?;

    let parsed = String::from(&raw[start_index..end_index]);

    let result: Result<T, serde_json::Error> = serde_json::from_str(&parsed);

    result.map_err(|e| anyhow::anyhow!(e.to_string()))
}

#[cfg(test)]
mod tests {
    use crate::infra::data::service::url2text;

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

        let url = "https://dummyjson.com/http/404/bad";
        let text = url2text(url).await;

        match text {
            Ok(txt) => {
                panic!("Expected Err, but got Ok: {txt:?}");
            }
            Err(e) => {
                assert_eq!(e.to_string(), "HTTP error: 404");
            }
        }

        let url = "https://dummyjso.com/http/404/bad";
        let text = url2text(url).await;

        match text {
            Ok(txt) => {
                panic!("Expected Err, but got Ok: {txt:?}");
            }
            Err(e) => {
                assert_eq!(
                    e.to_string(),
                    "Non-HTTP error: io: failed to lookup address information: nodename nor servname provided, or not known"
                );
            }
        }
    }
}
