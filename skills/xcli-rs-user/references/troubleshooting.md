# Troubleshooting

## Bridge status

Check:

```bash
curl http://127.0.0.1:10086/status
```

Common states:

- Connection refused: daemon is not running.
- `running: false`: daemon started but not ready.
- `extension_connected: false`: Chrome extension is not connected.

## Error codes

### daemon_unreachable

The CLI cannot reach `http://127.0.0.1:10086` or the configured `XCLI_WEBBRIDGE_URL`.

Actions:

1. Start the daemon.
2. Verify the port.
3. Set `XCLI_WEBBRIDGE_URL` if using a non-default URL.

### daemon_not_running

The daemon responded but reported it is not running.

Restart the daemon and retry.

### extension_not_connected

Chrome extension is not connected to the daemon.

Open Chrome, confirm the extension is installed and enabled, then retry.

### consent_required

Usually from Google Search. Accept the consent page manually in Chrome, then rerun the command.

### no_results

The page loaded but no matching result selectors were found.

Retry with `--verbose` and inspect whether the target page changed layout.

### search_failed or generate_failed

Generic browser-flow failure.

Collect:

```bash
x --verbose <command> >/tmp/xcli-out.json 2>/tmp/xcli-err.log
cat /tmp/xcli-out.json
cat /tmp/xcli-err.log
```

Identify the failing phase from stderr, such as `status`, `navigate`, `input`, `submit`, `wait_image`, `download_image`, or `write_file`.
