use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use xcli_browser::Browser;
use xcli_core::{Result, XCliError};
use xcli_webbridge::BrowserBridge;

// ---------------------------------------------------------------------------
// Data models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub query: String,
    pub limit: usize,
    /// Sort/filter mode: "live" (Latest), "top" (Top), "user", "image", "video".
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TweetSummary {
    pub id: String,
    pub url: String,
    pub author: String,
    pub handle: String,
    pub text: String,
    pub time: String,
    pub replies: String,
    pub retweets: String,
    pub likes: String,
    pub views: String,
    pub images: Vec<String>,
    pub videos: Vec<String>,
    pub links: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchOutput {
    pub query: String,
    pub mode: String,
    pub count: usize,
    pub tweets: Vec<TweetSummary>,
}

#[derive(Debug, Clone)]
pub struct ProfileOptions {
    pub handle: String,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct UserInfo {
    pub handle: String,
    pub display_name: String,
    pub bio: String,
    pub avatar: String,
    pub banner: String,
    pub location: String,
    pub website: String,
    pub joined: String,
    pub following: String,
    pub followers: String,
    pub verified: bool,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileOutput {
    pub user: UserInfo,
    pub tweets: Vec<TweetSummary>,
}

#[derive(Debug, Clone)]
pub struct PostOptions {
    /// Either a full URL (`https://x.com/<user>/status/<id>`),
    /// a `<user>/status/<id>` path, a `<user>/<id>` shortcut,
    /// or a bare tweet `<id>` (will be resolved through `/i/web/status/<id>`).
    pub reference: String,
    /// Optional output directory for downloading the tweet's embedded images
    /// and videos directly from the Twitter CDN. When `None`, downloads are
    /// skipped and the response contains URLs only.
    pub out_dir: Option<PathBuf>,
    /// Sleep between asset downloads to avoid hammering the CDN. Defaults to
    /// 250ms when constructed via [`PostOptions::new`]. Use [`Duration::ZERO`]
    /// to disable throttling.
    pub throttle: Duration,
}

impl PostOptions {
    pub fn new(reference: impl Into<String>) -> Self {
        Self {
            reference: reference.into(),
            out_dir: None,
            throttle: Duration::from_millis(250),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PostDetail {
    pub id: String,
    pub url: String,
    pub author: String,
    pub handle: String,
    pub text: String,
    pub time: String,
    pub replies: String,
    pub retweets: String,
    pub quotes: String,
    pub likes: String,
    pub bookmarks: String,
    pub views: String,
    pub images: Vec<String>,
    pub videos: Vec<String>,
    pub links: Vec<String>,
    pub quoted: Option<Box<PostDetail>>,
    /// Populated only when [`PostOptions::out_dir`] is set. Contains the local
    /// path and byte count for each successfully downloaded asset along with
    /// any per-asset error messages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub downloads: Option<DownloadReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DownloadReport {
    pub out_dir: String,
    pub images: Vec<DownloadedAsset>,
    pub videos: Vec<DownloadedAsset>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<DownloadError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DownloadedAsset {
    pub url: String,
    pub path: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DownloadError {
    pub url: String,
    pub error: String,
}

#[derive(Debug, Clone)]
pub struct RepliesOptions {
    pub reference: String,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplyItem {
    pub id: String,
    pub url: String,
    pub author: String,
    pub handle: String,
    pub text: String,
    pub time: String,
    pub replies: String,
    pub retweets: String,
    pub likes: String,
    pub images: Vec<String>,
    pub videos: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepliesOutput {
    pub tweet_id: String,
    pub url: String,
    pub count: usize,
    pub replies: Vec<ReplyItem>,
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

pub async fn search<B>(browser: &Browser<B>, options: SearchOptions) -> Result<SearchOutput>
where
    B: BrowserBridge,
{
    if options.query.trim().is_empty() {
        return Err(XCliError::MissingArgs(
            "search requires a query: twitter-cli search <query>".to_string(),
        ));
    }

    let mode = normalize_mode(&options.mode);
    let limit = normalize_limit(options.limit, 20);
    let url = search_url(&options.query, &mode);

    info!(step = "navigate", url = %url, "opening x.com search");
    browser.goto(&url).await.map_err(map_error)?;

    info!(step = "wait", "waiting for tweet timeline to render");
    browser
        .wait_for_js_truthy(timeline_ready_script(), Duration::from_secs(20))
        .await
        .map_err(|err| wait_error("search", err))?;

    info!(step = "extract", limit, "extracting tweets");
    let mut tweets: Vec<TweetSummary> = browser
        .eval(&timeline_extract_script())
        .await
        .map_err(map_error)?;

    if tweets.len() > limit {
        tweets.truncate(limit);
    }

    if tweets.is_empty() {
        return Err(XCliError::NoResults(
            "x.com returned no parseable tweets (login required, rate-limited, or DOM drifted)"
                .to_string(),
        ));
    }

    Ok(SearchOutput {
        query: options.query,
        mode,
        count: tweets.len(),
        tweets,
    })
}

// ---------------------------------------------------------------------------
// Profile
// ---------------------------------------------------------------------------

pub async fn profile<B>(browser: &Browser<B>, options: ProfileOptions) -> Result<ProfileOutput>
where
    B: BrowserBridge,
{
    let handle = clean_handle(&options.handle);
    if handle.is_empty() {
        return Err(XCliError::MissingArgs(
            "profile requires a handle: twitter-cli profile <handle>".to_string(),
        ));
    }

    let limit = normalize_limit(options.limit, 20);
    let url = profile_url(&handle);

    info!(step = "navigate", url = %url, "opening x.com profile");
    browser.goto(&url).await.map_err(map_error)?;

    info!(step = "wait", "waiting for profile to render");
    browser
        .wait_for_js_truthy(profile_ready_script(), Duration::from_secs(20))
        .await
        .map_err(|err| wait_error("profile", err))?;

    info!(step = "extract", "extracting profile and tweets");
    let payload: ProfilePayload = browser
        .eval(&profile_extract_script())
        .await
        .map_err(map_error)?;

    let mut tweets = payload.tweets;
    if tweets.len() > limit {
        tweets.truncate(limit);
    }

    let mut user = payload.user;
    if user.handle.is_empty() {
        user.handle = handle.clone();
    }
    if user.url.is_empty() {
        user.url = url;
    }

    Ok(ProfileOutput { user, tweets })
}

// ---------------------------------------------------------------------------
// Post detail
// ---------------------------------------------------------------------------

pub async fn post<B>(browser: &Browser<B>, options: PostOptions) -> Result<PostDetail>
where
    B: BrowserBridge,
{
    let url = resolve_post_url(&options.reference)?;

    info!(step = "navigate", url = %url, "opening x.com post");
    browser.goto(&url).await.map_err(map_error)?;

    info!(step = "wait", "waiting for tweet detail to render");
    browser
        .wait_for_js_truthy(post_ready_script(), Duration::from_secs(20))
        .await
        .map_err(|err| wait_error("post", err))?;

    info!(step = "extract", "extracting post detail");
    let mut detail: PostDetail = browser
        .eval(&post_extract_script())
        .await
        .map_err(map_error)?;

    if let Some(out_dir) = options.out_dir.as_deref() {
        info!(step = "download", out = %out_dir.display(), "downloading post media");
        let report = download_post_media(&detail, out_dir, options.throttle).await?;
        detail.downloads = Some(report);
    }

    Ok(detail)
}

// ---------------------------------------------------------------------------
// Media download
//
// Twitter CDN URLs under `pbs.twimg.com` and `video.twimg.com` are publicly
// reachable without cookies. We fetch them directly with `reqwest` (rather
// than asking the bridged Chrome to download), which keeps the user's
// logged-in session decoupled from the download traffic and avoids tripping
// the in-page "Download video" Premium gate.
//
// Downloads are serial with a small throttle between requests; per-asset
// failures are recorded in `errors` rather than aborting the command.
// ---------------------------------------------------------------------------

const DOWNLOAD_USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 \
                                   (KHTML, like Gecko) Chrome/124.0 Safari/537.36";

async fn download_post_media(
    detail: &PostDetail,
    out_dir: &Path,
    throttle: Duration,
) -> Result<DownloadReport> {
    tokio::fs::create_dir_all(out_dir).await.map_err(|err| {
        XCliError::BrowserActionFailed(format!("create out dir {}: {}", out_dir.display(), err))
    })?;

    let client = reqwest::Client::builder()
        .user_agent(DOWNLOAD_USER_AGENT)
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|err| XCliError::BrowserActionFailed(format!("build http client: {}", err)))?;

    let id = if detail.id.is_empty() {
        "tweet".to_string()
    } else {
        detail.id.clone()
    };

    let mut report = DownloadReport {
        out_dir: out_dir.display().to_string(),
        ..Default::default()
    };

    let mut first_image = true;
    for (idx, url) in detail.images.iter().enumerate() {
        if !first_image && !throttle.is_zero() {
            tokio::time::sleep(throttle).await;
        }
        first_image = false;
        let ext = guess_extension(url, "jpg");
        let filename = format!("{}-image-{:02}.{}", id, idx + 1, ext);
        match fetch_to_file(&client, url, out_dir, &filename).await {
            Ok(asset) => {
                info!(step = "image_downloaded", path = %asset.path, bytes = asset.bytes);
                report.images.push(asset);
            }
            Err(err) => {
                warn!(step = "image_failed", url = %url, error = %err);
                report.errors.push(DownloadError {
                    url: url.clone(),
                    error: err.to_string(),
                });
            }
        }
    }

    let mut first_video = true;
    for (idx, url) in detail.videos.iter().enumerate() {
        if !first_video && !throttle.is_zero() {
            tokio::time::sleep(throttle).await;
        }
        first_video = false;
        let ext = guess_extension(url, "mp4");
        let filename = format!("{}-video-{:02}.{}", id, idx + 1, ext);
        match fetch_to_file(&client, url, out_dir, &filename).await {
            Ok(asset) => {
                info!(step = "video_downloaded", path = %asset.path, bytes = asset.bytes);
                report.videos.push(asset);
            }
            Err(err) => {
                warn!(step = "video_failed", url = %url, error = %err);
                report.errors.push(DownloadError {
                    url: url.clone(),
                    error: err.to_string(),
                });
            }
        }
    }

    Ok(report)
}

async fn fetch_to_file(
    client: &reqwest::Client,
    url: &str,
    out_dir: &Path,
    filename: &str,
) -> std::result::Result<DownloadedAsset, String> {
    let response = client
        .get(url)
        .header("Referer", "https://x.com/")
        .send()
        .await
        .map_err(|err| format!("request failed: {}", err))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("http {}", status));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|err| format!("read body: {}", err))?;

    let path = out_dir.join(filename);
    tokio::fs::write(&path, &bytes)
        .await
        .map_err(|err| format!("write {}: {}", path.display(), err))?;

    Ok(DownloadedAsset {
        url: url.to_string(),
        path: path.display().to_string(),
        bytes: bytes.len() as u64,
    })
}

/// Pick a reasonable file extension from a Twitter CDN URL.
///
/// Twitter image URLs frequently look like
/// `https://pbs.twimg.com/media/<id>?format=jpg&name=large` — i.e. the real
/// extension lives in the `format` query parameter, not the path. Try the
/// query parameter first, then fall back to the path extension, then the
/// caller-supplied default.
fn guess_extension(url: &str, default: &str) -> String {
    fn looks_like_ext(candidate: &str) -> bool {
        !candidate.is_empty()
            && candidate.len() <= 5
            && candidate.chars().all(|c| c.is_ascii_alphanumeric())
    }

    if let Some(query) = url.split('?').nth(1) {
        for pair in query.split('&') {
            if let Some(value) = pair.strip_prefix("format=") {
                if looks_like_ext(value) {
                    return value.to_ascii_lowercase();
                }
            }
        }
    }

    let path_part = url.split('?').next().unwrap_or(url);
    if let Some(idx) = path_part.rfind('.') {
        let ext = &path_part[idx + 1..];
        if looks_like_ext(ext) {
            return ext.to_ascii_lowercase();
        }
    }
    default.to_string()
}

// ---------------------------------------------------------------------------
// Replies (comments)
// ---------------------------------------------------------------------------

pub async fn replies<B>(browser: &Browser<B>, options: RepliesOptions) -> Result<RepliesOutput>
where
    B: BrowserBridge,
{
    let url = resolve_post_url(&options.reference)?;
    let limit = normalize_limit(options.limit, 20);

    info!(step = "navigate", url = %url, "opening x.com post for replies");
    browser.goto(&url).await.map_err(map_error)?;

    info!(step = "wait", "waiting for replies to render");
    browser
        .wait_for_js_truthy(replies_ready_script(), Duration::from_secs(20))
        .await
        .map_err(|err| wait_error("replies", err))?;

    info!(step = "extract", limit, "extracting replies");
    let mut items: Vec<ReplyItem> = browser
        .eval(&replies_extract_script())
        .await
        .map_err(map_error)?;

    if items.len() > limit {
        items.truncate(limit);
    }

    let tweet_id = extract_tweet_id(&url).unwrap_or_default();

    Ok(RepliesOutput {
        tweet_id,
        url,
        count: items.len(),
        replies: items,
    })
}

// ---------------------------------------------------------------------------
// URL helpers / parsing
// ---------------------------------------------------------------------------

pub fn search_url(query: &str, mode: &str) -> String {
    let f = match mode {
        "live" => "&f=live",
        "user" => "&f=user",
        "image" => "&f=image",
        "video" => "&f=video",
        _ => "",
    };
    format!(
        "https://x.com/search?q={}&src=typed_query{}",
        urlencoding::encode(query),
        f,
    )
}

pub fn profile_url(handle: &str) -> String {
    format!("https://x.com/{}", clean_handle(handle))
}

pub fn post_url(handle: &str, tweet_id: &str) -> String {
    format!("https://x.com/{}/status/{}", clean_handle(handle), tweet_id)
}

pub fn post_url_by_id(tweet_id: &str) -> String {
    format!("https://x.com/i/web/status/{}", tweet_id)
}

pub fn resolve_post_url(reference: &str) -> Result<String> {
    let r = reference.trim();
    if r.is_empty() {
        return Err(XCliError::MissingArgs(
            "post requires a tweet reference: <id> or <user>/status/<id> or full URL".to_string(),
        ));
    }

    if r.starts_with("http://") || r.starts_with("https://") {
        return Ok(r.to_string());
    }

    // Strip leading slashes and `x.com/` / `twitter.com/` if pasted partially.
    let r = r
        .trim_start_matches('/')
        .trim_start_matches("x.com/")
        .trim_start_matches("twitter.com/")
        .trim_start_matches("www.x.com/")
        .trim_start_matches("www.twitter.com/");

    // Bare digits → /i/web/status/<id>
    if r.chars().all(|c| c.is_ascii_digit()) {
        return Ok(post_url_by_id(r));
    }

    // <user>/status/<id>
    let parts: Vec<&str> = r.split('/').collect();
    match parts.as_slice() {
        [user, "status", id] | [user, "statuses", id] => Ok(post_url(user, id)),
        // <user>/<id> shortcut
        [user, id] if id.chars().all(|c| c.is_ascii_digit()) => Ok(post_url(user, id)),
        _ => Err(XCliError::MissingArgs(format!(
            "could not parse tweet reference: {}",
            reference
        ))),
    }
}

pub fn extract_tweet_id(url: &str) -> Option<String> {
    // Find `/status/<digits>` or `/statuses/<digits>` segment.
    for marker in ["/status/", "/statuses/"] {
        if let Some(idx) = url.find(marker) {
            let tail = &url[idx + marker.len()..];
            let id: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !id.is_empty() {
                return Some(id);
            }
        }
    }
    None
}

fn clean_handle(h: &str) -> String {
    h.trim()
        .trim_start_matches('@')
        .trim_matches('/')
        .to_string()
}

fn normalize_limit(limit: usize, default: usize) -> usize {
    if limit == 0 {
        default
    } else {
        limit
    }
}

fn normalize_mode(mode: &str) -> String {
    let m = mode.trim().to_ascii_lowercase();
    match m.as_str() {
        "" | "top" => "top".to_string(),
        "live" | "latest" | "new" => "live".to_string(),
        "user" | "people" => "user".to_string(),
        "image" | "images" | "photo" | "photos" => "image".to_string(),
        "video" | "videos" => "video".to_string(),
        _ => "top".to_string(),
    }
}

fn map_error(err: XCliError) -> XCliError {
    match err {
        XCliError::DaemonUnreachable(_)
        | XCliError::DaemonNotRunning
        | XCliError::ExtensionNotConnected => err,
        other => XCliError::SearchFailed(other.to_string()),
    }
}

fn wait_error(what: &str, err: XCliError) -> XCliError {
    match err {
        XCliError::DaemonUnreachable(_)
        | XCliError::DaemonNotRunning
        | XCliError::ExtensionNotConnected => err,
        other => XCliError::SearchFailed(format!(
            "x.com {} did not load within timeout: {}",
            what, other
        )),
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ProfilePayload {
    #[serde(default)]
    user: UserInfo,
    #[serde(default)]
    tweets: Vec<TweetSummary>,
}

// ---------------------------------------------------------------------------
// JS extractor scripts
//
// x.com is an SPA with stable `data-testid` attributes used by Twitter's own
// QA. We rely on:
//   - `article[data-testid="tweet"]` for tweet containers
//   - `[data-testid="tweetText"]` for body text
//   - `[data-testid="User-Name"]` for the author block (display name + @handle)
//   - `[data-testid="tweetPhoto"] img` for photos
//   - `video` elements (poster URL + <source src>) for videos
//   - `time[datetime]` for ISO timestamps
//   - `[data-testid="like"|"reply"|"retweet"|"bookmark"]` aria-label numbers
//   - Permalinks via `a[href*="/status/"]`
// ---------------------------------------------------------------------------

fn timeline_ready_script() -> &'static str {
    r#"
    (() => {
      if (document.querySelector('article[data-testid="tweet"]')) return true;
      // "No results" / login-wall fallbacks should still resolve quickly.
      const empty = document.querySelector('[data-testid="emptyState"]');
      if (empty) return true;
      return false;
    })()
    "#
}

fn tweet_helpers_js() -> &'static str {
    // Reusable JS helpers injected at the top of every extract script.
    r#"
    function _xcliExtractCount(article, testid) {
      const el = article.querySelector('[data-testid="' + testid + '"]');
      if (!el) return '';
      const aria = el.getAttribute('aria-label') || '';
      const m = aria.match(/([\d,\.]+)\s*(K|M|B)?/i);
      if (m) return m[1] + (m[2] || '');
      const txt = (el.innerText || '').trim();
      const m2 = txt.match(/([\d,\.]+)\s*(K|M|B)?/i);
      if (m2) return m2[1] + (m2[2] || '');
      return '';
    }
    function _xcliExtractAuthor(article) {
      const block = article.querySelector('[data-testid="User-Name"]');
      let displayName = '';
      let handle = '';
      if (block) {
        const spans = block.querySelectorAll('span');
        for (const s of spans) {
          const t = (s.innerText || '').trim();
          if (!t) continue;
          if (t.startsWith('@') && !handle) {
            handle = t.slice(1);
          } else if (!displayName && !t.startsWith('@') && t !== '·') {
            displayName = t;
          }
        }
      }
      return { author: displayName, handle };
    }
    function _xcliExtractTime(article) {
      const t = article.querySelector('time');
      if (!t) return '';
      return t.getAttribute('datetime') || (t.innerText || '').trim();
    }
    function _xcliExtractText(article) {
      const el = article.querySelector('[data-testid="tweetText"]');
      return el ? (el.innerText || '').trim() : '';
    }
    function _xcliExtractMedia(article) {
      const images = [];
      const videos = [];
      const seenImg = new Set();
      const seenVid = new Set();
      const photoBlocks = article.querySelectorAll('[data-testid="tweetPhoto"] img, a[href*="/photo/"] img');
      for (const img of photoBlocks) {
        const src = img.src || img.getAttribute('data-src') || '';
        if (src && !seenImg.has(src) && /pbs\.twimg\.com\/media\//.test(src)) {
          seenImg.add(src);
          images.push(src);
        }
      }
      const vids = article.querySelectorAll('video');
      for (const v of vids) {
        const poster = v.poster || '';
        if (poster && !seenImg.has(poster)) {
          seenImg.add(poster);
          images.push(poster);
        }
        if (v.src) {
          if (!seenVid.has(v.src)) { seenVid.add(v.src); videos.push(v.src); }
        }
        const sources = v.querySelectorAll('source');
        for (const s of sources) {
          const sv = s.src || s.getAttribute('src') || '';
          if (sv && !seenVid.has(sv)) { seenVid.add(sv); videos.push(sv); }
        }
      }
      return { images, videos };
    }
    function _xcliExtractLinks(article) {
      const out = [];
      const seen = new Set();
      const tweetText = article.querySelector('[data-testid="tweetText"]');
      const root = tweetText || article;
      for (const a of root.querySelectorAll('a[href]')) {
        const href = a.href || '';
        if (!href) continue;
        // Filter out internal navigation (mentions, hashtags, status links).
        if (/^https?:\/\/(?:www\.)?(?:x|twitter)\.com\//.test(href)) continue;
        if (seen.has(href)) continue;
        seen.add(href);
        out.push(href);
      }
      return out;
    }
    function _xcliExtractPermalink(article) {
      const links = article.querySelectorAll('a[href*="/status/"]');
      for (const a of links) {
        const m = (a.getAttribute('href') || '').match(/^\/([^\/]+)\/status\/(\d+)/);
        if (m) {
          return {
            url: 'https://x.com' + a.getAttribute('href').split('?')[0],
            id: m[2],
            handle: m[1],
          };
        }
      }
      return { url: '', id: '', handle: '' };
    }
    function _xcliBuildSummary(article) {
      const author = _xcliExtractAuthor(article);
      const link = _xcliExtractPermalink(article);
      const media = _xcliExtractMedia(article);
      return {
        id: link.id,
        url: link.url,
        author: author.author,
        handle: author.handle || link.handle,
        text: _xcliExtractText(article),
        time: _xcliExtractTime(article),
        replies: _xcliExtractCount(article, 'reply'),
        retweets: _xcliExtractCount(article, 'retweet'),
        likes: _xcliExtractCount(article, 'like'),
        views: _xcliExtractCount(article, 'app-text-transition-container') ||
               _xcliExtractCount(article, 'analyticsButton'),
        images: media.images,
        videos: media.videos,
        links: _xcliExtractLinks(article),
      };
    }
    "#
}

fn timeline_extract_script() -> String {
    format!(
        r#"
    (() => {{
      {helpers}
      const articles = document.querySelectorAll('article[data-testid="tweet"]');
      const results = [];
      const seen = new Set();
      for (const a of articles) {{
        const t = _xcliBuildSummary(a);
        if (!t.id) continue;
        if (seen.has(t.id)) continue;
        seen.add(t.id);
        results.push(t);
      }}
      return results;
    }})()
    "#,
        helpers = tweet_helpers_js()
    )
}

fn profile_ready_script() -> &'static str {
    r#"
    (() => {
      const hasName = !!document.querySelector('[data-testid="UserName"], [data-testid="UserProfileHeader_Items"]');
      const hasTimeline = !!document.querySelector('article[data-testid="tweet"]');
      const empty = !!document.querySelector('[data-testid="emptyState"]');
      return hasName || hasTimeline || empty;
    })()
    "#
}

fn profile_extract_script() -> String {
    format!(
        r#"
    (() => {{
      {helpers}

      function extractUser() {{
        const user = {{
          handle: '', display_name: '', bio: '', avatar: '', banner: '',
          location: '', website: '', joined: '',
          following: '', followers: '',
          verified: false, url: location.origin + location.pathname,
        }};

        const handleMatch = location.pathname.match(/^\/([A-Za-z0-9_]{{1,15}})/);
        if (handleMatch) user.handle = handleMatch[1];

        const nameBlock = document.querySelector('[data-testid="UserName"]');
        if (nameBlock) {{
          const spans = nameBlock.querySelectorAll('span');
          for (const s of spans) {{
            const t = (s.innerText || '').trim();
            if (!t) continue;
            if (t.startsWith('@')) {{
              if (!user.handle) user.handle = t.slice(1);
            }} else if (!user.display_name && t !== '·') {{
              user.display_name = t;
            }}
          }}
          if (nameBlock.querySelector('svg[aria-label*="Verified"], svg[data-testid="icon-verified"]')) {{
            user.verified = true;
          }}
        }}

        const bio = document.querySelector('[data-testid="UserDescription"]');
        if (bio) user.bio = (bio.innerText || '').trim();

        const avatar = document.querySelector('a[href$="/photo"] img, [data-testid^="UserAvatar"] img');
        if (avatar) user.avatar = avatar.src || '';
        const banner = document.querySelector('a[href$="/header_photo"] img');
        if (banner) user.banner = banner.src || '';

        const items = document.querySelector('[data-testid="UserProfileHeader_Items"]');
        if (items) {{
          const loc = items.querySelector('[data-testid="UserLocation"]');
          if (loc) user.location = (loc.innerText || '').trim();
          const url = items.querySelector('[data-testid="UserUrl"]');
          if (url) user.website = url.getAttribute('href') || (url.innerText || '').trim();
          const joined = items.querySelector('[data-testid="UserJoinDate"]');
          if (joined) user.joined = (joined.innerText || '').trim();
        }}

        for (const a of document.querySelectorAll('a[href$="/following"], a[href$="/verified_followers"], a[href$="/followers"]')) {{
          const href = a.getAttribute('href') || '';
          const txt = (a.innerText || '').replace(/\s+/g, ' ').trim();
          const m = txt.match(/([\d,\.]+\s*[KMB]?)/i);
          const num = m ? m[1].replace(/\s+/g, '') : '';
          if (href.endsWith('/following') && !user.following) user.following = num;
          else if ((href.endsWith('/followers') || href.endsWith('/verified_followers')) && !user.followers) user.followers = num;
        }}

        return user;
      }}

      const user = extractUser();

      const articles = document.querySelectorAll('article[data-testid="tweet"]');
      const tweets = [];
      const seen = new Set();
      for (const a of articles) {{
        const t = _xcliBuildSummary(a);
        if (!t.id) continue;
        if (seen.has(t.id)) continue;
        seen.add(t.id);
        tweets.push(t);
      }}

      return {{ user, tweets }};
    }})()
    "#,
        helpers = tweet_helpers_js()
    )
}

fn post_ready_script() -> &'static str {
    r#"
    (() => {
      const article = document.querySelector('article[data-testid="tweet"]');
      if (!article) return false;
      // Tweet detail body usually appears within the first second.
      const hasText = !!article.querySelector('[data-testid="tweetText"]');
      const hasMedia = !!article.querySelector('[data-testid="tweetPhoto"], video');
      return hasText || hasMedia;
    })()
    "#
}

fn post_extract_script() -> String {
    format!(
        r#"
    (() => {{
      {helpers}
      const article = document.querySelector('article[data-testid="tweet"]');
      if (!article) return null;

      const base = _xcliBuildSummary(article);

      // Tweet detail page exposes additional aria-labels on the actions row.
      // Quote count is NOT reachable from a `data-testid` on the action row
      // (the retweet button covers both reposts and quotes); instead Twitter
      // renders a clickable `<a href=".../quotes">` summary above/below the
      // actions. Pull the visible count from there.
      let quotes = '';
      const quoteAnchor = article.querySelector(
        'a[href$="/quotes"], a[href*="/quotes?"]'
      );
      if (quoteAnchor) {{
        const text = (quoteAnchor.innerText || quoteAnchor.textContent || '').trim();
        const m = text.match(/([\d,\.]+\s*[KMB]?)/i);
        if (m) quotes = m[1].replace(/\s+/g, '');
      }}
      const bookmarks = _xcliExtractCount(article, 'removeBookmark') ||
                        _xcliExtractCount(article, 'bookmark');

      // Quoted tweet: an inner article without `[data-testid="tweet"]` but
      // with role="link" wrapping a sub-tweet. Best-effort extraction.
      let quoted = null;
      const inner = article.querySelector('div[role="link"][tabindex] article, div[role="link"] article');
      if (inner && inner !== article) {{
        const q = _xcliBuildSummary(inner);
        if (q.id && q.id !== base.id) {{
          quoted = {{
            ...q,
            quotes: '',
            bookmarks: '',
            quoted: null,
          }};
        }}
      }}

      // Force the URL to the canonical detail page when possible.
      let url = base.url;
      if (!url) {{
        url = location.origin + location.pathname;
      }}

      return {{
        id: base.id,
        url,
        author: base.author,
        handle: base.handle,
        text: base.text,
        time: base.time,
        replies: base.replies,
        retweets: base.retweets,
        quotes,
        likes: base.likes,
        bookmarks,
        views: base.views,
        images: base.images,
        videos: base.videos,
        links: base.links,
        quoted,
      }};
    }})()
    "#,
        helpers = tweet_helpers_js()
    )
}

fn replies_ready_script() -> &'static str {
    r#"
    (() => {
      const arts = document.querySelectorAll('article[data-testid="tweet"]');
      // Happy path: root tweet plus at least one reply rendered.
      if (arts.length >= 2) return true;
      // Tweets with no replies render an explicit empty-state hook.
      if (document.querySelector('[data-testid="emptyState"]')) return true;
      // Fallback: root tweet is up and the loading spinner has gone, which
      // means either no replies exist or replies failed to load -- we accept
      // and let the extractor return an empty list rather than time out.
      const progress = document.querySelector('[role="progressbar"]');
      if (arts.length >= 1 && !progress) return true;
      return false;
    })()
    "#
}

fn replies_extract_script() -> String {
    format!(
        r#"
    (() => {{
      {helpers}
      const articles = document.querySelectorAll('article[data-testid="tweet"]');
      const out = [];
      const seen = new Set();
      const pathId = (location.pathname.match(/\/status\/(\d+)/) || [])[1] || '';
      let skippedOriginal = false;
      for (const a of articles) {{
        const t = _xcliBuildSummary(a);
        if (!t.id) continue;
        // Skip the first occurrence whose id matches the URL (the root tweet).
        if (!skippedOriginal && pathId && t.id === pathId) {{
          skippedOriginal = true;
          continue;
        }}
        if (seen.has(t.id)) continue;
        seen.add(t.id);
        out.push({{
          id: t.id,
          url: t.url,
          author: t.author,
          handle: t.handle,
          text: t.text,
          time: t.time,
          replies: t.replies,
          retweets: t.retweets,
          likes: t.likes,
          images: t.images,
          videos: t.videos,
        }});
      }}
      return out;
    }})()
    "#,
        helpers = tweet_helpers_js()
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, sync::Mutex};

    use async_trait::async_trait;
    use serde::de::DeserializeOwned;
    use serde_json::json;
    use xcli_webbridge::BridgeStatus;

    use super::*;

    #[test]
    fn builds_search_url_with_mode() {
        assert_eq!(
            search_url("rust cli", "live"),
            "https://x.com/search?q=rust%20cli&src=typed_query&f=live"
        );
        assert_eq!(
            search_url("rust", "top"),
            "https://x.com/search?q=rust&src=typed_query"
        );
    }

    #[test]
    fn builds_profile_url() {
        assert_eq!(profile_url("@elonmusk"), "https://x.com/elonmusk");
        assert_eq!(profile_url("elonmusk"), "https://x.com/elonmusk");
    }

    #[test]
    fn builds_post_url() {
        assert_eq!(
            post_url("elonmusk", "1234567890"),
            "https://x.com/elonmusk/status/1234567890"
        );
        assert_eq!(
            post_url_by_id("1234567890"),
            "https://x.com/i/web/status/1234567890"
        );
    }

    #[test]
    fn resolve_post_url_accepts_many_shapes() {
        assert_eq!(
            resolve_post_url("https://x.com/foo/status/12345").unwrap(),
            "https://x.com/foo/status/12345"
        );
        assert_eq!(
            resolve_post_url("foo/status/12345").unwrap(),
            "https://x.com/foo/status/12345"
        );
        assert_eq!(
            resolve_post_url("foo/12345").unwrap(),
            "https://x.com/foo/status/12345"
        );
        assert_eq!(
            resolve_post_url("12345").unwrap(),
            "https://x.com/i/web/status/12345"
        );
        assert_eq!(
            resolve_post_url("x.com/foo/status/12345").unwrap(),
            "https://x.com/foo/status/12345"
        );
    }

    #[test]
    fn resolve_post_url_rejects_garbage() {
        let err = resolve_post_url("").unwrap_err();
        assert_eq!(err.code(), "missing_args");
    }

    #[test]
    fn extract_tweet_id_from_url() {
        assert_eq!(
            extract_tweet_id("https://x.com/foo/status/12345"),
            Some("12345".to_string())
        );
        assert_eq!(
            extract_tweet_id("https://x.com/i/web/status/9876?ref=abc"),
            Some("9876".to_string())
        );
        assert_eq!(extract_tweet_id("https://x.com/foo"), None);
    }

    #[test]
    fn normalize_mode_accepts_aliases() {
        assert_eq!(normalize_mode("Latest"), "live");
        assert_eq!(normalize_mode("People"), "user");
        assert_eq!(normalize_mode(""), "top");
        assert_eq!(normalize_mode("garbage"), "top");
    }

    #[tokio::test]
    async fn search_returns_parsed_tweets() {
        let bridge = MockBridge::new(vec![
            json!(true),
            json!([
                {
                    "id": "111",
                    "url": "https://x.com/alice/status/111",
                    "author": "Alice",
                    "handle": "alice",
                    "text": "hello world",
                    "time": "2026-05-19T12:00:00Z",
                    "replies": "5",
                    "retweets": "10",
                    "likes": "100",
                    "views": "1.2K",
                    "images": ["https://pbs.twimg.com/media/x.jpg"],
                    "videos": [],
                    "links": ["https://example.com"]
                },
                {
                    "id": "222",
                    "url": "https://x.com/bob/status/222",
                    "author": "Bob",
                    "handle": "bob",
                    "text": "rust",
                    "time": "2026-05-19T11:00:00Z",
                    "replies": "1",
                    "retweets": "2",
                    "likes": "30",
                    "views": "500",
                    "images": [],
                    "videos": [],
                    "links": []
                }
            ]),
        ]);
        let browser = Browser::new(bridge);

        let out = search(
            &browser,
            SearchOptions {
                query: "rust".to_string(),
                limit: 1,
                mode: "live".to_string(),
            },
        )
        .await
        .unwrap();

        assert_eq!(out.query, "rust");
        assert_eq!(out.mode, "live");
        assert_eq!(out.count, 1);
        assert_eq!(out.tweets[0].id, "111");
    }

    #[tokio::test]
    async fn search_rejects_empty_query() {
        let bridge = MockBridge::new(vec![]);
        let browser = Browser::new(bridge);

        let err = search(
            &browser,
            SearchOptions {
                query: "  ".to_string(),
                limit: 10,
                mode: "top".to_string(),
            },
        )
        .await
        .unwrap_err();

        assert_eq!(err.code(), "missing_args");
    }

    #[tokio::test]
    async fn profile_returns_user_and_tweets() {
        let bridge = MockBridge::new(vec![
            json!(true),
            json!({
                "user": {
                    "handle": "alice",
                    "display_name": "Alice",
                    "bio": "Rustacean",
                    "avatar": "https://x.example/a.jpg",
                    "banner": "",
                    "location": "Earth",
                    "website": "https://example.com",
                    "joined": "Joined June 2020",
                    "following": "100",
                    "followers": "10K",
                    "verified": false,
                    "url": "https://x.com/alice"
                },
                "tweets": [
                    {
                        "id": "111",
                        "url": "https://x.com/alice/status/111",
                        "author": "Alice",
                        "handle": "alice",
                        "text": "hi",
                        "time": "2026-05-19T12:00:00Z",
                        "replies": "1",
                        "retweets": "2",
                        "likes": "10",
                        "views": "100",
                        "images": [],
                        "videos": [],
                        "links": []
                    }
                ]
            }),
        ]);
        let browser = Browser::new(bridge);

        let out = profile(
            &browser,
            ProfileOptions {
                handle: "@alice".to_string(),
                limit: 10,
            },
        )
        .await
        .unwrap();

        assert_eq!(out.user.handle, "alice");
        assert_eq!(out.user.display_name, "Alice");
        assert_eq!(out.tweets.len(), 1);
    }

    #[tokio::test]
    async fn post_returns_detail() {
        let bridge = MockBridge::new(vec![
            json!(true),
            json!({
                "id": "111",
                "url": "https://x.com/alice/status/111",
                "author": "Alice",
                "handle": "alice",
                "text": "post body",
                "time": "2026-05-19T12:00:00Z",
                "replies": "1",
                "retweets": "2",
                "quotes": "0",
                "likes": "10",
                "bookmarks": "3",
                "views": "100",
                "images": ["https://pbs.twimg.com/media/x.jpg"],
                "videos": ["https://video.twimg.com/x.mp4"],
                "links": ["https://example.com"],
                "quoted": null
            }),
        ]);
        let browser = Browser::new(bridge);

        let out = post(&browser, PostOptions::new("alice/status/111"))
            .await
            .unwrap();

        assert_eq!(out.id, "111");
        assert_eq!(out.images.len(), 1);
        assert_eq!(out.videos.len(), 1);
        assert!(
            out.downloads.is_none(),
            "downloads should be absent when out_dir is unset"
        );
    }

    #[test]
    fn guess_extension_handles_common_urls() {
        assert_eq!(
            guess_extension("https://pbs.twimg.com/media/a.jpg", "x"),
            "jpg"
        );
        assert_eq!(
            guess_extension("https://pbs.twimg.com/media/a.png?name=orig", "x"),
            "png"
        );
        assert_eq!(
            guess_extension("https://video.twimg.com/x/720x1280/abc.mp4", "x"),
            "mp4"
        );
        // ?format= query parameter takes precedence over a missing path ext.
        assert_eq!(
            guess_extension("https://pbs.twimg.com/media/abc?format=jpg&name=large", "x"),
            "jpg"
        );
        // ?format= also wins over a misleading path segment.
        assert_eq!(
            guess_extension(
                "https://pbs.twimg.com/media/abc.bin?format=png&name=orig",
                "x"
            ),
            "png"
        );
        // No extension: fall back to default.
        assert_eq!(
            guess_extension("https://x.com/foo/status/111", "mp4"),
            "mp4"
        );
        // Garbage trailing dot segment longer than 5 chars: fall back.
        assert_eq!(guess_extension("https://x.com/a.longext", "x"), "x");
    }

    #[test]
    fn download_report_skips_serializing_when_no_downloads() {
        let mut detail = sample_detail();
        detail.downloads = None;
        let json = serde_json::to_value(&detail).unwrap();
        assert!(
            !json.as_object().unwrap().contains_key("downloads"),
            "downloads field must be omitted when None"
        );
    }

    #[test]
    fn download_report_round_trips_when_present() {
        let mut detail = sample_detail();
        detail.downloads = Some(DownloadReport {
            out_dir: "/tmp/out".to_string(),
            images: vec![DownloadedAsset {
                url: "https://pbs.twimg.com/media/a.jpg".to_string(),
                path: "/tmp/out/111-image-01.jpg".to_string(),
                bytes: 1024,
            }],
            videos: vec![],
            errors: vec![],
        });
        let json = serde_json::to_value(&detail).unwrap();
        let parsed: PostDetail = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, detail);
    }

    fn sample_detail() -> PostDetail {
        PostDetail {
            id: "111".to_string(),
            url: "https://x.com/alice/status/111".to_string(),
            author: "Alice".to_string(),
            handle: "alice".to_string(),
            text: "hi".to_string(),
            time: "2026-05-19T12:00:00Z".to_string(),
            replies: "0".to_string(),
            retweets: "0".to_string(),
            quotes: "0".to_string(),
            likes: "0".to_string(),
            bookmarks: "0".to_string(),
            views: "0".to_string(),
            images: vec![],
            videos: vec![],
            links: vec![],
            quoted: None,
            downloads: None,
        }
    }

    #[tokio::test]
    async fn fetch_to_file_writes_bytes_to_disk() {
        // Spin up a tiny HTTP server using tokio's TcpListener so we don't
        // depend on the network during unit tests.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let body = b"hello-tweet-media";
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: image/jpeg\r\n\r\n",
                    body.len()
                );
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.write_all(body).await;
                let _ = socket.shutdown().await;
            }
        });

        let dir = tempfile::tempdir().unwrap();
        let client = reqwest::Client::builder()
            .user_agent(DOWNLOAD_USER_AGENT)
            .build()
            .unwrap();

        let url = format!("http://{}/img.jpg", addr);
        let asset = fetch_to_file(&client, &url, dir.path(), "111-image-01.jpg")
            .await
            .unwrap();

        assert_eq!(asset.bytes, 17);
        assert_eq!(asset.url, url);
        let written = std::fs::read(dir.path().join("111-image-01.jpg")).unwrap();
        assert_eq!(written, b"hello-tweet-media");
    }

    #[tokio::test]
    async fn replies_returns_list_and_skips_root() {
        let bridge = MockBridge::new(vec![
            json!(true),
            json!([
                {
                    "id": "222",
                    "url": "https://x.com/bob/status/222",
                    "author": "Bob",
                    "handle": "bob",
                    "text": "reply",
                    "time": "2026-05-19T13:00:00Z",
                    "replies": "0",
                    "retweets": "0",
                    "likes": "1",
                    "images": [],
                    "videos": []
                }
            ]),
        ]);
        let browser = Browser::new(bridge);

        let out = replies(
            &browser,
            RepliesOptions {
                reference: "alice/status/111".to_string(),
                limit: 5,
            },
        )
        .await
        .unwrap();

        assert_eq!(out.tweet_id, "111");
        assert_eq!(out.replies.len(), 1);
        assert_eq!(out.replies[0].id, "222");
    }

    struct MockBridge {
        values: Mutex<VecDeque<serde_json::Value>>,
    }

    impl MockBridge {
        fn new(values: Vec<serde_json::Value>) -> Self {
            Self {
                values: Mutex::new(values.into()),
            }
        }
    }

    #[async_trait]
    impl BrowserBridge for MockBridge {
        async fn status(&self) -> Result<BridgeStatus> {
            Ok(BridgeStatus {
                running: true,
                extension_connected: true,
                extension_version: Some("test".to_string()),
                version: Some("test".to_string()),
            })
        }

        async fn navigate(&self, _url: &str) -> Result<()> {
            Ok(())
        }

        async fn eval<T>(&self, _javascript: &str) -> Result<T>
        where
            T: DeserializeOwned + Send,
        {
            let value = self
                .values
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| XCliError::BrowserActionFailed("mock exhausted".to_string()))?;
            serde_json::from_value(value)
                .map_err(|err| XCliError::BrowserActionFailed(err.to_string()))
        }
    }
}
