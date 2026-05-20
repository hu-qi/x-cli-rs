use std::{path::PathBuf, time::Duration};

use clap::{Parser, Subcommand};
use xcli_browser::Browser;
use xcli_chatgpt_image::{generate, GenerateOptions, GenerateOutput};
use xcli_webbridge::WebBridgeClient;

const DEFAULT_BRIDGE_URL: &str = "http://127.0.0.1:10086";
const SESSION_NAME: &str = "chatgpt-image-cli";

#[derive(Debug, Parser)]
pub struct Command {
    #[command(subcommand)]
    command: SubcommandArgs,
}

#[derive(Debug, Subcommand)]
enum SubcommandArgs {
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

pub async fn run(command: Command) -> xcli_core::Result<GenerateOutput> {
    match command.command {
        SubcommandArgs::Generate(args) | SubcommandArgs::Gen(args) => run_generate(args).await,
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
