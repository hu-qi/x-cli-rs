use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;
use xcli_browser::Browser;
use xcli_output::{print_json, JsonResponse};
use xcli_webbridge::WebBridgeClient;
use xcli_xiaohongshu::{
    comments, note, profile, search, CommentOptions, CommentsOutput, NoteDetail, NoteOptions,
    ProfileOptions, ProfileOutput, SearchOptions, SearchOutput,
};

const DEFAULT_BRIDGE_URL: &str = "http://127.0.0.1:10086";
const SESSION_NAME: &str = "xiaohongshu-cli";

#[derive(Debug, Parser)]
#[command(name = "xiaohongshu-cli")]
#[command(about = "Automate Xiaohongshu via the kimi-webbridge browser daemon")]
struct Cli {
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Search notes by keyword
    Search(SearchArgs),
    /// Get user profile and published notes
    Profile(ProfileArgs),
    /// Get note detail
    Note(NoteArgs),
    /// Get note comments
    Comments(CommentsArgs),
}

#[derive(Debug, Parser)]
struct SearchArgs {
    keyword: Vec<String>,

    #[arg(short = 'n', long, default_value_t = 10)]
    limit: usize,

    #[arg(long, env = "XCLI_WEBBRIDGE_URL", default_value = DEFAULT_BRIDGE_URL)]
    bridge_url: String,
}

#[derive(Debug, Parser)]
struct ProfileArgs {
    user_id: String,

    #[arg(short = 'n', long, default_value_t = 10)]
    limit: usize,

    #[arg(long, env = "XCLI_WEBBRIDGE_URL", default_value = DEFAULT_BRIDGE_URL)]
    bridge_url: String,
}

#[derive(Debug, Parser)]
struct NoteArgs {
    note_id: String,

    #[arg(long, env = "XCLI_WEBBRIDGE_URL", default_value = DEFAULT_BRIDGE_URL)]
    bridge_url: String,
}

#[derive(Debug, Parser)]
struct CommentsArgs {
    note_id: String,

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
        Commands::Note(args) => emit(run_note(args).await),
        Commands::Comments(args) => emit(run_comments(args).await),
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
    let keyword = args.keyword.join(" ");

    search(
        &browser,
        SearchOptions {
            keyword,
            limit: args.limit,
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
            user_id: args.user_id,
            limit: args.limit,
        },
    )
    .await
}

async fn run_note(args: NoteArgs) -> xcli_core::Result<NoteDetail> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, SESSION_NAME);
    let browser = Browser::new(bridge);

    note(
        &browser,
        NoteOptions {
            note_id: args.note_id,
        },
    )
    .await
}

async fn run_comments(args: CommentsArgs) -> xcli_core::Result<CommentsOutput> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, SESSION_NAME);
    let browser = Browser::new(bridge);

    comments(
        &browser,
        CommentOptions {
            note_id: args.note_id,
            limit: args.limit,
        },
    )
    .await
}
