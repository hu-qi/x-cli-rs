# Xiaohongshu CLI Guide

This document describes the `xcli-xiaohongshu` integration and how to use the `xiaohongshu-cli` / `x xiaohongshu` commands.

## Requirements

- A running `kimi-webbridge` daemon (default `http://127.0.0.1:10086`).
- Chrome WebBridge extension connected.
- You must be **logged in** to [xiaohongshu.com](https://www.xiaohongshu.com) in the bridged Chrome profile.

## Commands

### Search notes

```bash
x xiaohongshu search "穿搭" --limit 10
x xhs search "护肤" -n 5
xiaohongshu-cli search "美食探店" --limit 20
```

### User profile

```bash
x xiaohongshu profile USER_ID --limit 10
xiaohongshu-cli profile USER_ID -n 20
```

### Note detail

```bash
x xiaohongshu note NOTE_ID
xiaohongshu-cli note NOTE_ID
```

### Comments

```bash
x xiaohongshu comments NOTE_ID --limit 30
xiaohongshu-cli comments NOTE_ID -n 50
```

## Output examples

### Search output

```json
{
  "ok": true,
  "data": {
    "keyword": "穿搭",
    "count": 2,
    "notes": [
      {
        "id": "abc123",
        "title": "Summer OOTD",
        "desc": "Casual summer outfit ideas...",
        "author": "Alice",
        "author_id": "5f3a9b2c",
        "likes": "1.2k",
        "cover": "https://.../cover.jpg",
        "url": "https://www.xiaohongshu.com/explore/abc123"
      }
    ]
  }
}
```

### Profile output

```json
{
  "ok": true,
  "data": {
    "user": {
      "nickname": "Alice",
      "user_id": "5f3a9b2c",
      "avatar": "https://.../avatar.jpg",
      "bio": "Fashion blogger | Daily OOTD",
      "followers": "10k",
      "following": "200",
      "notes_count": "150"
    },
    "notes": [
      {
        "id": "abc123",
        "title": "Summer OOTD",
        "desc": "",
        "author": "Alice",
        "author_id": "5f3a9b2c",
        "likes": "1.2k",
        "cover": "",
        "url": "https://www.xiaohongshu.com/explore/abc123"
      }
    ]
  }
}
```

### Note detail output

```json
{
  "ok": true,
  "data": {
    "id": "abc123",
    "title": "Summer OOTD",
    "content": "Today I want to share...",
    "author": "Alice",
    "author_id": "5f3a9b2c",
    "likes": "1.2k",
    "collects": "500",
    "comments_count": "120",
    "images": [
      "https://.../img1.jpg",
      "https://.../img2.jpg"
    ],
    "url": "https://www.xiaohongshu.com/explore/abc123",
    "publish_time": "2024-06-15"
  }
}
```

### Comments output

```json
{
  "ok": true,
  "data": {
    "note_id": "abc123",
    "count": 2,
    "comments": [
      {
        "id": "",
        "user": "Bob",
        "user_id": "u2",
        "avatar": "",
        "content": "Great post!",
        "likes": "10",
        "time": "2 days ago",
        "replies": []
      }
    ]
  }
}
```

## DOM extraction strategy

Xiaohongshu is a SPA with hashed CSS class names, so `xcli-xiaohongshu` avoids brittle class-based selectors. Instead it uses:

1. **Stable URL patterns**: `/explore/<note_id>` and `/user/profile/<user_id>`.
2. **Link `href` scanning**: `querySelectorAll('a[href*="/explore/"]')` finds all note cards.
3. **DOM-tree heuristics**: walks up the parent chain from the link to locate the card container, then extracts nearby text, images, and numbers.
4. **Text-based inference**: identifies interaction counts by regex (`/^\d+[\d\.]*[kw]?$/i`), dates by patterns, and user names from profile links.

## Debugging

Run with `--verbose` to see flow-level logs on stderr:

```bash
x --verbose xiaohongshu search "穿搭" --limit 5
xiaohongshu-cli --verbose search "护肤"
```

Set `RUST_LOG` for more detail:

```bash
RUST_LOG=debug x --verbose xiaohongshu search "穿搭"
```

### Manual selector investigation

In the connected Chrome profile:

1. Open `https://www.xiaohongshu.com/search_result?keyword=穿搭`.
2. Open DevTools.
3. Verify note links exist:

```js
Array.from(document.querySelectorAll('a[href*="/explore/"]')).length
```

4. Inspect a single card:

```js
const a = document.querySelector('a[href*="/explore/"]');
let card = a;
for (let i = 0; i < 6; i++) card = card.parentElement;
console.log(card.innerText.slice(0, 200));
```

If no candidates are returned, check whether:

- The page is fully loaded (SPA hydration may take a few seconds).
- You are logged in (some content requires login).
- Xiaohongshu has changed its DOM structure.

## Known limitations

- **Hashed class names**: CSS classes change frequently; the heuristic approach may need tuning.
- **Login required**: Most content requires an active login session.
- **Rate limiting**: Aggressive scraping may trigger anti-bot measures. Use reasonable `--limit` values.
- **Comment deduplication**: The comment extractor uses text+user heuristics to deduplicate; nested replies are best-effort.
- **Image URLs**: Some images may be lazy-loaded (`data-src` instead of `src`).

## Update checklist for selector changes

When updating extractors:

- [ ] Keep output shape unchanged for each command.
- [ ] Keep stdout JSON-only.
- [ ] Keep errors stable.
- [ ] Update `xcli-xiaohongshu` tests.
- [ ] Update this document with new selector reasoning.
- [ ] Validate both `x xiaohongshu <cmd>` and `xiaohongshu-cli <cmd>`.
