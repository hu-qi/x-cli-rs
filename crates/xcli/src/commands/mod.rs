pub mod baidu;
pub mod chatgpt_image;
pub mod google;
pub mod nanobanana;
pub mod xiaohongshu;

use clap::Subcommand;
use serde::Serialize;
use xcli_output::{print_json, JsonResponse};

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(name = "chatgpt-image", aliases = ["image", "img"])]
    ChatgptImage(chatgpt_image::Command),

    #[command(name = "google")]
    Google(google::Command),

    #[command(name = "baidu")]
    Baidu(baidu::Command),

    #[command(name = "nanobanana", aliases = ["nano", "banana"])]
    Nanobanana(nanobanana::Command),

    #[command(name = "xiaohongshu", aliases = ["xhs"])]
    Xiaohongshu(xiaohongshu::Command),
}

pub async fn run(command: Commands) {
    match command {
        Commands::ChatgptImage(command) => emit(chatgpt_image::run(command).await),
        Commands::Google(command) => emit(google::run(command).await),
        Commands::Baidu(command) => emit(baidu::run(command).await),
        Commands::Nanobanana(command) => emit(nanobanana::run(command).await),
        Commands::Xiaohongshu(command) => xiaohongshu::run(command).await,
    }
}

pub fn emit<T>(result: xcli_core::Result<T>)
where
    T: Serialize,
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
