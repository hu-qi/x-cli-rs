use std::{path::PathBuf, time::Duration};

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;
use xcli_baidu::{
    search as baidu_search, SearchOptions as BaiduSearchOptions, SearchOutput as BaiduSearchOutput,
};
use xcli_browser::Browser;
use xcli_chatgpt_image::{generate, GenerateOptions, GenerateOutput};
use xcli_google::{search as google_search, SearchOptions as GoogleSearchOptions, SearchResult};
use xcli_nanobanana::{
    gen as nanobanana_gen, GenOptions as NanobananaGenOptions, GenOutput as NanobananaGenOutput,
};
use xcli_output::{print_json, JsonResponse};
use xcli_twitter::{
    post as twitter_post, profile as twitter_profile, replies as twitter_replies,
    search as twitter_search, PostDetail, PostOptions as TwitterPostOptions,
    ProfileOptions as TwitterProfileOptions, ProfileOutput as TwitterProfileOutput,
    RepliesOptions as TwitterRepliesOptions, RepliesOutput, SearchOptions as TwitterSearchOptions,
    SearchOutput as TwitterSearchOutput,
};
use xcli_webbridge::WebBridgeClient;
use xcli_xiaohongshu::{
    comments as xhs_comments, note as xhs_note, profile as xhs_profile, search as xhs_search,
    CommentOptions as XhsCommentOptions, CommentsOutput, NoteDetail, NoteOptions as XhsNoteOptions,
    ProfileOptions as XhsProfileOptions, ProfileOutput, SearchOptions as XhsSearchOptions,
    SearchOutput as XhsSearchOutput,
};

const DEFAULT_BRIDGE_URL: &str = "http://127.0.0.1:10086";
const CHATGPT_IMAGE_SESSION: &str = "chatgpt-image-cli";
const GOOGLE_SESSION: &str = "google-cli";
const BAIDU_SESSION: &str = "baidu";
const NANOBANANA_SESSION: &str = "nanobanana-cli";
const XIAOHONGSHU_SESSION: &str = "xiaohongshu-cli";
const TWITTER_SESSION: &str = "twitter-cli";

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

    #[command(name = "baidu")]
    Baidu(BaiduCommand),

    #[command(name = "nanobanana", aliases = ["nano", "banana"])]
    Nanobanana(NanobananaCommand),

    #[command(name = "xiaohongshu", aliases = ["xhs"])]
    Xiaohongshu(XiaohongshuCommand),

    #[command(name = "twitter", aliases = ["tw", "x.com"])]
    Twitter(TwitterCommand),
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
struct BaiduCommand {
    #[command(subcommand)]
    command: BaiduSubcommand,
}

#[derive(Debug, Subcommand)]
enum BaiduSubcommand {
    Search(BaiduSearchArgs),
}

#[derive(Debug, Parser)]
struct NanobananaCommand {
    #[command(subcommand)]
    command: NanobananaSubcommand,
}

#[derive(Debug, Subcommand)]
enum NanobananaSubcommand {
    Gen(NanobananaGenArgs),
    #[command(alias = "generate")]
    Generate(NanobananaGenArgs),
}

#[derive(Debug, Parser)]
struct TwitterCommand {
    #[command(subcommand)]
    command: TwitterSubcommand,
}

#[derive(Debug, Subcommand)]
enum TwitterSubcommand {
    Search(TwitterSearchArgs),
    Profile(TwitterProfileArgs),
    Post(TwitterPostArgs),
    Replies(TwitterRepliesArgs),
}

#[derive(Debug, Parser)]
struct TwitterSearchArgs {
    query: Vec<String>,

    #[arg(short = 'n', long, default_value_t = 20)]
    limit: usize,

    #[arg(long, default_value = "top")]
    mode: String,

    #[arg(long, env = "XCLI_WEBBRIDGE_URL", default_value = DEFAULT_BRIDGE_URL)]
    bridge_url: String,
}

#[derive(Debug, Parser)]
struct TwitterProfileArgs {
    handle: String,

    #[arg(short = 'n', long, default_value_t = 20)]
    limit: usize,

