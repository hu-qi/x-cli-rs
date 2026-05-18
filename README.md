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

## Usage

Unified entrypoint:

```bash
cargo run -p xcli -- chatgpt-image generate "a cute panda riding a bicycle" -o ./images
```

Short aliases:

```bash
cargo run -p xcli -- image g "a cat in a space suit" --timeout 180
cargo run -p xcli -- img gen "夕阳下的富士山" -o ./images
```

Compatibility entrypoint:

```bash
cargo run -p chatgpt-image-cli -- generate "a cute panda riding a bicycle" -o ./images
```

Both entrypoints call the same `xcli-chatgpt-image` library flow.

## Requirements

- `kimi-webbridge` daemon running at `http://127.0.0.1:10086` by default.
- Chrome WebBridge extension connected.
- You are already signed in to `chatgpt.com` in that Chrome profile.

Override the bridge URL when needed:

```bash
XCLI_WEBBRIDGE_URL=http://127.0.0.1:10086 cargo run -p xcli -- chatgpt-image generate "hello"
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

## Development

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p xcli -- chatgpt-image generate "hello"
cargo run -p chatgpt-image-cli -- generate "hello"
```

## Compatibility principles

- Stable command arguments.
- Stable stdout JSON.
- Stable error codes.
- Stable exit codes.
- Release assets for common desktop/server platforms.
