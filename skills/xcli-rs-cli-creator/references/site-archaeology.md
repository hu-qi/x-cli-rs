# Site Archaeology

Use this before implementing a new browser-backed CLI.

## Check bridge status

```bash
curl http://127.0.0.1:10086/status
```

Expected state:

```json
{
  "running": true,
  "extension_connected": true
}
```

## Navigate

```bash
curl -s -X POST http://127.0.0.1:10086/command \
  -H 'Content-Type: application/json' \
  -d '{"action":"navigate","args":{"url":"<TARGET_URL>","newTab":true},"session":"<site>"}'
```

## Snapshot

```bash
curl -s -X POST http://127.0.0.1:10086/command \
  -H 'Content-Type: application/json' \
  -d '{"action":"snapshot","session":"<site>"}'
```

Use the accessibility tree to identify stable user-visible controls and result regions.

## Evaluate

Use `evaluate` to prove each selector or browser action works before writing Rust code.

```bash
curl -s -X POST http://127.0.0.1:10086/command \
  -H 'Content-Type: application/json' \
  -d '{"action":"evaluate","args":{"code":"return document.title"},"session":"<site>"}'
```

## Network capture

Start capture:

```bash
curl -s -X POST http://127.0.0.1:10086/command \
  -H 'Content-Type: application/json' \
  -d '{"action":"network","args":{"cmd":"start"},"session":"<site>"}'
```

Stop and inspect:

```bash
curl -s -X POST http://127.0.0.1:10086/command \
  -H 'Content-Type: application/json' \
  -d '{"action":"network","args":{"cmd":"stop"},"session":"<site>"}'

curl -s -X POST http://127.0.0.1:10086/command \
  -H 'Content-Type: application/json' \
  -d '{"action":"network","args":{"cmd":"list"},"session":"<site>"}'
```

Inspect request details:

```bash
curl -s -X POST http://127.0.0.1:10086/command \
  -H 'Content-Type: application/json' \
  -d '{"action":"network","args":{"cmd":"detail","requestId":"<ID>"},"session":"<site>"}'
```

## Record findings

For each command, record:

```text
Feature:
Target URL:
Login requirement:
Selectors:
Readiness condition:
Network endpoint, if used:
Output JSON shape:
Failure states:
Manual verification command:
```
