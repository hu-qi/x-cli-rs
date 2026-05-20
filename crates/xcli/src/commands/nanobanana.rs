use std::{path::PathBuf, time::Duration};

use clap::{Parser, Subcommand};
use xcli_browser::Browser;
use xcli_nanobanana::{gen, GenOptions, GenOutput};
use xcli_webbridge::WebBridgeClient;

const DEFAULT_BRIDGE_URL: &str = "http://127.0.0.1:10086";
const SESSION_NAME: &str = "nanobanana-cli";

#[derive(Debug, Parser)]
pub struct Command {
    #[command(subcommand)]
    command: SubcommandArgs,
}

#[derive(Debug, Subcommand)]
enum SubcommandArgs {
    Gen(GenArgs),
    Generate(GenArgs),
}

#[derive(Debug, Parser)]
struct GenArgs {
    prompt: String,

    #[arg(short, long, default_value = ".")]
    out: PathBuf,

    #[arg(long, default_value_t = 256)]
    thumb_width: u32,

    #[arg(long, default_value_t = 300)]
    timeout: u64,

    #[arg(long, env = "XCLI_WEBBRIDGE_URL", default_value = DEFAULT_BRIDGE_URL)]
    bridge_url: String,
}

pub async fn run(command: Command) -> xcli_core::Result<GenOutput> {
    match command.command {
        SubcommandArgs::Gen(args) | SubcommandArgs::Generate(args) => run_gen(args).await,
    }
}

async fn run_gen(args: GenArgs) -> xcli_core::Result<GenOutput> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, SESSION_NAME);
    let browser = Browser::new(bridge);

    gen(
        &browser,
        GenOptions {
            prompt: args.prompt,
            out_dir: args.out,
            thumb_width: args.thumb_width,
            timeout: Duration::from_secs(args.timeout),
        },
    )
    .await
}
