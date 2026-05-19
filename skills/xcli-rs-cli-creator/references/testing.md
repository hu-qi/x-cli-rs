# Testing

Prefer deterministic tests for library crates.

## Mock BrowserBridge

Use a mock bridge with queued JSON responses.

Test these cases for every new integration:

- Empty required arguments.
- URL construction or JavaScript construction.
- Successful parse of browser output.
- Limit handling.
- Site-specific failure mapping.

## Real browser smoke tests

Real tests require:

- `kimi-webbridge` daemon at `http://127.0.0.1:10086`.
- Chrome extension connected.
- User logged in to the target site.

Add a Makefile target only for manual smoke tests:

```make
run-<site>:
	cargo run -p xcli -- --verbose <site> <command> ...
```

Do not require real browser tests in CI unless a stable test environment exists.

## Local verification

Run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --release --locked -p xcli -p <site>-cli
```
