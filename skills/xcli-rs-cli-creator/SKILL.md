---
name: xcli-rs-cli-creator
description: create or update browser-backed rust cli integrations in the x-cli-rs repository. use when the user asks to add a new website cli, port a better-world-ai/x-cli tool to rust, modify crates/xcli-* or examples/*-cli, perform kimi-webbridge site archaeology, define json stdout contracts, wire a new command into the unified x entrypoint, or update release/install/checklist files for a new x-cli-rs binary.
---

# x-cli-rs CLI Creator

Use this skill to add or modify browser-backed CLI commands in `hu-qi/x-cli-rs`.

The goal is always to produce a reusable Rust library crate plus a thin compatibility binary, then wire the feature into the unified `x` command.

## Core rules

- Do not put site-specific browser logic in `crates/xcli` or `examples/*-cli`.
- Put site logic in `crates/xcli-<site>/src/lib.rs`.
- Keep compatibility binaries thin: parse args, create `WebBridgeClient`, call the library crate, print `JsonResponse`.
- Keep stdout as JSON only. Logs go to stderr through tracing.
- Preserve stable error envelopes: `{ "ok": false, "error": { "code": "...", "message": "..." } }`.
- Prefer mock `BrowserBridge` tests before relying on real browser tests.
- For fragile DOM selectors, document the archaeology in `docs/` or a skill reference.

## Workflow

### 1. Clarify the integration

Confirm:

- Target website or original CLI being ported.
- Commands and flags to support.
- Required login/session assumptions.
- Output JSON shape.
- Whether the command reads data, writes data, downloads files, or triggers browser downloads.

### 2. Perform site archaeology first

Before writing code, inspect the site with `kimi-webbridge` using the process in `references/site-archaeology.md`.

Capture:

- Stable URLs.
- DOM selectors.
- Input, submit, and result readiness conditions.
- Download or network behavior.
- Known errors and user-visible failure states.

### 3. Implement the Rust layout

Follow `references/rust-layout.md`.

Expected layout:

```text
crates/xcli-<site>/
  Cargo.toml
  src/lib.rs
examples/<site>-cli/
  Cargo.toml
  src/main.rs
```

### 4. Wire the unified entrypoint

Update:

```text
crates/xcli/Cargo.toml
crates/xcli/src/main.rs
```

Support both:

```bash
x <site> <command> ...
<site>-cli <command> ...
```

### 5. Add tests and output contracts

Use `references/testing.md` and `references/output-contract.md`.

Each new crate should test:

- Empty required input.
- URL or script construction.
- Successful parsing.
- Error mapping.
- Output truncation or file writes when applicable.

### 6. Update release integration

Use `references/release-integration.md`.

Update every relevant file:

```text
Cargo.toml
Makefile
.github/workflows/release.yml
install.sh
install.ps1
README.md
docs/release-checklist.md
```

### 7. Validate

Ask the user to run or check CI for:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --release --locked -p xcli -p <site>-cli
```

If `Cargo.lock` is intentionally absent in the repository, release workflows may run `cargo generate-lockfile` before `--locked` verification.
