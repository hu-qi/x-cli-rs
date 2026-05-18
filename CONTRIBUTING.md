# Contributing

Thanks for contributing to `x-cli-rs`.

This repository contains Rust implementations of browser-agent CLI tools backed by `kimi-webbridge`.

## Local setup

Install Rust stable from <https://rustup.rs/>.

Recommended local checks:

```bash
make lock
make check
make build
```

Equivalent cargo commands:

```bash
cargo generate-lockfile
cargo fmt --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --release --locked -p xcli -p chatgpt-image-cli -p google-cli
```

## Cargo.lock policy

`x-cli-rs` builds end-user CLI binaries, so `Cargo.lock` should be committed once generated.

Why:

- Release builds should be reproducible.
- Installer artifacts should be traceable to a locked dependency graph.
- CI and local builds should resolve the same dependency versions.

Generate or refresh the lockfile:

```bash
make lock
make locked-check
```

When changing dependencies:

```bash
cargo update -p <crate-name>
make verify
```

Then commit `Cargo.lock` together with the dependency change.

CI and release workflows use `--locked`, so missing or stale `Cargo.lock` will fail fast.

## JSON output contract

All CLI commands must emit machine-readable JSON on stdout.

Success:

```json
{
  "ok": true,
  "data": {}
}
```

Failure:

```json
{
  "ok": false,
  "error": {
    "code": "stable_error_code",
    "message": "human-readable message"
  }
}
```

Rules:

- stdout is JSON only.
- logs go to stderr.
- `--verbose` must not pollute stdout.
- success exits with code `0`.
- failure exits with non-zero code.
- error codes must remain stable after release.

## Browser-agent flow rules

Reusable browser logic belongs in `crates/*`.

Thin CLI entrypoints belong in:

```text
crates/xcli
examples/<name>-cli
```

A new website flow should usually add:

```text
crates/xcli-<site>/
examples/<site>-cli/
```

Then wire it into:

```text
crates/xcli
.github/workflows/release.yml
install.sh
install.ps1
README.md
docs/release-checklist.md
```

## Real WebBridge testing

Some behavior cannot be fully tested in CI because it depends on the user's real Chrome profile.

Before release, verify:

```bash
make run-image
make run-google
```

Requirements:

- `kimi-webbridge` daemon is running at `http://127.0.0.1:10086`.
- Chrome WebBridge extension is connected.
- You are signed in to the target website in that Chrome profile.

## Google Search selector changes

Google Search markup changes frequently.

If you change selectors in `xcli-google`, update:

```text
docs/google-archaeology.md
```

Also validate:

```bash
cargo run -p xcli -- --verbose google search "rust cli" --limit 5 --hl en
cargo run -p google-cli -- --verbose search "rust cli" --limit 5 --hl en
```

## Release checklist

Before publishing a release, complete:

```text
docs/release-checklist.md
```

Do not tag a release until:

- local checks pass
- real WebBridge smoke tests pass
- release workflow dry run succeeds, if available
- install scripts are verified against release artifacts

## PR checklist

Before opening a PR:

- [ ] Run `make lock` if dependencies changed or `Cargo.lock` is missing.
- [ ] Run `make check`.
- [ ] Run `make build` when touching CLI or release code.
- [ ] Update README for user-facing changes.
- [ ] Update docs for selector or protocol changes.
- [ ] Keep stdout JSON-only.
- [ ] Add or update tests for reusable crate logic.
