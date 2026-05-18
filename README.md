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
examples/
  chatgpt-image-cli/   Compatibility binary for the original CLI shape
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

The installers download the release zip for your platform, verify the `.sha256` checksum, and install both binaries:

```text
x
chatgpt-image-cli
```

## Usage

Unified entrypoint:

```bash
x chatgpt-image generate "a cute panda riding a bicycle" -o ./images
```

Short aliases:

```bash
x image g "a cat in a space suit" --timeout 180
x img gen "夕阳下的富士山" -o ./images
```

Compatibility entrypoint:

```bash
chatgpt-image-cli generate "a cute panda riding a bicycle" -o ./images
```

Both entrypoints call the same `xcli-chatgpt-image` library flow.

## Requirements

- `kimi-webbridge` daemon running at `http://127.0.0.1:10086` by default.
- Chrome WebBridge extension connected.
- You are already signed in to `chatgpt.com` in that Chrome profile.

Override the bridge URL when needed:

```bash
XCLI_WEBBRIDGE_URL=http://127.0.0.1:10086 x chatgpt-image generate "hello"
```

## Debugging

Use `--verbose` to print flow-level logs to stderr while keeping stdout as machine-readable JSON:

```bash
x --verbose chatgpt-image generate "hello" -o ./images
chatgpt-image-cli --verbose generate "hello" -o ./images
```

Verbose logs show the major browser-agent steps:

```text
status -> navigate -> input -> submit -> wait_url -> wait_image -> read_image_meta -> download_image -> write_file
```

Set `RUST_LOG` for more control:

```bash
RUST_LOG=debug x --verbose chatgpt-image generate "hello"
```

## Expected successful output

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

## Status

This repository is being bootstrapped. The current milestone is a testable `chatgpt-image` implementation with:

- A unified `x` entrypoint.
- A compatibility `chatgpt-image-cli` entrypoint.
- Shared JSON output helpers.
- A `kimi-webbridge` protocol client.
- Mock-tested ChatGPT image generation flow.
- Optional verbose tracing for real browser debugging.
- Release packaging and install scripts.

## Development

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p xcli -- chatgpt-image generate "hello"
cargo run -p chatgpt-image-cli -- generate "hello"
```

## Release

The release workflow builds both binaries:

```text
x
chatgpt-image-cli
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
