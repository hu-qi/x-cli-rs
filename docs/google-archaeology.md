# Google Search DOM Archaeology

This document records the assumptions behind the `xcli-google` DOM extraction logic.

Google Search changes its markup frequently. The goal is not to depend on private APIs or scraping endpoints, but to drive the user's real Chrome session through `kimi-webbridge` and extract the same information that is visible in the page.

## Command surface

Unified entrypoint:

```bash
x google search "rust cli" --limit 10 --hl en
```

Compatibility entrypoint:

```bash
google-cli search "rust cli" --limit 10 --hl en
```

Both commands call the same `xcli-google` library flow.

## URL construction

The search URL is built as:

```text
https://www.google.com/search?q=<query>&hl=<hl>&num=<request_n>
```

Where:

- `q` is the user query.
- `hl` is the Google UI language.
- `num` is intentionally over-fetched.

The implementation over-fetches with:

```text
request_n = max(limit * 2, 10)
```

Reason: Google result pages often include non-organic blocks such as ads, related questions, widgets, and other cards. Over-fetching gives the extractor more candidate blocks while still truncating final output to the requested `limit`.

## Why `--hl en` defaults to English

`hl` affects the Google UI language and can affect nearby text, consent screens, and sometimes layout details. Keeping the default stable reduces DOM drift during automated parsing.

Use another language only when required:

```bash
x google search "Rust 命令行" --hl zh-CN
```

## Current selector strategy

The current extraction script waits for one of these conditions:

```js
location.host.startsWith('consent.')
document.querySelector('div#search div[data-hveid] h3')
```

Then it scans:

```js
div#search div[data-hveid]
```

A candidate block must contain:

```js
h3
a[href]
```

For each candidate, it returns:

```js
{
  title: h.innerText,
  url: a.href,
  snippet: (el.querySelector('[data-sncf]')?.innerText || '').replace(/\s*Read more\s*$/, '')
}
```

The extractor de-duplicates by result URL.

## Why these selectors

### `div#search`

Google's organic result area is commonly contained under `#search`. Restricting to this subtree reduces false positives from headers, menus, sidebars, and footer links.

### `div[data-hveid]`

Result-like blocks frequently carry `data-hveid`. It is not a public contract, but it has historically been a useful signal for Google Search result cards.

### `h3`

Organic results generally expose their clickable title in an `h3`. Requiring `h3` filters out many navigation, refinement, and widget blocks.

### `a[href]`

The result URL is taken from the first link in the block. This keeps extraction simple and close to the visible page structure.

### `[data-sncf]`

This is currently used as a snippet source. It may drift. When snippets become empty while titles and URLs still work, this selector is the first place to investigate.

## Consent behavior

If Google serves a consent interstitial, the extractor returns:

```json
{
  "consent": true,
  "items": []
}
```

The CLI then emits:

```json
{
  "ok": false,
  "error": {
    "code": "consent_required",
    "message": "google served a consent interstitial; accept it once in Chrome and retry"
  }
}
```

Fix:

1. Open the same Chrome profile connected to `kimi-webbridge`.
2. Accept the Google consent page once.
3. Re-run the command.

## Expected output

```json
{
  "ok": true,
  "data": [
    {
      "title": "...",
      "url": "https://example.com",
      "snippet": "..."
    }
  ]
}
```

## Failure modes

### `consent_required`

Google displayed a consent interstitial. Accept it once in Chrome and retry.

### `no_results`

Google returned no parseable results. Possible causes:

- Selector drift.
- Search page layout changed.
- Google displayed an anti-bot, CAPTCHA, or unusual interstitial.
- Query genuinely returned no standard web results.

### `search_failed`

The browser action or evaluation failed for an unexpected reason. Re-run with `--verbose` and inspect stderr.

## Debug commands

Run unified entrypoint with verbose logs:

```bash
x --verbose google search "rust cli" --limit 5 --hl en
```

Run compatibility entrypoint:

```bash
google-cli --verbose search "rust cli" --limit 5 --hl en
```

Check JSON contract:

```bash
x --verbose google search "rust cli" --limit 5 --hl en >/tmp/google-out.json 2>/tmp/google-err.log
python -m json.tool /tmp/google-out.json >/dev/null
cat /tmp/google-err.log
```

## Manual selector investigation

In the connected Chrome profile:

1. Open a Google search page.
2. Open DevTools.
3. Run:

```js
Array.from(document.querySelectorAll('div#search div[data-hveid]'))
  .filter(el => el.querySelector('h3') && el.querySelector('a[href]'))
  .map(el => ({
    title: el.querySelector('h3')?.innerText,
    url: el.querySelector('a[href]')?.href,
    snippet: el.querySelector('[data-sncf]')?.innerText || ''
  }))
```

If titles and URLs work but snippets are empty, search for a new snippet container near each `h3`.

If no candidates are returned, inspect whether:

- `#search` still exists.
- Result cards still carry `data-hveid`.
- Titles still use `h3`.
- Google is showing a consent, CAPTCHA, or non-standard page.

## Update checklist for selector changes

When updating selectors:

- [ ] Keep output shape unchanged: `title`, `url`, `snippet`.
- [ ] Keep stdout JSON-only.
- [ ] Keep errors stable.
- [ ] Update `xcli-google` tests.
- [ ] Update this document with the selector reasoning.
- [ ] Validate both `x google search` and `google-cli search`.