    #[arg(long, env = "XCLI_WEBBRIDGE_URL", default_value = DEFAULT_BRIDGE_URL)]
    bridge_url: String,
}

#[derive(Debug, Parser)]
struct TwitterPostArgs {
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
struct TwitterRepliesArgs {
    reference: String,

    #[arg(short = 'n', long, default_value_t = 20)]
    limit: usize,

    #[arg(long, env = "XCLI_WEBBRIDGE_URL", default_value = DEFAULT_BRIDGE_URL)]
    bridge_url: String,
}

#[derive(Debug, Parser)]
struct XiaohongshuCommand {
    #[command(subcommand)]
    command: XiaohongshuSubcommand,
}

#[derive(Debug, Subcommand)]
enum XiaohongshuSubcommand {
    Search(XiaohongshuSearchArgs),
    Profile(XiaohongshuProfileArgs),
    Note(XiaohongshuNoteArgs),
    Comments(XiaohongshuCommentsArgs),
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

#[derive(Debug, Parser)]
struct BaiduSearchArgs {
    query: Vec<String>,

    #[arg(short = 'n', long, default_value_t = 10)]
    limit: usize,

    #[arg(long)]
    all: bool,

    #[arg(long, env = "XCLI_WEBBRIDGE_URL", default_value = DEFAULT_BRIDGE_URL)]
    bridge_url: String,
}

#[derive(Debug, Parser)]
struct NanobananaGenArgs {
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

#[derive(Debug, Parser)]
struct XiaohongshuSearchArgs {
    query: Vec<String>,

    #[arg(short = 'n', long, default_value_t = 10)]
    limit: usize,

    #[arg(long, env = "XCLI_WEBBRIDGE_URL", default_value = DEFAULT_BRIDGE_URL)]
    bridge_url: String,
}

#[derive(Debug, Parser)]
struct XiaohongshuProfileArgs {
    user_id: String,

    #[arg(short = 'n', long, default_value_t = 10)]
    limit: usize,

    #[arg(long, env = "XCLI_WEBBRIDGE_URL", default_value = DEFAULT_BRIDGE_URL)]
    bridge_url: String,
}

#[derive(Debug, Parser)]
struct XiaohongshuNoteArgs {
    note_id: String,

    #[arg(long, env = "XCLI_WEBBRIDGE_URL", default_value = DEFAULT_BRIDGE_URL)]
    bridge_url: String,
}

#[derive(Debug, Parser)]
struct XiaohongshuCommentsArgs {
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
        Commands::ChatgptImage(command) => emit(run_chatgpt_image(command).await),
        Commands::Google(command) => emit(run_google(command).await),
        Commands::Baidu(command) => emit(run_baidu(command).await),
        Commands::Nanobanana(command) => emit(run_nanobanana(command).await),
        Commands::Xiaohongshu(command) => match command.command {
            XiaohongshuSubcommand::Search(args) => emit(run_xiaohongshu_search(args).await),
            XiaohongshuSubcommand::Profile(args) => emit(run_xiaohongshu_profile(args).await),
            XiaohongshuSubcommand::Note(args) => emit(run_xiaohongshu_note(args).await),
            XiaohongshuSubcommand::Comments(args) => emit(run_xiaohongshu_comments(args).await),
        },
        Commands::Twitter(command) => match command.command {
            TwitterSubcommand::Search(args) => emit(run_twitter_search(args).await),
            TwitterSubcommand::Profile(args) => emit(run_twitter_profile(args).await),
            TwitterSubcommand::Post(args) => emit(run_twitter_post(args).await),
            TwitterSubcommand::Replies(args) => emit(run_twitter_replies(args).await),
        },
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
        GoogleSearchOptions {
            query,
            limit: args.limit,
            hl: args.hl,
        },
    )
    .await
}

async fn run_baidu(command: BaiduCommand) -> xcli_core::Result<BaiduSearchOutput> {
    match command.command {
        BaiduSubcommand::Search(args) => run_baidu_search(args).await,
    }
}

async fn run_baidu_search(args: BaiduSearchArgs) -> xcli_core::Result<BaiduSearchOutput> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, BAIDU_SESSION);
    let browser = Browser::new(bridge);
    let query = args.query.join(" ");

