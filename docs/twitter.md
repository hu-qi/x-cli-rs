# Twitter / x.com CLI Guide

This document describes the `xcli-twitter` integration and how to use the
`twitter-cli` / `x twitter` commands. The integration drives a real Chrome
session through `kimi-webbridge`, so it works on the **logged-in** x.com web
app — no API tokens or developer keys are required.

## Why a browser-bridge integration

The official X API v2 free tier no longer exposes search / read endpoints,
and the paid tiers start at $5,000/month. Driving the logged-in web app
through `kimi-webbridge` is currently the most reliable, no-API-key way to
read tweets, profiles, replies, and embedded media.

## Requirements

- A running `kimi-webbridge` daemon (default `http://127.0.0.1:10086`).
- Chrome WebBridge extension connected.
- You must be **logged in** to [x.com](https://x.com) in the bridged Chrome
  profile. Most x.com pages now require authentication.

## Commands

### Search tweets

```bash
x twitter search "rust cli" --limit 20
x twitter search "rust" --mode live          # Latest tab
x twitter search "ai" --mode image -n 30     # Photos tab
x tw search "ai" --mode video                # alias
twitter-cli search "rust cli" -n 50
```

Modes: `top` (default), `live`, `user`, `image`, `video`.

### User profile

```bash
x twitter profile elonmusk --limit 20
x twitter profile @elonmusk
twitter-cli profile elonmusk -n 50
```

### Post (tweet) detail

```bash
x twitter post https://x.com/elonmusk/status/1234567890
x twitter post elonmusk/status/1234567890
x twitter post elonmusk/1234567890
x twitter post 1234567890                       # routed through /i/web/status/<id>
x twitter post 1234567890 --out ./out           # also download all images + videos
x twitter post 1234567890 --out ./out --throttle-ms 500
twitter-cli post elonmusk/status/1234567890 -o ./out
```

The `post` command returns the tweet text, author, time, all interaction
counts (replies / retweets / quotes / likes / bookmarks / views), and **all
embedded media**:

- `images`: `pbs.twimg.com/media/...` URLs as rendered in the DOM (often
  medium-size; see the limitations section for the `?name=orig` caveat)
- `videos`: direct `video.twimg.com/...mp4` URLs from `<source>` tags plus
  the video poster image
- `links`: external URLs (mentions, hashtags, internal `/status/` links are
  filtered out)
- `quoted`: best-effort sub-tweet payload when the tweet quotes another

#### `--out <dir>`: downloading media

When `--out <dir>` is set, the tool additionally downloads every URL in
`images[]` and `videos[]` and adds a `downloads` object to the response:

- Files are fetched **directly from the Twitter CDN** (`pbs.twimg.com`,
  `video.twimg.com`) via `reqwest`, **not** through the bridged Chrome
  session. This decouples your logged-in account from the download
  traffic and avoids the in-page "Download video" Premium gate.
- Downloads run **serially**, with `--throttle-ms` (default `250`)
  between each request. Use `--throttle-ms 0` to disable.
- Filenames are `<tweet_id>-image-NN.<ext>` and `<tweet_id>-video-NN.<ext>`.
- Per-asset failures are recorded under `downloads.errors[]` rather than
  aborting the command — the overall response is still `ok: true`.

##### ToS and account-safety notes

- The Twitter CDN endpoints are publicly reachable without cookies, so the
  download itself does not consume your logged-in session and is generally
  safe for occasional/personal use.
- "Download video" inside the x.com UI is a Premium-only feature, but that
  is a UI gate, not a CDN authorization gate. Saving the mp4 URLs that the
  page already exposes is a long-standing third-party pattern.
- Bulk / high-frequency use can still trigger IP-level rate limiting and,
  more importantly, the **`post` and `replies` page loads themselves**
  (which do go through your logged-in browser) can trigger account-level
  anti-automation enforcement. Use reasonable rates.
- You are responsible for complying with the X Terms of Service and any
  applicable copyright / privacy laws when redistributing downloaded
  media.

The `replies` command intentionally does not accept `--out`. To download
media from a specific reply, run `x twitter post <reply_id> --out ./out`.

### Replies (comments)

```bash
x twitter replies https://x.com/elonmusk/status/1234567890 --limit 30
x twitter replies elonmusk/1234567890 -n 100
twitter-cli replies 1234567890 -n 50
```

The original (root) tweet is automatically skipped; only replies are
returned.

## Output examples

### Search output

```json
{
  "ok": true,
  "data": {
    "query": "rust cli",
    "mode": "top",
    "count": 1,
    "tweets": [
      {
        "id": "1800000000000000001",
        "url": "https://x.com/alice/status/1800000000000000001",
        "author": "Alice",
        "handle": "alice",
        "text": "Just shipped a new rust cli...",
        "time": "2026-05-19T12:00:00.000Z",
        "replies": "5",
        "retweets": "10",
        "likes": "100",
        "views": "1.2K",
        "images": ["https://pbs.twimg.com/media/abc.jpg"],
        "videos": [],
        "links": ["https://example.com/blog"]
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
      "handle": "alice",
      "display_name": "Alice",
      "bio": "Rustacean | OSS",
      "avatar": "https://pbs.twimg.com/profile_images/.../normal.jpg",
      "banner": "https://pbs.twimg.com/profile_banners/...",
      "location": "Earth",
      "website": "https://example.com",
      "joined": "Joined June 2020",
      "following": "200",
      "followers": "10K",
      "verified": false,
      "url": "https://x.com/alice"
    },
    "tweets": [ /* TweetSummary[] */ ]
  }
}
```

### Post detail output

Without `--out`:

```json
{
  "ok": true,
  "data": {
    "id": "1800000000000000001",
    "url": "https://x.com/alice/status/1800000000000000001",
    "author": "Alice",
    "handle": "alice",
    "text": "Demo post with media",
    "time": "2026-05-19T12:00:00.000Z",
    "replies": "5",
    "retweets": "10",
    "quotes": "2",
    "likes": "100",
    "bookmarks": "12",
    "views": "1.2K",
    "images": [
      "https://pbs.twimg.com/media/abc.jpg",
      "https://pbs.twimg.com/ext_tw_video_thumb/.../img.jpg"
    ],
    "videos": [
      "https://video.twimg.com/ext_tw_video/.../pl/720x1280/xxx.mp4"
    ],
    "links": ["https://example.com"],
    "quoted": null
  }
}
```

With `--out ./out`:

```json
{
  "ok": true,
  "data": {
    "id": "1800000000000000001",
    "...": "...same fields as above...",
    "downloads": {
      "out_dir": "./out",
      "images": [
        {
          "url": "https://pbs.twimg.com/media/abc.jpg",
          "path": "./out/1800000000000000001-image-01.jpg",
          "bytes": 184231
        }
      ],
      "videos": [
        {
          "url": "https://video.twimg.com/ext_tw_video/.../720x1280/xxx.mp4",
          "path": "./out/1800000000000000001-video-01.mp4",
          "bytes": 2204312
        }
      ]
    }
  }
}
```

If any single asset fails, the command still succeeds and reports per-URL
errors:

```json
{
  "ok": true,
  "data": {
    "...": "...",
    "downloads": {
      "out_dir": "./out",
      "images": [],
      "videos": [],
      "errors": [
        { "url": "https://pbs.twimg.com/media/abc.jpg", "error": "http 404" }
      ]
    }
  }
}
```

### Replies output

```json
{
  "ok": true,
  "data": {
    "tweet_id": "1800000000000000001",
    "url": "https://x.com/alice/status/1800000000000000001",
    "count": 1,
    "replies": [
      {
        "id": "1800000000000000111",
        "url": "https://x.com/bob/status/1800000000000000111",
        "author": "Bob",
        "handle": "bob",
        "text": "great post!",
        "time": "2026-05-19T13:00:00.000Z",
        "replies": "0",
        "retweets": "0",
        "likes": "1",
        "images": [],
        "videos": []
      }
    ]
  }
}
```

## DOM extraction strategy

`xcli-twitter` relies on the `data-testid` attributes that Twitter's own
QA tooling depends on. These have been stable for years even when the CSS
classes are reshuffled:

| Selector | Purpose |
| --- | --- |
| `article[data-testid="tweet"]` | Tweet container |
| `[data-testid="tweetText"]` | Tweet body text |
| `[data-testid="User-Name"]` | Display name + `@handle` block |
| `[data-testid="UserName"]` | Profile header name block |
| `[data-testid="UserDescription"]` | Profile bio |
| `[data-testid="UserProfileHeader_Items"]` | Location / website / join date |
| `[data-testid="tweetPhoto"] img` | Photo media |
| `video` (+ `<source>`) | Video URLs and poster |
| `time[datetime]` | ISO timestamp |
| `[data-testid="reply"\|"retweet"\|"like"\|"bookmark"]` | Interaction counts (from `aria-label`) |
| `a[href*="/status/"]` | Permalink → tweet id and handle |

Counts are parsed out of `aria-label` so they reflect Twitter's own
formatted numbers (`1.2K`, `3.4M`, etc.) rather than the visible UI string.

## Debugging

```bash
x --verbose twitter search "rust" --limit 5
twitter-cli --verbose post 1234567890

RUST_LOG=debug x --verbose twitter profile elonmusk
```

### Manual selector investigation

Inside the bridged Chrome profile:

```js
document.querySelectorAll('article[data-testid="tweet"]').length
// > 0 means the timeline rendered

const a = document.querySelector('article[data-testid="tweet"]');
console.log(a?.querySelector('[data-testid="tweetText"]')?.innerText);
console.log(a?.querySelector('[data-testid="User-Name"]')?.innerText);
```

If nothing appears, check whether:

- You are logged in (most x.com pages require auth).
- The page hydration finished (give it a few seconds).
- You are rate-limited / shown a challenge.
- x.com renamed a `data-testid` (rare, but happens).

## Known limitations

- **Login required**: x.com hides almost all content from logged-out users.
- **Rate limiting**: aggressive scraping triggers the standard "Rate limit
  exceeded" page. Use reasonable `--limit` values.
- **Pagination**: this integration extracts what is currently rendered.
  Twitter virtualizes its timeline, so even with a large `--limit` you may
  see fewer tweets than requested without scrolling.
- **Quoted tweets**: extracted on a best-effort basis; deeply nested quotes
  are not recursed.
- **Threads**: the current `replies` extractor returns immediate replies; it
  does not reconstruct threaded reply chains.
- **Spaces / community-only / circle tweets**: not supported.
- **`--out` on `replies`**: not implemented. Use
  `x twitter post <reply_id> --out ./out` for each reply you care about.
- **Image resolution**: downloads use the URL as it appears in the DOM
  (often medium-size). Twitter exposes `?name=orig` for full resolution but
  it is not automatically appended by the downloader to keep the URL
  verbatim from the page.

## Update checklist for selector changes

When updating extractors:

- [ ] Keep output shape unchanged for each command.
- [ ] Keep stdout JSON-only.
- [ ] Keep error codes stable.
- [ ] Update `xcli-twitter` tests.
- [ ] Update this document with new selector reasoning.
- [ ] Validate both `x twitter <cmd>` and `twitter-cli <cmd>`.
