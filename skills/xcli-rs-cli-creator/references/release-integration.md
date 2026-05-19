# Release Integration

When adding a new binary, keep the release/install/documentation binary lists aligned through `xcli.manifest.toml` and `xtask`.

## Required files

Manual implementation files:

- Root `Cargo.toml`: add workspace members.
- `crates/xcli/Cargo.toml`: add library dependency.
- `crates/xcli/src/commands/<site>.rs`: add unified command implementation.
- `crates/xcli/src/commands/mod.rs`: register the command once.
- `crates/xcli-<site>/`: add reusable library crate.
- `examples/<site>-cli/`: add compatibility binary.

Manifest and docs files:

- `xcli.manifest.toml`: add the shipped binary, package name, aliases, and smoke command.
- `README.md`: add usage and output examples.
- `README-zh.md`: keep the Chinese README aligned with the English README.
- `docs/release-checklist.md`: add site-specific smoke checks or release notes.

Files checked by `xtask`:

- `Makefile`
- `.github/workflows/release.yml`
- `install.sh`
- `install.ps1`
- `README.md`
- `README-zh.md`
- `docs/release-checklist.md`

Run:

```bash
cargo run -p xtask -- check
```

CI runs the same check to catch stale binary lists.

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
