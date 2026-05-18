use std::{path::PathBuf, time::Instant};

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use time::{format_description::FormatItem, macros::format_description, OffsetDateTime};
use xcli_browser::Browser;
use xcli_core::{Result, XCliError};
use xcli_webbridge::BrowserBridge;

pub const CHATGPT_IMAGES_URL: &str = "https://chatgpt.com/images/";
pub const DEFAULT_IMAGE_SELECTOR: &str = "main img[src*='/backend-api/estuary/content']";

const FILE_TS_FORMAT: &[FormatItem<'_>] =
    format_description!("[year][month][day]-[hour][minute][second]");

#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub prompt: String,
    pub out_dir: PathBuf,
    pub timeout: std::time::Duration,
}

#[derive(Debug, Serialize)]
pub struct GenerateOutput {
    pub prompt: String,
    pub path: String,
    pub bytes: u64,
    pub caption: Option<String>,
    pub conversation_url: Option<String>,
    pub elapsed_ms: u128,
}

#[derive(Debug, serde::Deserialize)]
struct ImageMeta {
    src: Option<String>,
    alt: Option<String>,
    url: Option<String>,
}

pub async fn generate<B>(browser: &Browser<B>, options: GenerateOptions) -> Result<GenerateOutput>
where
    B: BrowserBridge,
{
    if options.prompt.trim().is_empty() {
        return Err(XCliError::InvalidArgs(
            "prompt must not be empty".to_string(),
        ));
    }

    std::fs::create_dir_all(&options.out_dir)
        .map_err(|err| XCliError::InvalidArgs(err.to_string()))?;

    let started = Instant::now();

    browser.ensure_ready().await?;
    browser.goto(CHATGPT_IMAGES_URL).await?;
    browser
        .insert_text("#prompt-textarea", &options.prompt)
        .await?;
    browser.click("#composer-submit-button").await?;

    browser
        .wait_for_js_truthy("location.href.includes('/c/')", options.timeout)
        .await?;

    browser
        .wait_for_js_truthy(
            "Boolean(document.querySelector(\"main img[src*='/backend-api/estuary/content']\"))",
            options.timeout,
        )
        .await?;

    let image_meta: ImageMeta = browser.eval(image_meta_script()).await?;
    let src = image_meta
        .src
        .ok_or_else(|| XCliError::GenerateFailed("image src not found".to_string()))?;

    let bytes_b64: String = browser.eval(&download_image_script(&src)).await?;
    let bytes = STANDARD
        .decode(bytes_b64)
        .map_err(|err| XCliError::GenerateFailed(err.to_string()))?;

    let timestamp = OffsetDateTime::now_utc()
        .format(FILE_TS_FORMAT)
        .map_err(|err| XCliError::GenerateFailed(err.to_string()))?;
    let path = options.out_dir.join(format!("chatgpt-{timestamp}.png"));
    std::fs::write(&path, &bytes).map_err(|err| XCliError::GenerateFailed(err.to_string()))?;

    Ok(GenerateOutput {
        prompt: options.prompt,
        path: path
            .canonicalize()
            .unwrap_or(path)
            .to_string_lossy()
            .to_string(),
        bytes: bytes.len() as u64,
        caption: image_meta.alt,
        conversation_url: image_meta.url,
        elapsed_ms: started.elapsed().as_millis(),
    })
}

fn image_meta_script() -> &'static str {
    r#"
    (() => {
      const img = document.querySelector("main img[src*='/backend-api/estuary/content']");
      return {
        src: img?.src ?? null,
        alt: img?.alt || null,
        url: location.href
      };
    })()
    "#
}

fn download_image_script(src: &str) -> String {
    format!(
        r#"
        (async () => {{
          const response = await fetch({src:?});
          const blob = await response.blob();
          const buffer = await blob.arrayBuffer();
          const bytes = new Uint8Array(buffer);
          let binary = '';
          for (const byte of bytes) binary += String.fromCharCode(byte);
          return btoa(binary);
        }})()
        "#,
    )
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, path::PathBuf, sync::Mutex, time::Duration};

    use async_trait::async_trait;
    use serde::de::DeserializeOwned;
    use serde_json::json;
    use xcli_webbridge::BridgeStatus;

    use super::*;

    #[tokio::test]
    async fn generate_writes_image_and_returns_metadata() {
        let out_dir = unique_temp_dir("success");
        let bridge = MockBridge::new(vec![
            json!(true),
            json!(true),
            json!(true),
            json!(true),
            json!({
                "src": "https://chatgpt.com/backend-api/estuary/content/test.png",
                "alt": "mock caption",
                "url": "https://chatgpt.com/c/mock"
            }),
            json!(STANDARD.encode(b"png-bytes")),
        ]);
        let browser = Browser::new(bridge);

        let output = generate(
            &browser,
            GenerateOptions {
                prompt: "a panda".to_string(),
                out_dir: out_dir.clone(),
                timeout: Duration::from_millis(1),
            },
        )
        .await
        .unwrap();

        assert_eq!(output.prompt, "a panda");
        assert_eq!(output.bytes, 9);
        assert_eq!(output.caption.as_deref(), Some("mock caption"));
        assert_eq!(
            output.conversation_url.as_deref(),
            Some("https://chatgpt.com/c/mock")
        );
        assert_eq!(std::fs::read(&output.path).unwrap(), b"png-bytes");

        let _ = std::fs::remove_dir_all(out_dir);
    }

    #[tokio::test]
    async fn generate_rejects_empty_prompt_before_browser_calls() {
        let out_dir = unique_temp_dir("empty");
        let bridge = MockBridge::new(vec![]);
        let browser = Browser::new(bridge);

        let err = generate(
            &browser,
            GenerateOptions {
                prompt: "   ".to_string(),
                out_dir: out_dir.clone(),
                timeout: Duration::from_millis(1),
            },
        )
        .await
        .unwrap_err();

        assert_eq!(err.code(), "invalid_args");
        assert!(!out_dir.exists());
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("xcli-chatgpt-image-{name}-{}", std::process::id()))
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
