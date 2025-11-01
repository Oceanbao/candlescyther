use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

use ureq::Agent;

const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 6.1; WOW64; Trident/7.0; SLCC2; .NET CLR 2.0.50727; .NET CLR 3.5.30729; .NET CLR 3.0.30729; .NET4.0C; .NET4.0E; rv:11.0) like Gecko",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36 Edg/134.0.0.0",
    "Mozilla/5.0 (X11; CrOS x86_64 14541.0.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.3.1 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 6.1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.10 Safari/605.1.1",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.3",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.3",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.3",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36 Trailer/93.3.8652.5",
];

// url2text GET url and returns text results.
pub async fn url2text(url: &str) -> Result<String, anyhow::Error> {
    // let proxy = Proxy::new("http://121.43.150.231:3128")?;
    // let agent: Agent = Agent::config_builder().proxy(Some(proxy)).build().into();
    let agent: Agent = Agent::config_builder().build().into();
    let user_agent = choose_random(USER_AGENTS).unwrap();

    match agent.get(url).header("User-Agent", *user_agent).call() {
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

fn choose_random<T>(slice: &[T]) -> Option<&T> {
    if slice.is_empty() {
        return None;
    }

    // Get current time in nanoseconds
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let nanos = now.as_nanos() as usize;
    let index = nanos % slice.len();

    Some(&slice[index])
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
