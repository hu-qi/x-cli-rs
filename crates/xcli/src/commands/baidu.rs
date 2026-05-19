use clap::{Parser, Subcommand};
use xcli_baidu::{search, SearchOptions, SearchOutput};
use xcli_browser::Browser;
use xcli_webbridge::WebBridgeClient;

const DEFAULT_BRIDGE_URL: &str = "http://127.0.0.1:10086";
const SESSION_NAME: &str = "baidu";

#[derive(Debug, Parser)]
pub struct Command {
    #[command(subcommand)]
    command: SubcommandArgs,
}

#[derive(Debug, Subcommand)]
enum SubcommandArgs {
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

pub async fn run(command: Command) -> xcli_core::Result<SearchOutput> {
    match command.command {
        SubcommandArgs::Search(args) => run_search(args).await,
    }
}

async fn run_search(args: SearchArgs) -> xcli_core::Result<SearchOutput> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, SESSION_NAME);
    let browser = Browser::new(bridge);

    search(
        &browser,
        SearchOptions {
            query: args.query.join(" "),
            limit: args.limit,
            include_all: args.all,
        },
    )
    .await
}
