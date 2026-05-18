use std::{io::Cursor, path::PathBuf, time::Instant};

use base64::{engine::general_purpose::STANDARD, Engine};
use image::{imageops::FilterType, GenericImageView, ImageFormat};
use serde::{Deserialize, Serialize};
use time::{format_description::FormatItem, macros::format_description, OffsetDateTime};
use tracing::info;
use xcli_browser::Browser;
use xcli_core::{Result, XCliError};
use xcli_webbridge::BrowserBridge;

pub const GEMINI_URL: &str = "https://gemini.google.com/";

const FILE_TS_FORMAT: &[FormatItem<'_>] =
    format_description!("[year][month][day]-[hour][minute][second]");

#[derive(Debug, Clone)]
pub struct GenOptions {
    pub prompt: String,
    pub out_dir: PathBuf,
    pub thumb_width: u32,
    pub timeout: std::time::Duration,
}

#[derive(Debug, Serialize)]
pub struct GenOutput {
    pub prompt: String,
    pub full: String,
    pub thumb: String,
    pub width: u32,
    pub height: u32,
    pub thumb_width: u32,
    pub elapsed_ms: u128,
}

#[derive(Debug, Deserialize)]
struct BoolResult {
    ok: bool,
    #[serde(default)]
    err: String,
}

#[derive(Debug, Deserialize)]
struct FetchImageResult {
    ok: bool,
    #[serde(default)]
    err: String,
    #[serde(default)]
    status: u16,
    #[serde(default, rename = "contentType")]
    content_type: String,
    #[serde(default)]
    size: usize,
    #[serde(default)]
    base64: String,
}

pub async fn gen<B>(browser: &Browser<B>, options: GenOptions) -> Result<GenOutput>
where
    B: BrowserBridge,
{
    if options.prompt.trim().is_empty() {
        return Err(XCliError::InvalidArgs("prompt is empty".to_string()));
    }

    let thumb_width = if options.thumb_width == 0 {
        256
    } else {
        options.thumb_width
    };
    std::fs::create_dir_all(&options.out_dir)
        .map_err(|err| XCliError::InvalidArgs(err.to_string()))?;

    let started = Instant::now();

    info!(step = "status", "checking kimi-webbridge status");
    browser.ensure_ready().await?;

    info!(step = "navigate", url = GEMINI_URL, "opening Gemini");
    browser.goto(GEMINI_URL).await.map_err(map_gen_error)?;

    info!(step = "wait_textbox", "waiting for Gemini prompt textbox");
    browser
        .wait_for_js_truthy(
            "Boolean(document.querySelector('div[contenteditable=\"true\"][role=\"textbox\"]'))",
            std::time::Duration::from_secs(15),
        )
        .await
        .map_err(map_gen_error)?;

    info!(step = "input", "injecting prompt");
    eval_bool(
        browser,
        &inject_prompt_script(&options.prompt),
        "inject prompt",
    )
    .await?;

    info!(step = "submit", "clicking send button");
    eval_bool(browser, click_send_script(), "click send").await?;

    info!(
        step = "wait_image",
        timeout_ms = options.timeout.as_millis(),
        "waiting for displayed image"
    );
    browser
        .wait_for_js_truthy(displayed_image_ready_script(), options.timeout)
        .await
        .map_err(map_gen_error)?;

    info!(
        step = "install_download_hook",
        "installing Gemini download fetch hook"
    );
    eval_bool(
        browser,
        install_download_hook_script(),
        "install download hook",
    )
    .await?;

    info!(
        step = "click_download",
        "clicking download full-size button"
    );
    eval_bool(browser, click_download_script(), "click download").await?;

    info!(step = "fetch_image", "fetching intercepted full-size image");
    let png_bytes = fetch_intercepted_image(browser).await?;
    let (width, height) = png_dimensions(&png_bytes)?;

    let timestamp = OffsetDateTime::now_utc()
        .format(FILE_TS_FORMAT)
        .map_err(|err| XCliError::GenerateFailed(err.to_string()))?;
    let full_path = options.out_dir.join(format!("{timestamp}-full.png"));
    let thumb_path = options.out_dir.join(format!("{timestamp}-thumb.png"));

    info!(step = "write_full", path = %full_path.display(), bytes = png_bytes.len(), "writing full-size image");
    std::fs::write(&full_path, &png_bytes)
        .map_err(|err| XCliError::GenerateFailed(err.to_string()))?;

    info!(step = "write_thumb", path = %thumb_path.display(), thumb_width, "writing thumbnail");
    write_thumbnail(&png_bytes, &thumb_path, thumb_width)?;

    Ok(GenOutput {
        prompt: options.prompt,
        full: full_path
            .canonicalize()
            .unwrap_or(full_path)
            .to_string_lossy()
            .to_string(),
        thumb: thumb_path
            .canonicalize()
            .unwrap_or(thumb_path)
            .to_string_lossy()
            .to_string(),
        width,
        height,
        thumb_width,
        elapsed_ms: started.elapsed().as_millis(),
    })
}

async fn eval_bool<B>(browser: &Browser<B>, script: &str, label: &str) -> Result<()>
where
    B: BrowserBridge,
{
    let out: BoolResult = browser.eval(script).await.map_err(map_gen_error)?;
    if out.ok {
        Ok(())
    } else {
        Err(XCliError::GenerateFailed(format!(
            "{label} failed: {}",
            out.err
        )))
    }
}

async fn fetch_intercepted_image<B>(browser: &Browser<B>) -> Result<Vec<u8>>
where
    B: BrowserBridge,
{
    let out: FetchImageResult = browser
        .eval(fetch_intercepted_image_script())
        .await
        .map_err(map_gen_error)?;
    if !out.ok {
        return Err(XCliError::GenerateFailed(format!(
            "fetch intercepted url failed: {} (status={})",
            out.err, out.status
        )));
    }
    if !out.content_type.starts_with("image/") {
        return Err(XCliError::GenerateFailed(format!(
            "unexpected content-type: {} (size={})",
            out.content_type, out.size
        )));
    }
    STANDARD
        .decode(out.base64)
        .map_err(|err| XCliError::GenerateFailed(format!("base64 decode: {err}")))
}

fn map_gen_error(err: XCliError) -> XCliError {
    match err {
        XCliError::DaemonUnreachable(_)
        | XCliError::DaemonNotRunning
        | XCliError::ExtensionNotConnected => err,
        other => XCliError::GenerateFailed(other.to_string()),
    }
}

fn inject_prompt_script(prompt: &str) -> String {
    format!(
        r#"
        (() => {{
          const tb = document.querySelector('div[contenteditable="true"][role="textbox"]');
          if (!tb) return {{ ok: false, err: 'textbox_not_found' }};
          tb.focus();
          document.execCommand('selectAll', false, null);
          document.execCommand('insertText', false, {prompt:?});
          tb.dispatchEvent(new InputEvent('input', {{ bubbles: true, inputType: 'insertText', data: {prompt:?} }}));
          return {{ ok: true }};
        }})()
        "#,
    )
}

fn click_send_script() -> &'static str {
    r#"
    (() => {
      const selectors = ['button.send-button','button[aria-label="发送"]','button[aria-label="Send"]'];
      for (const sel of selectors) {
        const b = document.querySelector(sel);
        if (b && !b.disabled) { b.click(); return { ok: true }; }
      }
      return { ok: false, err: 'send_button_not_found' };
    })()
    "#
}

