use std::{path::PathBuf, time::Duration};

use clap::{Parser, Subcommand};
use xcli_browser::Browser;
use xcli_chatgpt_image::{generate, GenerateOptions, GenerateOutput};
use xcli_output::{print_json, JsonResponse};
use xcli_webbridge::WebBridgeClient;

const DEFAULT_BRIDGE_URL: &str = "http://127.0.0.1:10086";
const SESSION_NAME: &str = "chatgpt-image-cli";

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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Generate(args) | Commands::Gen(args) => run_generate(args).await,
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

async fn run_generate(args: GenerateArgs) -> xcli_core::Result<GenerateOutput> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, SESSION_NAME);
    let browser = Browser::new(bridge);

    generate(
        &browser,
        GenerateOptions {
            prompt: args.prompt,
            out_dir: args.out,
            timeout: Duration::from_secs(args.timeout),
        },
    )
    .await
}