    baidu_search(
        &browser,
        BaiduSearchOptions {
            query,
            limit: args.limit,
            include_all: args.all,
        },
    )
    .await
}

async fn run_nanobanana(command: NanobananaCommand) -> xcli_core::Result<NanobananaGenOutput> {
    match command.command {
        NanobananaSubcommand::Gen(args) | NanobananaSubcommand::Generate(args) => {
            run_nanobanana_gen(args).await
        }
    }
}

async fn run_nanobanana_gen(args: NanobananaGenArgs) -> xcli_core::Result<NanobananaGenOutput> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, NANOBANANA_SESSION);
    let browser = Browser::new(bridge);

    nanobanana_gen(
        &browser,
        NanobananaGenOptions {
            prompt: args.prompt,
            out_dir: args.out,
            thumb_width: args.thumb_width,
            timeout: Duration::from_secs(args.timeout),
        },
    )
    .await
}

async fn run_xiaohongshu_search(args: XiaohongshuSearchArgs) -> xcli_core::Result<XhsSearchOutput> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, XIAOHONGSHU_SESSION);
    let browser = Browser::new(bridge);
    let keyword = args.query.join(" ");

    xhs_search(
        &browser,
        XhsSearchOptions {
            keyword,
            limit: args.limit,
        },
    )
    .await
}

async fn run_xiaohongshu_profile(args: XiaohongshuProfileArgs) -> xcli_core::Result<ProfileOutput> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, XIAOHONGSHU_SESSION);
    let browser = Browser::new(bridge);

    xhs_profile(
        &browser,
        XhsProfileOptions {
            user_id: args.user_id,
            limit: args.limit,
        },
    )
    .await
}

async fn run_xiaohongshu_note(args: XiaohongshuNoteArgs) -> xcli_core::Result<NoteDetail> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, XIAOHONGSHU_SESSION);
    let browser = Browser::new(bridge);

    xhs_note(
        &browser,
        XhsNoteOptions {
            note_id: args.note_id,
        },
    )
    .await
}

async fn run_xiaohongshu_comments(
    args: XiaohongshuCommentsArgs,
) -> xcli_core::Result<CommentsOutput> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, XIAOHONGSHU_SESSION);
    let browser = Browser::new(bridge);

    xhs_comments(
        &browser,
        XhsCommentOptions {
            note_id: args.note_id,
            limit: args.limit,
        },
    )
    .await
}

async fn run_twitter_search(args: TwitterSearchArgs) -> xcli_core::Result<TwitterSearchOutput> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, TWITTER_SESSION);
    let browser = Browser::new(bridge);
    let query = args.query.join(" ");

    twitter_search(
        &browser,
        TwitterSearchOptions {
            query,
            limit: args.limit,
            mode: args.mode,
        },
    )
    .await
}

async fn run_twitter_profile(args: TwitterProfileArgs) -> xcli_core::Result<TwitterProfileOutput> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, TWITTER_SESSION);
    let browser = Browser::new(bridge);

    twitter_profile(
        &browser,
        TwitterProfileOptions {
            handle: args.handle,
            limit: args.limit,
        },
    )
    .await
}

async fn run_twitter_post(args: TwitterPostArgs) -> xcli_core::Result<PostDetail> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, TWITTER_SESSION);
    let browser = Browser::new(bridge);

    twitter_post(
        &browser,
        TwitterPostOptions {
            reference: args.reference,
            out_dir: args.out,
            throttle: Duration::from_millis(args.throttle_ms),
        },
    )
    .await
}

async fn run_twitter_replies(args: TwitterRepliesArgs) -> xcli_core::Result<RepliesOutput> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, TWITTER_SESSION);
    let browser = Browser::new(bridge);

    twitter_replies(
        &browser,
        TwitterRepliesOptions {
            reference: args.reference,
            limit: args.limit,
        },
    )
    .await
}
