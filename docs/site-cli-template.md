# Site CLI Template

Copy this template when adding a new browser-backed CLI.

Replace:

```text
<site>
<Site>
<command>
```

## Workspace entries

Root `Cargo.toml`:

```toml
[workspace]
members = [
  "crates/xcli-<site>",
  "examples/<site>-cli",
]
```

## Manifest entry

Add the shipped binary to `xcli.manifest.toml`:

```toml
[[binaries]]
name = "<site>-cli"
package = "<site>-cli"
site = "<site>"
aliases = ["<optional-short-alias>"]
smoke = 'cargo run -p xcli -- --verbose <site> <command> "sample" --limit 5'
```

Then run:

```bash
cargo run -p xtask -- check
```

## Library crate

`crates/xcli-<site>/Cargo.toml`:

```toml
[package]
name = "xcli-<site>"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true
rust-version.workspace = true

[dependencies]
serde.workspace = true
tracing.workspace = true
xcli-browser = { path = "../xcli-browser" }
xcli-core = { path = "../xcli-core" }
xcli-webbridge = { path = "../xcli-webbridge" }

[dev-dependencies]
async-trait.workspace = true
serde_json.workspace = true
tokio.workspace = true
```

`crates/xcli-<site>/src/lib.rs`:

```rust
use serde::{Deserialize, Serialize};
use tracing::info;
use xcli_browser::Browser;
use xcli_core::{Result, XCliError};
use xcli_webbridge::BrowserBridge;

#[derive(Debug, Clone)]
pub struct CommandOptions {
    pub query: String,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandOutput {
    pub title: String,
    pub url: String,
}

pub async fn run_command<B>(browser: &Browser<B>, options: CommandOptions) -> Result<Vec<CommandOutput>>
where
    B: BrowserBridge,
{
    if options.query.trim().is_empty() {
        return Err(XCliError::MissingArgs(
            "<command> requires a query: <site>-cli <command> <query>".to_string(),
        ));
    }

    let url = format!("https://example.com/search?q={}", options.query);

    info!(step = "navigate", url = %url, "opening <Site>");
    browser.goto(&url).await.map_err(map_error)?;

    info!(step = "extract", "extracting <Site> results");
    browser.eval(extract_script()).await.map_err(map_error)
}

fn map_error(err: XCliError) -> XCliError {
    match err {
        XCliError::DaemonUnreachable(_)
        | XCliError::DaemonNotRunning
        | XCliError::ExtensionNotConnected => err,
        other => XCliError::SearchFailed(other.to_string()),
    }
}

fn extract_script() -> &'static str {
    r#"
    (() => {
      return Array.from(document.querySelectorAll('a')).slice(0, 10).map(a => ({
        title: a.innerText.trim(),
        url: a.href
      }));
    })()
    "#
}
```

## Compatibility binary

`examples/<site>-cli/Cargo.toml`:

```toml
[package]
name = "<site>-cli"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true
rust-version.workspace = true

[dependencies]
clap.workspace = true
serde.workspace = true
tokio.workspace = true
tracing-subscriber.workspace = true
xcli-browser = { path = "../../crates/xcli-browser" }
xcli-core = { path = "../../crates/xcli-core" }
xcli-<site> = { path = "../../crates/xcli-<site>" }
xcli-output = { path = "../../crates/xcli-output" }
xcli-webbridge = { path = "../../crates/xcli-webbridge" }
```

`examples/<site>-cli/src/main.rs`:

```rust
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;
use xcli_browser::Browser;
use xcli_<site>::{run_command, CommandOptions, CommandOutput};
use xcli_output::{print_json, JsonResponse};
use xcli_webbridge::WebBridgeClient;

const DEFAULT_BRIDGE_URL: &str = "http://127.0.0.1:10086";
const SESSION_NAME: &str = "<site>-cli";

#[derive(Debug, Parser)]
#[command(name = "<site>-cli", version)]
#[command(about = "Automate <Site> via kimi-webbridge")]
struct Cli {
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Command(CommandArgs),
}

#[derive(Debug, Parser)]
struct CommandArgs {
    query: Vec<String>,

    #[arg(long, default_value_t = 10)]
    limit: usize,

    #[arg(long, env = "XCLI_WEBBRIDGE_URL", default_value = DEFAULT_BRIDGE_URL)]
    bridge_url: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let result = match cli.command {
        Commands::Command(args) => run(args).await,
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

async fn run(args: CommandArgs) -> xcli_core::Result<Vec<CommandOutput>> {
    let bridge = WebBridgeClient::with_session(args.bridge_url, SESSION_NAME);
    let browser = Browser::new(bridge);

    run_command(
        &browser,
        CommandOptions {
            query: args.query.join(" "),
            limit: args.limit,
        },
    )
    .await
}
```

## Unified `x` entrypoint patch checklist

`crates/xcli/Cargo.toml`:

```toml
xcli-<site> = { path = "../xcli-<site>" }
```

Add a module:

```text
crates/xcli/src/commands/<site>.rs
```

Register it in:

```text
crates/xcli/src/commands/mod.rs
```

The site module should own session constants, clap args, subcommands, and conversion into the reusable library crate options.

## Release integration patch checklist

Add `<site>-cli` to `xcli.manifest.toml`, then run:

```bash
cargo run -p xtask -- check
```

The manifest check covers these files:

```text
Cargo.toml
Makefile
.github/workflows/release.yml
install.sh
install.ps1
README.md
README-zh.md
docs/release-checklist.md
```

## Test template

Add mock tests in `crates/xcli-<site>/src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, sync::Mutex};

    use async_trait::async_trait;
    use serde::de::DeserializeOwned;
    use serde_json::json;
    use xcli_webbridge::BridgeStatus;

    use super::*;

    #[tokio::test]
    async fn command_rejects_empty_query() {
        let bridge = MockBridge::new(vec![]);
        let browser = Browser::new(bridge);

        let err = run_command(
            &browser,
            CommandOptions {
                query: " ".to_string(),
                limit: 10,
            },
        )
        .await
        .unwrap_err();

        assert_eq!(err.code(), "missing_args");
    }

    struct MockBridge {
        values: Mutex<VecDeque<serde_json::Value>>,
    }

    impl MockBridge {
        fn new(values: Vec<serde_json::Value>) -> Self {
            Self {
                values: Mutex::new(values.into()),
            }
        }
    }

    #[async_trait]
    impl BrowserBridge for MockBridge {
        async fn status(&self) -> Result<BridgeStatus> {
            Ok(BridgeStatus {
                running: true,
                extension_connected: true,
                extension_version: Some("test".to_string()),
                version: Some("test".to_string()),
            })
        }

        async fn navigate(&self, _url: &str) -> Result<()> {
            Ok(())
        }

        async fn eval<T>(&self, _javascript: &str) -> Result<T>
        where
            T: DeserializeOwned + Send,
        {
            let value = self
                .values
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| XCliError::BrowserActionFailed("mock exhausted".to_string()))?;
            serde_json::from_value(value)
                .map_err(|err| XCliError::BrowserActionFailed(err.to_string()))
        }
    }
}
```
