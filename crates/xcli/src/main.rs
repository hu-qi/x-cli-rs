use std::{path::PathBuf, time::Duration};

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;
use xcli_browser::Browser;
use xcli_chatgpt_image::{generate, GenerateOptions, GenerateOutput};
use xcli_google::{search as google_search, SearchOptions, SearchResult};
use xcli_output::{print_json, JsonResponse};
use xcli_webbridge::WebBridgeClient;

const DEFAULT_BRIDGE_URL: &str = "http://127.0.0.1:10086";
const CHATGPT_IMAGE_SESSION: &str = "chatgpt-image-cli";
const GOOGLE_SESSION: &str = "google-cli";

#[derive(Debug, Parser)]
#[command(name = "x")]
#[command(about = "Rust implementation of browser-agent CLI tools")]
struct Cli {
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(name = "chatgpt-image", aliases = ["image", "img"])]
    ChatgptImage(ChatgptImageCommand),

    #[command(name = "google")]
    Google(GoogleCommand),
}

#[derive(Debug, Parser)]
struct ChatgptImageCommand {
    #[command(subcommand)]
    command: ChatgptImageSubcommand,
}

#[derive(Debug, Subcommand)]
enum ChatgptImageSubcommand {
    Generate(GenerateArgs),
    #[command(alias = "g")]
    Gen(GenerateArgs),
}

#[derive(Debug, Parser)]
struct GoogleCommand {
    #[command(subcommand)]
    command: GoogleSubcommand,
}

#[derive(Debug, Subcommand)]
enum GoogleSubcommand {
    Search(GoogleSearchArgs),
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

#[derive(Debug, Parser)]
struct GoogleSearchArgs {
    query: Vec<String>,

    #[arg(long, default_value_t = 10)]
    limit: usize,

    #[arg(long, default_value = "en")]
    hl: String,

    #[arg(long, env = "XCLI_WEBBRIDGE_URL", default_value = DEFAULT_BRIDGE_URL)]
    bridge_url: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    match cli.command {
        Commands::ChatgptImage(command) => emit(run_chatgpt_image(command).await),
        Commands::Google(command) => emit(run_google(command).await),
    }
}

fn emit<T>(result: xcli_core::Result<T>)
where
    T: serde::Serialize,
{
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

fn init_tracing(verbose: bool) {
    if !verbose {
        return;
    }

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .without_time()
        .try_init();
}

async fn run_chatgpt_image(command: ChatgptImageCommand) -> xcli_core::Result<GenerateOutput> {
    match command.command {
        ChatgptImageSubcommand::Generate(args) | ChatgptImageSubcommand::Gen(args) => {
            run_chatgpt_image_generate(args).await
        }
    }
}

async fn run_chatgpt_image_generate(args: GenerateArgs) -> xcli_core::Result<GenerateOutput> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, CHATGPT_IMAGE_SESSION);
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

async fn run_google(command: GoogleCommand) -> xcli_core::Result<Vec<SearchResult>> {
    match command.command {
        GoogleSubcommand::Search(args) => run_google_search(args).await,
    }
}

async fn run_google_search(args: GoogleSearchArgs) -> xcli_core::Result<Vec<SearchResult>> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, GOOGLE_SESSION);
    let browser = Browser::new(bridge);
    let query = args.query.join(" ");

    google_search(
        &browser,
        SearchOptions {
            query,
            limit: args.limit,
            hl: args.hl,
        },
    )
    .await
}
