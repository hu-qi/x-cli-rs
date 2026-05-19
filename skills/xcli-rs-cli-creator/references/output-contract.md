# Output Contract

All x-cli-rs commands must be machine-readable.

## Success

Stdout:

```json
{
  "ok": true,
  "data": {}
}
```

Exit code: `0`.

## Failure

Stdout:

```json
{
  "ok": false,
  "error": {
    "code": "missing_args",
    "message": "human-readable message"
  }
}
```

Exit code: non-zero, normally `1`.

## Logging

Verbose logs must go to stderr only:

```bash
x --verbose <site> <command> >/tmp/out.json 2>/tmp/err.log
```

Never print progress messages to stdout.

## Error code guidance

Use existing `xcli-core` variants when possible:

```text
missing_args
invalid_args
daemon_unreachable
daemon_not_running
extension_not_connected
browser_action_failed
search_failed
generate_failed
consent_required
no_results
```

Preserve bridge errors instead of hiding them behind feature errors:

```rust
match err {
    XCliError::DaemonUnreachable(_)
    | XCliError::DaemonNotRunning
    | XCliError::ExtensionNotConnected => err,
    other => XCliError::SearchFailed(other.to_string()),
}
```
