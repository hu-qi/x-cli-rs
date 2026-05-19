# Release Integration

When adding a new binary, update every place that enumerates workspace members or release binaries.

## Required files

- Root `Cargo.toml`: add workspace members.
- `crates/xcli/Cargo.toml`: add library dependency.
- `crates/xcli/src/main.rs`: add unified command.
- `Makefile`: add build package and optional smoke target.
- `.github/workflows/release.yml`: add package to `cargo build` and Python packaging list.
- `install.sh`: add binary to `BINS`.
- `install.ps1`: add binary to `$Bins`.
- `README.md`: add usage and output examples.
- `docs/release-checklist.md`: add local build, smoke test, JSON contract, and install checks.

## Release workflow conventions

Use target-specific builds:

```bash
cargo build --release --locked --target ${{ matrix.target }} -p xcli -p <site>-cli
```

Package all binaries into the target zip and create a `.sha256` file.

Use `macos-15-intel` for `x86_64-apple-darwin` builds.

## Cargo.lock policy

If the repository intentionally does not commit `Cargo.lock`, release workflows may run:

```bash
cargo generate-lockfile
cargo check --workspace --locked
```

If `Cargo.lock` is committed, do not regenerate it inside release workflows.