fn displayed_image_ready_script() -> &'static str {
    r#"
    (() => {
      const img = document.querySelector('generated-image img, .generated-image img, single-image img');
      return Boolean(img && img.complete && img.naturalWidth > 0);
    })()
    "#
}

fn click_download_script() -> &'static str {
    r#"
    (() => {
      const b = document.querySelector('[data-test-id="download-generated-image-button"]');
      if (!b) return { ok: false, err: 'download_button_not_found' };
      b.click();
      return { ok: true };
    })()
    "#
}

fn install_download_hook_script() -> &'static str {
    r#"
    (() => {
      if (window.__nbHookV3) return { ok: true, already: true };
      window.__nbHookV3 = true;
      window.__nbFinalURL = null;
      window.__nbFinalURLAt = 0;
      if (!window.__nbOrigFetch) window.__nbOrigFetch = window.fetch;
      const origFetch = window.__nbOrigFetch;
      window.fetch = async function(input, init) {
        const url = typeof input === 'string' ? input : (input && input.url) || '';
        if (url.includes('work.fife.usercontent.google.com/rd-gg-dl/')) {
          const resp = await origFetch.apply(this, arguments);
          try {
            const text = await resp.clone().text();
            window.__nbFinalURL = (text || '').trim();
            window.__nbFinalURLAt = Date.now();
          } catch (e) {}
          return new Response('', { status: 200, statusText: 'OK', headers: { 'content-type': 'text/plain' } });
        }
        return origFetch.apply(this, arguments);
      };
      return { ok: true };
    })()
    "#
}

