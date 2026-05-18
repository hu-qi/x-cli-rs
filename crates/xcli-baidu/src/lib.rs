use serde::{Deserialize, Serialize};
use tracing::info;
use xcli_browser::Browser;
use xcli_core::{Result, XCliError};
use xcli_webbridge::BrowserBridge;

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub query: String,
    pub limit: usize,
    pub include_all: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchResult {
    pub rank: usize,
    pub id: String,
    pub tpl: String,
    pub title: String,
    pub url: String,
    #[serde(rename = "abstract")]
    pub abstract_text: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SearchOutput {
    pub query: String,
    pub count: usize,
    pub results: Vec<SearchResult>,
}

pub async fn search<B>(browser: &Browser<B>, options: SearchOptions) -> Result<SearchOutput>
where
    B: BrowserBridge,
{
    if options.query.trim().is_empty() {
        return Err(XCliError::MissingArgs(
            "search requires a query: baidu-cli search <query>".to_string(),
        ));
    }

    let limit = normalize_limit(options.limit);
    let url = search_url(&options.query, limit);

    info!(step = "navigate", url = %url, "opening Baidu Search");
    browser.goto(&url).await.map_err(map_search_error)?;

    info!(
        step = "extract",
        limit,
        include_all = options.include_all,
        "extracting Baidu Search results"
    );
    let mut results: Vec<SearchResult> = browser
        .eval(extractor_script())
        .await
        .map_err(map_search_error)?;

    if !options.include_all {
        results = filter_organic(results);
    }
    if results.len() > limit {
        results.truncate(limit);
    }
    for (idx, item) in results.iter_mut().enumerate() {
        item.rank = idx + 1;
    }

    Ok(SearchOutput {
        query: options.query,
        count: results.len(),
        results,
    })
}

pub fn search_url(query: &str, limit: usize) -> String {
    format!(
        "https://www.baidu.com/s?wd={}&rn={}",
        urlencoding::encode(query),
        normalize_limit(limit)
    )
}

fn normalize_limit(limit: usize) -> usize {
    if limit == 0 {
        10
    } else {
        limit
    }
}

fn map_search_error(err: XCliError) -> XCliError {
    match err {
        XCliError::DaemonUnreachable(_)
        | XCliError::DaemonNotRunning
        | XCliError::ExtensionNotConnected => err,
        other => XCliError::SearchFailed(other.to_string()),
    }
}

fn filter_organic(results: Vec<SearchResult>) -> Vec<SearchResult> {
    // Keep parity with the original Go version for now: pass-through filter.
    // The source repository intentionally left this as a user decision because
    // Baidu mixes organic results, Baike cards, AI answers, and recommendations.
    results
}

fn extractor_script() -> &'static str {
    r#"
    (() => {
      const items = document.querySelectorAll(".result.c-container, .result-op.c-container");
      return Array.from(items).map((el, i) => {
        const titleEl = el.querySelector("h3 a") || el.querySelector(".t a");
        const absEl = el.querySelector("[class*=summary-text]")
          || el.querySelector("[class*=abstract]")
          || el.querySelector(".c-abstract")
          || el.querySelector("[class*=paragraph]");
        const sourceEl = el.querySelector("[class*=source-text]")
          || el.querySelector("[class*=source]");
        return {
          rank: i + 1,
          id: el.id || "",
          tpl: el.getAttribute("tpl") || "",
          title: titleEl ? titleEl.innerText.trim() : "",
          url: el.getAttribute("mu") || (titleEl && titleEl.href) || "",
          abstract: absEl ? absEl.innerText.trim().replace(/\s+/g, " ").slice(0, 400) : "",
          source: sourceEl ? sourceEl.innerText.trim().replace(/\s+/g, " ").slice(0, 80) : ""
        };
      });
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
    fn builds_baidu_search_url() {
        let url = search_url("天气 北京", 20);
        assert_eq!(
            url,
            "https://www.baidu.com/s?wd=%E5%A4%A9%E6%B0%94%20%E5%8C%97%E4%BA%AC&rn=20"
        );
    }

    #[tokio::test]
    async fn search_returns_wrapped_output_and_reranks() {
        let bridge = MockBridge::new(vec![json!([
            {"rank": 9, "id":"a", "tpl":"www_index", "title":"one", "url":"https://example.com/1", "abstract":"a", "source":"s1"},
            {"rank": 10, "id":"b", "tpl":"www_index", "title":"two", "url":"https://example.com/2", "abstract":"b", "source":"s2"},
            {"rank": 11, "id":"c", "tpl":"www_index", "title":"three", "url":"https://example.com/3", "abstract":"c", "source":"s3"}
        ])]);
        let browser = Browser::new(bridge);

        let output = search(
            &browser,
            SearchOptions {
                query: "rust".to_string(),
                limit: 2,
                include_all: false,
            },
        )
        .await
        .unwrap();

        assert_eq!(output.query, "rust");
        assert_eq!(output.count, 2);
        assert_eq!(output.results[0].rank, 1);
        assert_eq!(output.results[1].rank, 2);
        assert_eq!(output.results[0].title, "one");
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
                include_all: false,
            },
        )
        .await
        .unwrap_err();

        assert_eq!(err.code(), "missing_args");
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
