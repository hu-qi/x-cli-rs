# x-cli-rs

Rust implementation of browser-agent CLI examples inspired by [`better-world-ai/x-cli`](https://github.com/better-world-ai/x-cli).

The project is designed around `kimi-webbridge`: a local bridge that drives the user's real Chrome session, so command-line tools can automate logged-in websites without API keys or tokens.

## Goals

- Reimplement selected `x-cli` examples in Rust.
- Keep stdout JSON stable and agent-friendly.
- Provide reusable crates for browser-driven CLI tools.
- Ship prebuilt binaries for macOS, Linux, and Windows.

## Workspace layout

```text
crates/
  xcli/                Top-level `x` CLI entrypoint
  xcli-core/           Shared errors, config, and small utilities
  xcli-output/         Stable JSON response and error output
  xcli-webbridge/      HTTP client for kimi-webbridge-compatible daemons
  xcli-browser/        Browser action abstraction over the bridge
  xcli-chatgpt-image/  Reusable ChatGPT image generation flow
  xcli-google/         Reusable Google Search flow
  xcli-baidu/          Reusable Baidu Search flow
examples/
  chatgpt-image-cli/   Compatibility binary for the original CLI shape
  google-cli/          Compatibility binary for Google Search
  baidu-cli/           Compatibility binary for Baidu Search
```

## Install

Install the latest release on macOS or Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/hu-qi/x-cli-rs/main/install.sh | sh
```

Use `wget` when `curl` is not available:

```bash
wget -qO- https://raw.githubusercontent.com/hu-qi/x-cli-rs/main/install.sh | sh
```

Install a specific version:

```bash
XCLI_RS_VERSION=v0.1.0 curl -fsSL https://raw.githubusercontent.com/hu-qi/x-cli-rs/main/install.sh | sh
```

Install to a custom directory:

```bash
XCLI_RS_INSTALL_DIR=/usr/local/bin curl -fsSL https://raw.githubusercontent.com/hu-qi/x-cli-rs/main/install.sh | sh
```

Install on Windows PowerShell:

```powershell
iwr https://raw.githubusercontent.com/hu-qi/x-cli-rs/main/install.ps1 -UseB | iex
```

The installers download the release zip for your platform, verify the `.sha256` checksum, and install:

```text
x
chatgpt-image-cli
google-cli
baidu-cli
```

## Usage

Unified ChatGPT image entrypoint:

```bash
x chatgpt-image generate "a cute panda riding a bicycle" -o ./images
```

Short aliases:

```bash
x image g "a cat in a space suit" --timeout 180
x img gen "夕阳下的富士山" -o ./images
```

Unified Google Search entrypoint:

```bash
x google search "rust cli" --limit 10 --hl en
```

Unified Baidu Search entrypoint:

```bash
x baidu search "大模型" --limit 10
x baidu search "天气 北京" -n 20 --all
```

Compatibility entrypoints:

```bash
chatgpt-image-cli generate "a cute panda riding a bicycle" -o ./images
google-cli search "rust cli" --limit 10 --hl en
baidu-cli search "大模型" --limit 10
baidu-cli search "天气 北京" -n 20 --all
```

The unified and compatibility entrypoints call the same reusable library flows.

## Requirements

- `kimi-webbridge` daemon running at `http://127.0.0.1:10086` by default.
- Chrome WebBridge extension connected.
- You are already signed in to the target website in that Chrome profile.

Override the bridge URL when needed:

```bash
XCLI_WEBBRIDGE_URL=http://127.0.0.1:10086 x chatgpt-image generate "hello"
XCLI_WEBBRIDGE_URL=http://127.0.0.1:10086 x google search "rust cli"
XCLI_WEBBRIDGE_URL=http://127.0.0.1:10086 x baidu search "大模型"
```

## Debugging

Use `--verbose` to print flow-level logs to stderr while keeping stdout as machine-readable JSON:

```bash
x --verbose chatgpt-image generate "hello" -o ./images
x --verbose google search "rust cli"
x --verbose baidu search "大模型"
chatgpt-image-cli --verbose generate "hello" -o ./images
google-cli --verbose search "rust cli"
baidu-cli --verbose search "大模型"
```

Verbose ChatGPT image logs show:

```text
status -> navigate -> input -> submit -> wait_url -> wait_image -> read_image_meta -> download_image -> write_file
```

Google selector and consent behavior is documented in [Google Search DOM Archaeology](docs/google-archaeology.md).

Set `RUST_LOG` for more control:

```bash
RUST_LOG=debug x --verbose chatgpt-image generate "hello"
```

## Expected successful output

ChatGPT image output:

```json
{
  "ok": true,
  "data": {
    "prompt": "a cute panda riding a bicycle",
    "path": "/absolute/path/to/chatgpt-20260518-120000.png",
    "bytes": 2228437,
    "caption": "...",
    "conversation_url": "https://chatgpt.com/c/...",
    "elapsed_ms": 59970
  }
}
```

Google Search output:

```json
{
  "ok": true,
  "data": [
    {
      "title": "...",
      "url": "https://example.com",
      "snippet": "..."
    }
  ]
}
```

Baidu Search output:

```json
{
  "ok": true,
  "data": {
    "query": "大模型",
    "count": 1,
    "results": [
      {
        "rank": 1,
        "id": "...",
        "tpl": "www_index",
        "title": "...",
        "url": "https://example.com",
        "abstract": "...",
        "source": "..."
      }
    ]
  }
}
```

## Status

This repository is being bootstrapped. The current milestone is a testable `chatgpt-image`, `google`, and `baidu` implementation with:

- A unified `x` entrypoint.
- Compatibility `chatgpt-image-cli`, `google-cli`, and `baidu-cli` entrypoints.
- Shared JSON output helpers.
- A `kimi-webbridge` protocol client.
- Mock-tested ChatGPT image generation, Google Search, and Baidu Search flows.
- Optional verbose tracing for real browser debugging.
- Release packaging and install scripts.

## Development

Use the Makefile for the common local workflow:

```bash
make lock
make check
make build
make verify
```

Equivalent cargo commands:

```bash
cargo generate-lockfile
cargo fmt --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --release --locked -p xcli -p chatgpt-image-cli -p google-cli -p baidu-cli
```

Real WebBridge smoke tests:

```bash
make run-image
make run-google
make run-baidu
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for Cargo.lock policy, PR checklist, and release expectations.

## Release

Before publishing, complete the [release checklist](docs/release-checklist.md).

The release workflow builds:

```text
x
chatgpt-image-cli
google-cli
baidu-cli
```

Release artifacts are zipped per target triple:

```text
x-cli-rs-x86_64-unknown-linux-gnu.zip
x-cli-rs-aarch64-apple-darwin.zip
x-cli-rs-x86_64-apple-darwin.zip
x-cli-rs-x86_64-pc-windows-msvc.zip
```

Each zip has a matching SHA256 file:

```text
x-cli-rs-x86_64-unknown-linux-gnu.zip.sha256
```

Create a release by pushing a version tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The workflow can also be run manually from GitHub Actions via `workflow_dispatch`.

## Compatibility principles

- Stable command arguments.
- Stable stdout JSON.
- Stable error codes.
- Stable exit codes.
- Release assets for common desktop/server platforms.
