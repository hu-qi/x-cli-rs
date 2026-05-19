# JSON Contract

All commands print JSON to stdout.

## Success

```json
{
  "ok": true,
  "data": {}
}
```

Exit code: `0`.

## Failure

```json
{
  "ok": false,
  "error": {
    "code": "error_code",
    "message": "human-readable message"
  }
}
```

Exit code: non-zero.

## Verbose logs

Verbose logs are emitted to stderr:

```bash
x --verbose google search "rust cli" >/tmp/out.json 2>/tmp/err.log
```

Scripts should parse stdout only.
