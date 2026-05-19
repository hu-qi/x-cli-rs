# Rust Layout

Every site integration has two layers:

```text
crates/xcli-<site>/      reusable library crate
examples/<site>-cli/     compatibility binary
```

The unified `x` binary depends on the reusable crate and exposes `x <site> ...`.

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

`src/lib.rs` should expose typed options and output structs:

```rust
#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub query: String,
    pub limit: usize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}
```

Public functions should be generic over `BrowserBridge`:

```rust
pub async fn search<B>(browser: &Browser<B>, options: SearchOptions) -> xcli_core::Result<Vec<SearchResult>>
where
    B: xcli_webbridge::BrowserBridge,
{
    // site logic here
}
```

## Compatibility binary

The compatibility binary should not contain site logic.

It should only:

1. Parse CLI args.
2. Initialize tracing when `--verbose` is set.
3. Create `WebBridgeClient::with_session`.
4. Call the reusable crate.
5. Print `JsonResponse`.
6. Exit `1` on errors.

## Unified x entrypoint

Update:

```text
crates/xcli/Cargo.toml
crates/xcli/src/main.rs
```

Add the dependency:

```toml
xcli-<site> = { path = "../xcli-<site>" }
```

Add a subcommand:

```rust
#[derive(Debug, Subcommand)]
enum Commands {
    #[command(name = "<site>")]
    Site(SiteCommand),
}
```

Keep command aliases conservative. Avoid duplicate aliases such as assigning `alias = "generate"` to a variant already named `Generate`.
