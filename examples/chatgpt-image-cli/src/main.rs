use std::{path::PathBuf, time::Instant};

use base64::{engine::general_purpose::STANDARD, Engine};
use clap::{Parser, Subcommand};
use serde::Serialize;
use time::{format_description::FormatItem, macros::format_description, OffsetDateTime};
use xcli_browser::Browser;
use xcli_core::{Result, XCliError};
use xcli_output::{print_json, JsonResponse};
use xcli_webbridge::WebBridgeClient;

const DEFAULT_BRIDGE_URL: &str = "http://127.0.0.1:14588";
const CHATGPT_IMAGES_URL: &str = "https://chatgpt.com/images/";
const FILE_TS_FORMAT: &[FormatItem<'_>] = format_description!("[year][month][day]-[hour][minute][second]");

#[derive(Debug, Parser)]
#[command(name = "chatgpt-image-cli")]
#[command(about = "Generate images through the ChatGPT web UI using kimi-webbridge")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Generate(GenerateArgs),
    #[command(alias = "g")]
    Gen(GenerateArgs),
}

#[derive(Debug, Parser)]
struct GenerateArgs {
    prompt: String,

    #[arg(short, long, default_value = ".")]
    out: PathBuf,

    #[arg(long, default_value_t = 180)]
    timeout: u64,

    #[arg(long, env = "XCLI_WEBBRIDGE_URL", default_value = DEFAULT_BRIDGE_URL)]
    bridge_url: String,
}

#[derive(Debug, Serialize)]
struct GenerateOutput {
    prompt: String,
    path: String,
    bytes: u64,
    caption: Option<String>,
    conversation_url: Option<String>,
    elapsed_ms: u128,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Generate(args) | Commands::Gen(args) => generate(args).await,
    };

    match result {
        Ok(data) => {
            let _ = print_json(&JsonResponse::ok(data));
        }
        Err(err) => {
            let _ = print_json(&JsonResponse::<()>::error(err.code(), err.to_string()));
            std::process::exit(1);
        }
    }
}

async fn generate(args: GenerateArgs) -> Result<GenerateOutput> {
    if args.prompt.trim().is_empty() {
        return Err(XCliError::InvalidArgs("prompt must not be empty".to_string()));
    }

    std::fs::create_dir_all(&args.out).map_err(|err| XCliError::InvalidArgs(err.to_string()))?;

    let started = Instant::now();
    let bridge = WebBridgeClient::new(args.bridge_url);
    let browser = Browser::new(bridge);

    browser.ensure_ready().await?;
    browser.goto(CHATGPT_IMAGES_URL).await?;
    browser.insert_text("#prompt-textarea", &args.prompt).await?;
    browser.click("#composer-submit-button").await?;

    let timeout = std::time::Duration::from_secs(args.timeout);
    browser
        .wait_for_js_truthy("location.href.includes('/c/')", timeout)
        .await?;

    browser
        .wait_for_js_truthy(
            "Boolean(document.querySelector(\"main img[src*='/backend-api/estuary/content']\"))",
            timeout,
        )
        .await?;

    let image_meta: ImageMeta = browser
        .eval(
            r#"
            (() => {
              const img = document.querySelector("main img[src*='/backend-api/estuary/content']");
              return {
                src: img?.src ?? null,
                alt: img?.alt || null,
                url: location.href
              };
            })()
            "#,
        )
        .await?;

    let src = image_meta
        .src
        .ok_or_else(|| XCliError::GenerateFailed("image src not found".to_string()))?;

    let bytes_b64: String = browser
        .eval(&format!(
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
        ))
        .await?;

    let bytes = STANDARD
        .decode(bytes_b64)
        .map_err(|err| XCliError::GenerateFailed(err.to_string()))?;
    let timestamp = OffsetDateTime::now_utc()
        .format(FILE_TS_FORMAT)
        .map_err(|err| XCliError::GenerateFailed(err.to_string()))?;
    let path = args.out.join(format!("chatgpt-{timestamp}.png"));
    std::fs::write(&path, &bytes).map_err(|err| XCliError::GenerateFailed(err.to_string()))?;

    Ok(GenerateOutput {
        prompt: args.prompt,
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

#[derive(Debug, serde::Deserialize)]
struct ImageMeta {
    src: Option<String>,
    alt: Option<String>,
    url: Option<String>,
}
