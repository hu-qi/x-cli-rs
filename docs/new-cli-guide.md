# New Browser CLI Guide

This guide is the Rust replacement for the original `skills/agent-cli-creator` workflow from `better-world-ai/x-cli`.

Use it when adding a new browser-backed CLI to `x-cli-rs`.

## Goal

A new site integration should provide both:

```text
x <site> <command> ...
<site>-cli <command> ...
```

Both entrypoints must call the same reusable library crate.

## Phase 1: Prerequisites

The current implementation uses the `kimi-webbridge` compatibility backend.

Check the daemon and Chrome extension:

```bash
curl http://127.0.0.1:10086/status
```

Proceed only when the daemon is reachable and the Chrome extension is connected.

The user must already be signed in to the target website in the same Chrome profile.

## Phase 2: Requirements interview

Before writing code, record:

1. Target website URL.
2. Whether login is required.
3. First 1-3 features to implement.
4. Output shape expected by users or by the original CLI.
5. Side effects, if any.

Prefer read-only features first:

```text
search
feed/list
profile/detail
status/check
```

Add write features only after read features work end-to-end.

## Phase 3: Site archaeology

Do not write business logic before site archaeology is complete.

For every planned feature, document:

```text
Feature:
Target page:
DOM selectors:
Network endpoints, if used:
Auth behavior:
Working browser eval call:
Expected output shape:
Known failure modes:
```

### 3.1 Navigate

```bash
curl -s -X POST http://127.0.0.1:10086/command \
  -H 'Content-Type: application/json' \
  -d '{"action":"navigate","args":{"url":"<TARGET_URL>","newTab":true},"session":"<site>"}'
```

### 3.2 Snapshot

```bash
curl -s -X POST http://127.0.0.1:10086/command \
  -H 'Content-Type: application/json' \
  -d '{"action":"snapshot","session":"<site>"}'
```

Use the accessibility tree to identify visible text, buttons, inputs, and stable interaction points.

### 3.3 Network capture, when needed

Start capture:

```bash
curl -s -X POST http://127.0.0.1:10086/command \
  -H 'Content-Type: application/json' \
  -d '{"action":"network","args":{"cmd":"start"},"session":"<site>"}'
```

Trigger the action manually in Chrome, then stop and inspect requests:

```bash
curl -s -X POST http://127.0.0.1:10086/command \
  -H 'Content-Type: application/json' \
  -d '{"action":"network","args":{"cmd":"stop"},"session":"<site>"}'

curl -s -X POST http://127.0.0.1:10086/command \
  -H 'Content-Type: application/json' \
  -d '{"action":"network","args":{"cmd":"list"},"session":"<site>"}'
```

Inspect a candidate request:

```bash
curl -s -X POST http://127.0.0.1:10086/command \
  -H 'Content-Type: application/json' \
  -d '{"action":"network","args":{"cmd":"detail","requestId":"<ID>"},"session":"<site>"}'
```

### 3.4 Verify with evaluate

The browser context inherits cookies and session state.

```bash
curl -s -X POST http://127.0.0.1:10086/command \
  -H 'Content-Type: application/json' \
  -d '{"action":"evaluate","args":{"code":"return document.title"},"session":"<site>"}'
```

Only proceed once a working evaluation proves the feature can be implemented.

## Phase 4: Rust implementation layout

Add a reusable library crate:

```text
crates/xcli-<site>/
  Cargo.toml
  src/lib.rs
```

Add a compatibility binary:

```text
examples/<site>-cli/
  Cargo.toml
  src/main.rs
```

Wire the unified entrypoint through the command modules:

```text
crates/xcli/Cargo.toml
crates/xcli/src/commands/<site>.rs
crates/xcli/src/commands/mod.rs
```

Register release/install metadata through the manifest:

```text
xcli.manifest.toml
```

Then run:

```bash
cargo run -p xtask -- check
```

## Phase 5: Library crate conventions

The library crate should own site-specific logic and expose typed input/output.

Use this shape:

```rust
use serde::{Deserialize, Serialize};
use xcli_browser::Browser;
use xcli_core::{Result, XCliError};
use xcli_webbridge::BrowserBridge;

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub query: String,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

pub async fn search<B>(browser: &Browser<B>, options: SearchOptions) -> Result<Vec<SearchResult>>
where
    B: BrowserBridge,
{
    if options.query.trim().is_empty() {
        return Err(XCliError::MissingArgs(
            "search requires a query: <site>-cli search <query>".to_string(),
        ));
    }

    browser.goto("https://example.com").await?;
    browser.eval("(() => [])()").await
}
```

Rules:

- Keep business logic out of `crates/xcli` and `examples/*`.
- Keep JSON output handled by `xcli-output` only.
- Use stable error codes from `xcli-core`.
- Map unexpected browser/eval failures into feature-specific errors such as `search_failed` or `generate_failed`.
- Log to stderr through tracing, never stdout.

## Phase 6: Compatibility CLI conventions

A compatibility binary should be thin.

It should:

- Parse flags with `clap`.
- Initialize optional verbose tracing.
- Construct `WebBridgeClient::with_session`.
- Call the reusable crate.
- Print `JsonResponse`.
- Exit non-zero on error.

Do not duplicate site logic in the compatibility binary.

## Phase 7: Unified `x` entrypoint conventions

Add a site module:

```text
crates/xcli/src/commands/<site>.rs
```

Then register it once in:

```text
crates/xcli/src/commands/mod.rs
```

The unified command should expose:

```bash
x <site> <command> ...
```

If the original CLI had a short or common alias, add it carefully:

```rust
#[command(name = "nanobanana", aliases = ["nano", "banana"])]
```

Avoid duplicate aliases. For example, do not add `alias = "generate"` to a variant already named `Generate`.

Keep the compatibility CLI command shape stable:

```bash
<site>-cli <command> ...
```

## Phase 8: Testing

Every reusable crate should include mock tests.

Use a mock `BrowserBridge` that returns queued `serde_json::Value` responses. Cover at least:

- URL construction.
- Empty required arguments.
- Successful parsing.
- Limit truncation or output shaping.
- Site-specific errors such as consent, no results, missing image, or refusal.

Also run the manifest consistency check:

```bash
cargo run -p xtask -- check
```

## Phase 9: Release integration checklist

When adding a new CLI, update:

- [ ] Workspace members in `Cargo.toml`.
- [ ] `crates/xcli/Cargo.toml` dependency list.
- [ ] `crates/xcli/src/commands/<site>.rs` command module.
- [ ] `crates/xcli/src/commands/mod.rs` registration.
- [ ] `xcli.manifest.toml` binary/package/smoke metadata.
- [ ] `README.md` layout, usage, output examples, release binary list.
- [ ] `README-zh.md` matching translated sections.
- [ ] `docs/release-checklist.md` local checks, smoke tests, JSON contract, install checks.
- [ ] Optional site archaeology document under `docs/` for fragile DOM selectors.
- [ ] `cargo run -p xtask -- check` passes.

The following files are checked against `xcli.manifest.toml` and should not drift:

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

## Phase 10: Optional companion skill

A companion skill is not required for this Rust rewrite, but can be useful later.

Purpose:

- The creator guide explains how to build a CLI.
- A companion skill explains how to use an already-built CLI.

Use `docs/companion-skill-template.md` if you want to create one later.
