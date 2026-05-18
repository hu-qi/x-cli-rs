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
