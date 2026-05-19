use clap::{Parser, Subcommand};
use xcli_browser::Browser;
use xcli_webbridge::WebBridgeClient;
use xcli_xiaohongshu::{
    comments, note, profile, search, CommentOptions, CommentsOutput, NoteDetail, NoteOptions,
    ProfileOptions, ProfileOutput, SearchOptions, SearchOutput,
};

use super::emit;

const DEFAULT_BRIDGE_URL: &str = "http://127.0.0.1:10086";
const SESSION_NAME: &str = "xiaohongshu-cli";

#[derive(Debug, Parser)]
pub struct Command {
    #[command(subcommand)]
    command: SubcommandArgs,
}

#[derive(Debug, Subcommand)]
enum SubcommandArgs {
    Search(SearchArgs),
    Profile(ProfileArgs),
    Note(NoteArgs),
    Comments(CommentsArgs),
}

#[derive(Debug, Parser)]
struct SearchArgs {
    query: Vec<String>,

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

pub async fn run(command: Command) {
    match command.command {
        SubcommandArgs::Search(args) => emit(run_search(args).await),
        SubcommandArgs::Profile(args) => emit(run_profile(args).await),
        SubcommandArgs::Note(args) => emit(run_note(args).await),
        SubcommandArgs::Comments(args) => emit(run_comments(args).await),
    }
}

async fn run_search(args: SearchArgs) -> xcli_core::Result<SearchOutput> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, SESSION_NAME);
    let browser = Browser::new(bridge);

    search(
        &browser,
        SearchOptions {
            keyword: args.query.join(" "),
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
