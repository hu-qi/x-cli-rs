# Companion Skill Template

A companion skill tells a future AI agent how to use an existing CLI.

It is different from `docs/new-cli-guide.md`, which explains how to build a CLI.

Use this template only after a CLI works end-to-end.

Copy the block below into a skill `SKILL.md` and fill in the placeholders.

---

```markdown
---
name: {platform}-cli
description: Use when the user wants to [read/search/post/interact with] [Site Name]. Invoke when the user mentions "[site name]", asks to automate [site], or needs to [key action verbs].
---

# {Platform} CLI

Automates [Site Name] through `x-cli-rs` using the user's real logged-in Chrome session via `kimi-webbridge`.

## Prerequisites

1. `kimi-webbridge` daemon is running:

   ```bash
   curl http://127.0.0.1:10086/status
   ```

2. CLI binary is available:

   ```bash
   {platform}-cli --help
   ```

3. User is logged in to [Site Name] in Chrome.

   If not logged in, ask the user to open Chrome, visit [Site URL], log in manually, then retry.

## Commands

| Command | Args / Flags | Returns |
| --- | --- | --- |
| `{platform}-cli login-status` | none | `{logged_in, user_id, username}` |
| `{platform}-cli search` | `<query> [--limit N]` | `[{title, url, snippet}]` |
| `{platform}-cli item` | `<id>` | `{id, title, body}` |
| `{platform}-cli post` | `--content "text"` | `{id, url}` |

Remove commands that do not exist. Add every real command from `{platform}-cli --help`.

## Output Contract

All commands print JSON to stdout.

Success:

```json
{"ok": true, "data": {}}
```

Failure:

```json
{"ok": false, "error": {"code": "error_code", "message": "human-readable message"}}
```

Logs, if any, go to stderr.

## Common Workflows

### Search

```bash
{platform}-cli search "keyword" --limit 10
```

### Use the unified x entrypoint

```bash
x {platform} search "keyword" --limit 10
```

### Debug with verbose logs

```bash
{platform}-cli --verbose search "keyword" --limit 10 \
  >/tmp/{platform}-out.json \
  2>/tmp/{platform}-err.log
```

## Known Limitations

- Login is manual in Chrome.
- Requires `kimi-webbridge` and the Chrome extension.
- Site DOM/API may change.
- Rate limits or anti-bot interstitials may occur.
- [Add site-specific limitations.]
```

---

## Fill-in checklist

- [ ] Replace `{platform}`.
- [ ] Replace `[Site Name]` and `[Site URL]`.
- [ ] Update trigger phrases in `description`.
- [ ] Replace the command table with real commands.
- [ ] Add real workflows.
- [ ] Add known limitations.
- [ ] Confirm all examples print valid JSON.
