# Commands

Prefer the unified `x` command unless compatibility with the original CLI shape is required.

## Google Search

```bash
x google search "rust cli" --limit 10 --hl en
google-cli search "rust cli" --limit 10 --hl en
```

Output data is an array of:

```json
{
  "title": "...",
  "url": "https://example.com",
  "snippet": "..."
}
```

## Baidu Search

```bash
x baidu search "大模型" --limit 10
x baidu search "天气 北京" -n 20 --all
baidu-cli search "大模型" --limit 10
```

Output data contains:

```json
{
  "query": "大模型",
  "count": 1,
  "results": []
}
```

## ChatGPT Images

```bash
x chatgpt-image generate "a cute panda riding a bicycle" -o ./images
x image g "a cat in a space suit" --timeout 180
chatgpt-image-cli generate "a cute panda riding a bicycle" -o ./images
```

Requires the user to be signed in to ChatGPT in Chrome.

## Gemini Nano Banana

```bash
x nanobanana gen "a macro shot of a pink rose" -o ./out
x nano gen "a tiny robot in a garden" --thumb-width 320 --timeout 300
nanobanana-cli gen "a macro shot of a pink rose" -o ./out
```

Requires the user to be signed in to Gemini in Chrome.

## Xiaohongshu

```bash
x xiaohongshu search "穿搭" --limit 10
x xhs profile <user_id>
x xhs note <note_id>
x xhs comments <note_id> --limit 20
xiaohongshu-cli search "穿搭" --limit 10
```

Requires the user to be signed in to `xiaohongshu.com` in Chrome. See
`docs/xiaohongshu.md` for the full output schema.

## Twitter / x.com

```bash
x twitter search "rust cli" --limit 20
x twitter search "rust" --mode live          # Latest tab
x twitter profile <handle>
x twitter post <user>/status/<id>
x twitter post <tweet_id>                    # routed via /i/web/status/<id>
x twitter post <tweet_id> --out ./out        # also download images and videos
x twitter replies <tweet_id> --limit 30
twitter-cli search "rust cli" --limit 20
```

The `post` command always returns embedded `images[]` and `videos[]` URLs.
When `--out <dir>` is passed, the tool additionally downloads each asset
directly from the Twitter CDN (`pbs.twimg.com`, `video.twimg.com`) using a
serial throttled fetch and reports per-file `path` and `bytes` under
`downloads`.

Requires the user to be signed in to `x.com` in Chrome. Anonymous browsing
of x.com is heavily restricted. See `docs/twitter.md` for the full output
schema and selector reasoning.
