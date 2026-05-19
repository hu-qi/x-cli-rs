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
