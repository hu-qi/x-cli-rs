use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;
use xcli_baidu::{search as baidu_search, SearchOptions, SearchOutput};
use xcli_browser::Browser;
use xcli_output::{print_json, JsonResponse};
use xcli_webbridge::WebBridgeClient;

const DEFAULT_BRIDGE_URL: &str = "http://127.0.0.1:10086";
const SESSION_NAME: &str = "baidu";

#[derive(Debug, Parser)]
#[command(name = "baidu-cli")]
#[command(about = "Automate Baidu Search via the kimi-webbridge daemon")]
struct Cli {
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Search(SearchArgs),
}

#[derive(Debug, Parser)]
struct SearchArgs {
    query: Vec<String>,

    #[arg(short = 'n', long, default_value_t = 10)]
    limit: usize,

    #[arg(long)]
    all: bool,

    #[arg(long, env = "XCLI_WEBBRIDGE_URL", default_value = DEFAULT_BRIDGE_URL)]
    bridge_url: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let result = match cli.command {
        Commands::Search(args) => run_search(args).await,
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

async fn run_search(args: SearchArgs) -> xcli_core::Result<SearchOutput> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, SESSION_NAME);
    let browser = Browser::new(bridge);
    let query = args.query.join(" ");

    baidu_search(
        &browser,
        SearchOptions {
            query,
            limit: args.limit,
            include_all: args.all,
        },
    )
    .await
}
