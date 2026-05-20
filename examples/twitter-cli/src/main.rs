use std::{path::PathBuf, time::Duration};

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;
use xcli_browser::Browser;
use xcli_output::{print_json, JsonResponse};
use xcli_twitter::{
    post, profile, replies, search, PostDetail, PostOptions, ProfileOptions, ProfileOutput,
    RepliesOptions, RepliesOutput, SearchOptions, SearchOutput,
};
use xcli_webbridge::WebBridgeClient;

const DEFAULT_BRIDGE_URL: &str = "http://127.0.0.1:10086";
const SESSION_NAME: &str = "twitter-cli";

#[derive(Debug, Parser)]
#[command(name = "twitter-cli")]
#[command(about = "Automate x.com (Twitter) via the kimi-webbridge browser daemon")]
struct Cli {
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Search tweets by keyword
    Search(SearchArgs),
    /// Get user profile and recent tweets
    Profile(ProfileArgs),
    /// Get a single post (tweet) detail with images, videos, and links
    Post(PostArgs),
    /// Get replies to a post (comments)
    Replies(RepliesArgs),
}

#[derive(Debug, Parser)]
struct SearchArgs {
    query: Vec<String>,

    #[arg(short = 'n', long, default_value_t = 20)]
    limit: usize,

    /// Sort/filter mode: top (default), live, user, image, video
    #[arg(long, default_value = "top")]
    mode: String,

    #[arg(long, env = "XCLI_WEBBRIDGE_URL", default_value = DEFAULT_BRIDGE_URL)]
    bridge_url: String,
}

#[derive(Debug, Parser)]
struct ProfileArgs {
    /// Twitter handle, with or without leading `@`
    handle: String,

    #[arg(short = 'n', long, default_value_t = 20)]
    limit: usize,

    #[arg(long, env = "XCLI_WEBBRIDGE_URL", default_value = DEFAULT_BRIDGE_URL)]
    bridge_url: String,
}

#[derive(Debug, Parser)]
struct PostArgs {
    /// Tweet reference: full URL, `<user>/status/<id>`, `<user>/<id>`, or bare `<id>`
    reference: String,

    /// Optional output directory; when set, the post's images and videos are
    /// downloaded directly from the Twitter CDN.
    #[arg(short, long)]
    out: Option<PathBuf>,

    /// Milliseconds to wait between asset downloads (default 250ms).
    #[arg(long, default_value_t = 250)]
    throttle_ms: u64,

    #[arg(long, env = "XCLI_WEBBRIDGE_URL", default_value = DEFAULT_BRIDGE_URL)]
    bridge_url: String,
}

#[derive(Debug, Parser)]
struct RepliesArgs {
    reference: String,

    #[arg(short = 'n', long, default_value_t = 20)]
    limit: usize,

    #[arg(long, env = "XCLI_WEBBRIDGE_URL", default_value = DEFAULT_BRIDGE_URL)]
    bridge_url: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    match cli.command {
        Commands::Search(args) => emit(run_search(args).await),
        Commands::Profile(args) => emit(run_profile(args).await),
        Commands::Post(args) => emit(run_post(args).await),
        Commands::Replies(args) => emit(run_replies(args).await),
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

async fn run_search(args: SearchArgs) -> xcli_core::Result<SearchOutput> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, SESSION_NAME);
    let browser = Browser::new(bridge);
    let query = args.query.join(" ");

    search(
        &browser,
        SearchOptions {
            query,
            limit: args.limit,
            mode: args.mode,
        },
    )
    .await
}

async fn run_profile(args: ProfileArgs) -> xcli_core::Result<ProfileOutput> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, SESSION_NAME);
    let browser = Browser::new(bridge);

    profile(
        &browser,
        ProfileOptions {
            handle: args.handle,
            limit: args.limit,
        },
    )
    .await
}

async fn run_post(args: PostArgs) -> xcli_core::Result<PostDetail> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, SESSION_NAME);
    let browser = Browser::new(bridge);

    post(
        &browser,
        PostOptions {
            reference: args.reference,
            out_dir: args.out,
            throttle: Duration::from_millis(args.throttle_ms),
        },
    )
    .await
}

async fn run_replies(args: RepliesArgs) -> xcli_core::Result<RepliesOutput> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, SESSION_NAME);
    let browser = Browser::new(bridge);

    replies(
        &browser,
        RepliesOptions {
            reference: args.reference,
            limit: args.limit,
        },
    )
    .await
}
