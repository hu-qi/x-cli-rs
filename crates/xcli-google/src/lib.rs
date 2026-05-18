use serde::{Deserialize, Serialize};
use tracing::info;
use xcli_browser::Browser;
use xcli_core::{Result, XCliError};
use xcli_webbridge::BrowserBridge;

pub const DEFAULT_GOOGLE_HL: &str = "en";

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub query: String,
    pub limit: usize,
    pub hl: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Debug, Deserialize)]
struct SearchPayload {
    consent: bool,
    items: Vec<SearchResult>,
}

#[derive(Debug, thiserror::Error)]
#[error("google served a consent interstitial; accept it once in Chrome and retry")]
pub struct ConsentRequired;

pub async fn search<B>(browser: &Browser<B>, options: SearchOptions) -> Result<Vec<SearchResult>>
where
    B: BrowserBridge,
{
    if options.query.trim().is_empty() {
        return Err(XCliError::InvalidArgs(
            "search requires a non-empty query".to_string(),
        ));
    }

    let limit = normalize_limit(options.limit);
    let hl = normalize_hl(&options.hl);
    let url = search_url(&options.query, limit, &hl);

    info!(step = "navigate", url = %url, "opening Google Search");
    browser.goto(&url).await?;

    info!(
        step = "extract",
        limit,
        hl = %hl,
        "extracting Google Search results"
    );
    let payload: SearchPayload = browser.eval(search_extract_script()).await?;

    if payload.consent {
        return Err(XCliError::BrowserActionFailed(ConsentRequired.to_string()));
    }

    let mut items = payload.items;
    if items.len() > limit {
        items.truncate(limit);
    }
    Ok(items)
}

pub fn search_url(query: &str, limit: usize, hl: &str) -> String {
    let request_n = normalize_limit(limit).saturating_mul(2).max(10);
    format!(
        "https://www.google.com/search?q={}&hl={}&num={}",
        urlencoding::encode(query),
        urlencoding::encode(&normalize_hl(hl)),
        request_n
    )
}

fn normalize_limit(limit: usize) -> usize {
    if limit == 0 { 10 } else { limit }
}

fn normalize_hl(hl: &str) -> String {
    let trimmed = hl.trim();
    if trimmed.is_empty() {
        DEFAULT_GOOGLE_HL.to_string()
    } else {
        trimmed.to_string()
    }
}

fn search_extract_script() -> &'static str {
    r#"
    (async () => {
      const deadline = Date.now() + 8000;
      while (Date.now() < deadline) {
        if (location.host.startsWith('consent.')) break;
        if (document.querySelector('div#search div[data-hveid] h3')) break;
        await new Promise(r => setTimeout(r, 150));
      }
      const seen = new Set();
      const items = Array.from(document.querySelectorAll('div#search div[data-hveid]'))
        .filter(el => el.querySelector('h3') && el.querySelector('a[href]'))
        .map(el => {
          const a = el.querySelector('a[href]');
          const h = el.querySelector('h3');
          return {
            title: h.innerText,
            url: a.href,
            snippet: (el.querySelector('[data-sncf]')?.innerText || '').replace(/\s*Read more\s*$/, '')
          };
        })
        .filter(r => { if (seen.has(r.url)) return false; seen.add(r.url); return true; });
      return { consent: location.host.startsWith('consent.'), items };
    })()
    "#
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, sync::Mutex};

    use async_trait::async_trait;
    use serde::de::DeserializeOwned;
    use serde_json::json;
    use xcli_webbridge::BridgeStatus;

    use super::*;

    #[test]
    fn builds_google_search_url() {
        let url = search_url("rust cli", 3, "zh-CN");
        assert_eq!(
            url,
            "https://www.google.com/search?q=rust%20cli&hl=zh-CN&num=10"
        );
    }

    #[tokio::test]
    async fn search_truncates_results() {
        let bridge = MockBridge::new(vec![json!({
            "consent": false,
            "items": [
                {"title":"one","url":"https://example.com/1","snippet":"a"},
                {"title":"two","url":"https://example.com/2","snippet":"b"},
                {"title":"three","url":"https://example.com/3","snippet":"c"}
            ]
        })]);
        let browser = Browser::new(bridge);

        let results = search(
            &browser,
            SearchOptions {
                query: "rust".to_string(),
                limit: 2,
                hl: "en".to_string(),
            },
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "one");
    }

    #[tokio::test]
    async fn search_rejects_empty_query() {
        let bridge = MockBridge::new(vec![]);
        let browser = Browser::new(bridge);

        let err = search(
            &browser,
            SearchOptions {
                query: " ".to_string(),
                limit: 10,
                hl: "en".to_string(),
            },
        )
        .await
        .unwrap_err();

        assert_eq!(err.code(), "invalid_args");
    }

    struct MockBridge {
        values: Mutex<VecDeque<serde_json::Value>>,
    }

    impl MockBridge {
        fn new(values: Vec<serde_json::Value>) -> Self {
            Self {
                values: Mutex::new(values.into()),
            }
        }
    }

    #[async_trait]
    impl BrowserBridge for MockBridge {
        async fn status(&self) -> Result<BridgeStatus> {
            Ok(BridgeStatus {
                running: true,
                extension_connected: true,
                extension_version: Some("test".to_string()),
                version: Some("test".to_string()),
            })
        }

        async fn navigate(&self, _url: &str) -> Result<()> {
            Ok(())
        }

        async fn eval<T>(&self, _javascript: &str) -> Result<T>
        where
            T: DeserializeOwned + Send,
        {
            let value = self
                .values
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| XCliError::BrowserActionFailed("mock exhausted".to_string()))?;
            serde_json::from_value(value)
                .map_err(|err| XCliError::BrowserActionFailed(err.to_string()))
        }
    }
}
