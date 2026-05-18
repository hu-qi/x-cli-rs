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
  xcli-core/          Shared errors, config, and small utilities
  xcli-output/        Stable JSON response and error output
  xcli-webbridge/     HTTP client for kimi-webbridge-compatible daemons
  xcli-browser/       Browser action abstraction over the bridge
examples/
  chatgpt-image-cli/  First compatibility target
```

## First target

```bash
chatgpt-image-cli generate "a cute panda riding a bicycle" -o ./images
```

Expected successful output:

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

This repository is being bootstrapped. The current milestone is a compilable workspace with stable response/error models and a thin `chatgpt-image-cli` scaffold.

## Development

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p chatgpt-image-cli -- generate "hello"
```

## Compatibility principles

- Stable command arguments.
- Stable stdout JSON.
- Stable error codes.
- Stable exit codes.
- Release assets for common desktop/server platforms.
