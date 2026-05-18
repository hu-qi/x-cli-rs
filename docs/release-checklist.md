# Release Checklist

Use this checklist before publishing the first public release of `x-cli-rs`.

## 1. Repository health

- [ ] GitHub Actions is enabled for the repository.
- [ ] `CI` workflow runs on push and pull requests.
- [ ] `Release` workflow is visible in the Actions tab.
- [ ] Branch protection rules are configured, if desired.

## 2. Local Rust checks

Run from the repository root:

```bash
cargo generate-lockfile
cargo fmt --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --release --locked -p xcli -p chatgpt-image-cli -p google-cli -p baidu-cli -p nanobanana-cli
```

Expected result:

- [ ] formatting passes
- [ ] clippy passes with `-D warnings`
- [ ] all tests pass
- [ ] release binaries build locally
- [ ] `target/release/x` exists
- [ ] `target/release/chatgpt-image-cli` exists
- [ ] `target/release/google-cli` exists
- [ ] `target/release/baidu-cli` exists
- [ ] `target/release/nanobanana-cli` exists

## 3. WebBridge compatibility

Before release, verify against a real `kimi-webbridge` daemon.

Prerequisites:

- [ ] daemon is running at `http://127.0.0.1:10086`
- [ ] Chrome extension is connected
- [ ] Chrome is signed in to `chatgpt.com`
- [ ] Chrome is signed in to `gemini.google.com`
- [ ] Chrome can open Google Search without a blocking consent page, or you are ready to accept the consent page once and retry
- [ ] Chrome can open Baidu Search
- [ ] ChatGPT Images page is available in the logged-in account
- [ ] Gemini can generate Nano Banana / Gemini 2.5 Flash Image responses in the logged-in account

### 3.1 ChatGPT image flow

Run:

```bash
cargo run -p xcli -- --verbose chatgpt-image generate "a cute panda riding a bicycle" -o ./images
cargo run -p chatgpt-image-cli -- --verbose generate "a cute panda riding a bicycle" -o ./images
```

Verify:

- [ ] verbose logs show `status`
- [ ] verbose logs show `navigate`
- [ ] verbose logs show `input`
- [ ] verbose logs show `submit`
- [ ] verbose logs show `wait_url`
- [ ] verbose logs show `wait_image`
- [ ] verbose logs show `read_image_meta`
- [ ] verbose logs show `download_image`
- [ ] verbose logs show `write_file`
- [ ] generated PNG exists
- [ ] generated PNG can be opened

### 3.2 Google Search flow

Run:

```bash
cargo run -p xcli -- --verbose google search "rust cli" --limit 5 --hl en
cargo run -p google-cli -- --verbose search "rust cli" --limit 5 --hl en
```

Verify:

- [ ] command succeeds, or returns `consent_required` with clear instructions
- [ ] after accepting consent in Chrome, the command succeeds on retry
- [ ] stdout contains `ok: true`
- [ ] stdout `data` is an array
- [ ] each result has `title`, `url`, and `snippet`
- [ ] `--limit 5` returns no more than five results
- [ ] `--hl en` is reflected in the generated Google URL in verbose logs

Selector assumptions and consent behavior are documented in [Google Search DOM Archaeology](google-archaeology.md). If Google extraction fails or selectors are changed, update that document in the same PR.

### 3.3 Baidu Search flow

Run:

```bash
cargo run -p xcli -- --verbose baidu search "大模型" --limit 5
cargo run -p baidu-cli -- --verbose search "大模型" --limit 5
cargo run -p baidu-cli -- --verbose search "天气 北京" -n 20 --all
```

Verify:

- [ ] command succeeds
- [ ] stdout contains `ok: true`
- [ ] stdout `data.query` matches the query
- [ ] stdout `data.count` matches the number of returned results
- [ ] stdout `data.results` is an array
- [ ] each result has `rank`, `id`, `tpl`, `title`, `url`, `abstract`, and `source`
- [ ] `--limit 5` returns no more than five results
- [ ] `-n 20` is reflected in the generated Baidu URL as `rn=20` in verbose logs
- [ ] `--all` does not change output shape

### 3.4 Nano Banana flow

Run:

```bash
cargo run -p xcli -- --verbose nanobanana gen "画一朵粉色月季花，微距特写" -o ./out --thumb-width 256 --timeout 300
cargo run -p nanobanana-cli -- --verbose gen "画一朵粉色月季花，微距特写" -o ./out --thumb-width 256 --timeout 300
```

Verify:

- [ ] command succeeds
- [ ] stdout contains `ok: true`
- [ ] stdout `data.full` points to an existing full-size PNG
- [ ] stdout `data.thumb` points to an existing thumbnail PNG
- [ ] `data.width` and `data.height` are non-zero
- [ ] `data.thumb_width` matches `--thumb-width`
- [ ] verbose logs show `wait_textbox`, `input`, `submit`, `wait_image`, `install_download_hook`, `click_download`, `fetch_image`, `write_full`, and `write_thumb`
- [ ] no native browser download/save dialog appears

## 4. JSON output contract

Successful ChatGPT image output must be valid JSON on stdout only:

```json
{
  "ok": true,
  "data": {
    "prompt": "...",
    "path": "...",
    "bytes": 123,
    "caption": "...",
    "conversation_url": "https://chatgpt.com/c/...",
    "elapsed_ms": 12345
  }
}
```

Successful Google Search output must be valid JSON on stdout only:

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

Successful Baidu Search output must be valid JSON on stdout only:

```json
{
  "ok": true,
  "data": {
    "query": "大模型",
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

Successful Nano Banana output must be valid JSON on stdout only:

```json
{
  "ok": true,
  "data": {
    "prompt": "画一朵粉色月季花，微距特写",
    "full": "/abs/path/out/20260518-120000-full.png",
    "thumb": "/abs/path/out/20260518-120000-thumb.png",
    "width": 2816,
    "height": 1536,
    "thumb_width": 256,
    "elapsed_ms": 184230
  }
}
```

Error output must be valid JSON on stdout only:

```json
{
  "ok": false,
  "error": {
    "code": "invalid_args",
    "message": "..."
  }
}
```

Verify:

- [ ] success exits with code `0`
- [ ] failure exits with code `1`
- [ ] stdout is JSON only
- [ ] verbose logs are written to stderr
- [ ] error codes are stable
- [ ] `chatgpt-image` supports `invalid_args`, `daemon_unreachable`, `daemon_not_running`, `extension_not_connected`, `generate_failed`
- [ ] `google` supports `missing_args`, `daemon_unreachable`, `daemon_not_running`, `extension_not_connected`, `consent_required`, `no_results`, `search_failed`
- [ ] `baidu` supports `missing_args`, `daemon_unreachable`, `daemon_not_running`, `extension_not_connected`, `search_failed`
- [ ] `nanobanana` supports `invalid_args`, `daemon_unreachable`, `daemon_not_running`, `extension_not_connected`, `generate_failed`

Recommended checks:

```bash
cargo run -p xcli -- chatgpt-image generate "" ; echo $?
cargo run -p xcli -- google search ; echo $?
cargo run -p xcli -- baidu search ; echo $?
cargo run -p xcli -- nanobanana gen "" ; echo $?
cargo run -p xcli -- --verbose chatgpt-image generate "hello" >/tmp/xcli-image-out.json 2>/tmp/xcli-image-err.log
cargo run -p xcli -- --verbose google search "rust cli" >/tmp/xcli-google-out.json 2>/tmp/xcli-google-err.log
cargo run -p xcli -- --verbose baidu search "大模型" >/tmp/xcli-baidu-out.json 2>/tmp/xcli-baidu-err.log
cargo run -p xcli -- --verbose nanobanana gen "画一朵花" >/tmp/xcli-nb-out.json 2>/tmp/xcli-nb-err.log
python -m json.tool /tmp/xcli-image-out.json >/dev/null
python -m json.tool /tmp/xcli-google-out.json >/dev/null
python -m json.tool /tmp/xcli-baidu-out.json >/dev/null
python -m json.tool /tmp/xcli-nb-out.json >/dev/null
```

## 5. Release workflow dry run

Use manual dispatch before tagging, if possible:

- [ ] Run `Release` workflow with `workflow_dispatch`.
- [ ] Linux artifact is produced.
- [ ] macOS arm64 artifact is produced.
- [ ] macOS x86_64 artifact is produced.
- [ ] Windows artifact is produced.
- [ ] Each artifact contains `x`, `chatgpt-image-cli`, `google-cli`, `baidu-cli`, and `nanobanana-cli`.
- [ ] Windows artifact contains `x.exe`, `chatgpt-image-cli.exe`, `google-cli.exe`, `baidu-cli.exe`, and `nanobanana-cli.exe`.
- [ ] Each artifact has a matching `.sha256` file.

## 6. Install scripts

After a release exists, verify install scripts.

macOS / Linux:

```bash
XCLI_RS_VERSION=v0.1.0 sh ./install.sh
XCLI_RS_VERSION=v0.1.0 XCLI_RS_INSTALL_DIR=/tmp/x-cli-rs-bin sh ./install.sh
```

Windows PowerShell:

```powershell
$env:XCLI_RS_VERSION="v0.1.0"
./install.ps1
```

Verify:

- [ ] correct target triple is detected
- [ ] release zip downloads
- [ ] checksum downloads
- [ ] checksum verification passes
- [ ] binaries are installed
- [ ] installed `x --help` works
- [ ] installed `chatgpt-image-cli --help` works
- [ ] installed `google-cli --help` works
- [ ] installed `baidu-cli --help` works
- [ ] installed `nanobanana-cli --help` works

## 7. Publish v0.1.0

Create and push the tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Verify:

- [ ] Release workflow starts automatically.
- [ ] Release workflow succeeds.
- [ ] GitHub Release is created.
- [ ] Release notes are generated.
- [ ] All zip files are attached.
- [ ] All checksum files are attached.

## 8. Post-release smoke test

Install from the public release:

```bash
curl -fsSL https://raw.githubusercontent.com/hu-qi/x-cli-rs/main/install.sh | sh
x --help
chatgpt-image-cli --help
google-cli --help
baidu-cli --help
nanobanana-cli --help
```

Run real commands:

```bash
x --verbose chatgpt-image generate "a cute panda riding a bicycle" -o ./images
x --verbose google search "rust cli" --limit 5 --hl en
x --verbose baidu search "大模型" --limit 5
x --verbose nanobanana gen "画一朵粉色月季花，微距特写" -o ./out --thumb-width 256 --timeout 300
```

Verify:

- [ ] commands succeed
- [ ] image file exists
- [ ] Google results are returned
- [ ] Baidu results are returned
- [ ] Nano Banana full image and thumb are written
- [ ] stdout JSON is valid
- [ ] stderr logs are useful

## 9. Rollback plan

If release is broken:

- [ ] Delete or mark the GitHub Release as prerelease.
- [ ] Create a patch branch.
- [ ] Fix the issue.
- [ ] Tag `v0.1.1`.
- [ ] Update README if install instructions changed.
- [ ] Add a release note explaining the fix.