fn fetch_intercepted_image_script() -> &'static str {
    r#"
    (async () => {
      const deadline = Date.now() + 30000;
      while (Date.now() < deadline) {
        if (window.__nbFinalURL) break;
        await new Promise(r => setTimeout(r, 300));
      }
      const u = window.__nbFinalURL;
      if (!u) return { ok: false, err: 'no_final_url' };
      try {
        const r = await fetch(u);
        if (!r.ok) return { ok: false, err: 'fetch_failed', status: r.status };
        const blob = await r.blob();
        const buf = await blob.arrayBuffer();
        const u8 = new Uint8Array(buf);
        let s = '';
        const chunk = 32768;
        for (let i = 0; i < u8.length; i += chunk) {
          s += String.fromCharCode.apply(null, u8.subarray(i, i + chunk));
        }
        return { ok: true, contentType: blob.type, size: blob.size, base64: btoa(s) };
      } catch (e) {
        return { ok: false, err: String(e).slice(0, 300) };
      }
    })()
    "#
}

fn png_dimensions(bytes: &[u8]) -> Result<(u32, u32)> {
    let img = image::load_from_memory_with_format(bytes, ImageFormat::Png)
        .map_err(|err| XCliError::GenerateFailed(format!("parse downloaded PNG: {err}")))?;
    Ok(img.dimensions())
}

fn write_thumbnail(bytes: &[u8], path: &std::path::Path, width: u32) -> Result<()> {
    let img = image::load_from_memory_with_format(bytes, ImageFormat::Png)
        .map_err(|err| XCliError::GenerateFailed(format!("decode png: {err}")))?;
    let src_w = img.width();
    let src_h = img.height();
    if src_w == 0 {
        return Err(XCliError::GenerateFailed(
            "source image has zero width".to_string(),
        ));
    }
    let height = (width.saturating_mul(src_h) / src_w).max(1);
    let thumb = img.resize_exact(width, height, FilterType::CatmullRom);
    let mut out = Vec::new();
    thumb
        .write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
        .map_err(|err| XCliError::GenerateFailed(format!("encode thumb: {err}")))?;
    std::fs::write(path, out)
        .map_err(|err| XCliError::GenerateFailed(format!("write thumb: {err}")))
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, path::PathBuf, sync::Mutex, time::Duration};

    use async_trait::async_trait;
    use image::{ImageBuffer, Rgba};
    use serde::de::DeserializeOwned;
    use serde_json::json;
    use xcli_webbridge::BridgeStatus;

    use super::*;

    #[tokio::test]
    async fn gen_writes_full_and_thumb() {
        let png = sample_png();
        let out_dir = unique_temp_dir("success");
        let bridge = MockBridge::new(vec![
            json!(true),
            json!({ "ok": true }),
            json!({ "ok": true }),
            json!(true),
            json!({ "ok": true }),
            json!({ "ok": true }),
            json!({ "ok": true, "contentType": "image/png", "size": png.len(), "base64": STANDARD.encode(&png) }),
        ]);
        let browser = Browser::new(bridge);

        let output = gen(
            &browser,
            GenOptions {
                prompt: "画一朵粉色月季花".to_string(),
                out_dir: out_dir.clone(),
                thumb_width: 16,
                timeout: Duration::from_millis(1),
            },
        )
        .await
        .unwrap();

        assert_eq!(output.prompt, "画一朵粉色月季花");
        assert_eq!(output.width, 32);
        assert_eq!(output.height, 16);
        assert_eq!(output.thumb_width, 16);
        assert!(std::fs::metadata(&output.full).unwrap().len() > 0);
        assert!(std::fs::metadata(&output.thumb).unwrap().len() > 0);

        let _ = std::fs::remove_dir_all(out_dir);
    }

    #[tokio::test]
    async fn gen_rejects_empty_prompt() {
        let out_dir = unique_temp_dir("empty");
        let bridge = MockBridge::new(vec![]);
        let browser = Browser::new(bridge);

        let err = gen(
            &browser,
            GenOptions {
                prompt: " ".to_string(),
                out_dir,
                thumb_width: 256,
                timeout: Duration::from_millis(1),
            },
        )
        .await
        .unwrap_err();

        assert_eq!(err.code(), "invalid_args");
    }

    fn sample_png() -> Vec<u8> {
        let img = ImageBuffer::from_fn(32, 16, |_x, _y| Rgba([255u8, 0, 0, 255]));
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap();
        bytes
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("xcli-nanobanana-{name}-{}", std::process::id()))
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
