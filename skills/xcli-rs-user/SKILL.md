---
name: xcli-rs-user
description: use, install, run, and troubleshoot the x-cli-rs command line tools. use when the user asks how to run x, chatgpt-image-cli, google-cli, baidu-cli, or nanobanana-cli; needs help with kimi-webbridge status, chrome extension setup, json output, verbose logs, release installers, or common runtime errors such as daemon_unreachable, extension_not_connected, consent_required, search_failed, or generate_failed.
---

# x-cli-rs User

Use this skill to help users run installed `x-cli-rs` binaries and troubleshoot browser-backed commands.

## First checks

1. Confirm the binary exists:

   ```bash
   x --help
   google-cli --help
   baidu-cli --help
   chatgpt-image-cli --help
   nanobanana-cli --help
   ```

2. Confirm the local bridge is ready:

   ```bash
   curl http://127.0.0.1:10086/status
   ```

3. Confirm Chrome is open, the extension is connected, and the user is signed in to the target site.

## Command selection

Use `references/commands.md` to choose commands.

Prefer the unified entrypoint:

```bash
x google search "rust cli" --limit 5 --hl en
x baidu search "大模型" --limit 5
x chatgpt-image generate "a cute panda" -o ./images
x nanobanana gen "a macro shot of a pink rose" -o ./out
```

Use compatibility binaries when the user specifically asks for them:

```bash
google-cli search "rust cli" --limit 5 --hl en
baidu-cli search "大模型" --limit 5
chatgpt-image-cli generate "a cute panda" -o ./images
nanobanana-cli gen "a macro shot of a pink rose" -o ./out
```

## Debugging rules

- Preserve stdout JSON when collecting diagnostics.
- Redirect verbose logs to stderr.
- Ask for both stdout JSON and stderr when a command fails.
- Interpret error codes using `references/troubleshooting.md`.

Recommended diagnostic pattern:

```bash
x --verbose google search "rust cli" --limit 5 >/tmp/xcli-out.json 2>/tmp/xcli-err.log
cat /tmp/xcli-out.json
cat /tmp/xcli-err.log
```

## Output contract

Use `references/json-contract.md` when explaining outputs or building scripts around x-cli-rs.
