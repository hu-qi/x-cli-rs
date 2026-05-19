# x-cli-rs

[简体中文](README-zh.md) | English

Rust implementation of browser-agent CLI examples inspired by [`better-world-ai/x-cli`](https://github.com/better-world-ai/x-cli).

`x-cli-rs` provides a set of small, JSON-first command-line tools that automate websites through a real Chrome session. It is designed around `kimi-webbridge`, a local bridge that lets Rust CLIs drive logged-in browser pages without API keys, browser cookies, or service tokens.

## Highlights

- **One unified CLI**: use `x` for ChatGPT Images, Google Search, Baidu Search, Gemini Nano Banana image generation, and Xiaohongshu browsing.
- **Compatibility binaries**: keep dedicated commands such as `chatgpt-image-cli`, `google-cli`, `baidu-cli`, `nanobanana-cli`, and `xiaohongshu-cli`.
- **Stable JSON output**: stdout is designed for agents, scripts, and automation pipelines.
- **Reusable Rust crates**: each browser flow is split into a library crate that can be reused by other binaries.
- **Real browser automation**: uses your existing Chrome profile and login state through `kimi-webbridge`.
- **Release artifacts**: installers and zipped binaries are prepared for macOS, Linux, and Windows.

## Requirements

Before running any command, make sure the browser bridge is ready:

1. Start a `kimi-webbridge`-compatible daemon. The default URL is `http://127.0.0.1:10086`.
2. Connect the Chrome WebBridge extension.
3. Sign in to the target website in that Chrome profile.
4. Keep Chrome open while the CLI command runs.

Override the bridge URL when needed:

```bash
XCLI_WEBBRIDGE_URL=http://127.0.0.1:10086 x google search "rust cli"
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

The installers download the release zip for your platform, verify the matching `.sha256` checksum, and install these binaries:

```text
x
chatgpt-image-cli
google-cli
baidu-cli
nanobanana-cli
xiaohongshu-cli
```

## Quick start

Generate an image with ChatGPT Images:

```bash
x chatgpt-image generate "a cute panda riding a bicycle" -o ./images

# Short aliases
x image g "a cat in a space suit" --timeout 180
x img gen "sunset over Mt. Fuji" -o ./images
```

Search Google:

```bash
x google search "rust cli" --limit 10 --hl en
```

Search Baidu:

```bash
x baidu search "LLM" --limit 10
x baidu search "weather Beijing" -n 20 --all
```

Generate a Gemini Nano Banana image and thumbnail:

```bash
x nanobanana gen "a macro shot of a pink rose" -o ./out

# Short aliases
x nano gen "a tiny robot in a garden" --thumb-width 320 --timeout 300
x banana gen "a cyberpunk style teacup" -o ./out
```

Browse Xiaohongshu (Little Red Book):

```bash
x xiaohongshu search "穿搭" --limit 10
x xhs profile <user_id>
x xhs note <note_id>
x xhs comments <note_id> --limit 20
```

See the [Xiaohongshu CLI guide](docs/xiaohongshu.md) for selectors, login
requirements, and output schema details.

Every successful command writes a JSON object like this to stdout:

```json
{
  "ok": true,
  "data": {}
}
```

Errors use the same envelope shape and return a non-zero exit code:

```json
{
  "ok": false,
  "error": {
    "code": "missing_args",
    "message": "..."
  }
}
```

## Commands

### Unified `x` entrypoint

| Task | Command |
| --- | --- |
| ChatGPT image generation | `x chatgpt-image generate "prompt" -o ./images` |
| ChatGPT image aliases | `x image g "prompt"`, `x img gen "prompt"` |
| Google Search | `x google search "rust cli" --limit 10 --hl en` |
| Baidu Search | `x baidu search "LLM" --limit 10` |
| Baidu Search with all result types | `x baidu search "weather Beijing" -n 20 --all` |
| Gemini Nano Banana | `x nanobanana gen "prompt" -o ./out` |
| Nano Banana aliases | `x nano gen "prompt"`, `x banana gen "prompt"` |
| Xiaohongshu search | `x xiaohongshu search "穿搭" --limit 10` |
| Xiaohongshu profile | `x xhs profile <user_id>` |
| Xiaohongshu note detail | `x xhs note <note_id>` |
| Xiaohongshu comments | `x xhs comments <note_id> --limit 20` |

### Compatibility entrypoints

```bash
chatgpt-image-cli generate "a cute panda riding a bicycle" -o ./images
google-cli search "rust cli" --limit 10 --hl en
baidu-cli search "LLM" --limit 10
baidu-cli search "weather Beijing" -n 20 --all
nanobanana-cli gen "a macro shot of a pink rose" -o ./out
xiaohongshu-cli search "穿搭" --limit 10
xiaohongshu-cli profile <user_id>
xiaohongshu-cli note <note_id>
xiaohongshu-cli comments <note_id> --limit 20
```

The unified and compatibility entrypoints call the same reusable library flows.

## Output examples

### ChatGPT image output

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

### Google Search output

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

### Baidu Search output

```json
{
  "ok": true,
  "data": {
    "query": "LLM",
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

### Nano Banana output

```json
{
  "ok": true,
  "data": {
    "prompt": "a macro shot of a pink rose",
    "full": "/abs/path/out/20260518-120000-full.png",
    "thumb": "/abs/path/out/20260518-120000-thumb.png",
    "width": 2816,
    "height": 1536,
    "thumb_width": 256,
    "elapsed_ms": 184230
  }
}
```

## Debugging

Use `--verbose` to print flow-level logs to stderr while keeping stdout machine-readable:

```bash
x --verbose chatgpt-image generate "hello" -o ./images
x --verbose google search "rust cli"
x --verbose baidu search "LLM"
x --verbose nanobanana gen "a macro shot of a pink rose" -o ./out
```

Compatibility binaries also support `--verbose`:

```bash
chatgpt-image-cli --verbose generate "hello" -o ./images
google-cli --verbose search "rust cli"
baidu-cli --verbose search "LLM"
nanobanana-cli --verbose gen "a macro shot of a pink rose" -o ./out
```

Set `RUST_LOG` for more control:

```bash
RUST_LOG=debug x --verbose chatgpt-image generate "hello"
```

Verbose ChatGPT image logs show the high-level flow:

```text
status -> navigate -> input -> submit -> wait_url -> wait_image -> read_image_meta -> download_image -> write_file
```

Google selector and consent behavior is documented in [Google Search DOM Archaeology](docs/google-archaeology.md).

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
  xcli-nanobanana/     Reusable Gemini Nano Banana image flow
  xcli-xiaohongshu/    Reusable Xiaohongshu browsing flow (search/profile/note/comments)
examples/
  chatgpt-image-cli/   Compatibility binary for the original CLI shape
  google-cli/          Compatibility binary for Google Search
  baidu-cli/           Compatibility binary for Baidu Search
  nanobanana-cli/      Compatibility binary for Gemini Nano Banana
  xiaohongshu-cli/     Compatibility binary for Xiaohongshu
```

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
cargo build --release --locked -p xcli -p chatgpt-image-cli -p google-cli -p baidu-cli -p nanobanana-cli -p xiaohongshu-cli
```

Real WebBridge smoke tests:

```bash
make run-image
make run-google
make run-baidu
make run-nanobanana
```

Guides for adding new browser-backed CLIs:

- [New Browser CLI Guide](docs/new-cli-guide.md)
- [Site CLI Template](docs/site-cli-template.md)
- [Companion Skill Template](docs/companion-skill-template.md)

See [CONTRIBUTING.md](CONTRIBUTING.md) for Cargo.lock policy, PR checklist, and release expectations.

## Release

Before publishing, complete the [release checklist](docs/release-checklist.md).

The release workflow builds:

```text
x
chatgpt-image-cli
google-cli
baidu-cli
nanobanana-cli
xiaohongshu-cli
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

## Status

This repository is being actively bootstrapped. The current milestone is a testable browser-driven CLI suite with:

- A unified `x` entrypoint.
- Compatibility `chatgpt-image-cli`, `google-cli`, `baidu-cli`, `nanobanana-cli`, and `xiaohongshu-cli` entrypoints.
- Shared JSON output helpers.
- A `kimi-webbridge` protocol client.
- Mock-tested ChatGPT image generation, Google Search, Baidu Search, Nano Banana, and Xiaohongshu flows.
- Optional verbose tracing for real browser debugging.
- Release packaging and install scripts.

## Design principles

- Stable command arguments.
- Stable stdout JSON.
- Stable error codes.
- Stable exit codes.
- Reusable browser-flow crates.
- Release assets for common desktop/server platforms.
