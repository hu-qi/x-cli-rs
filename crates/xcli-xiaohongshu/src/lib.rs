use std::time::Duration;

use serde::{Deserialize, Serialize};
use tracing::info;
use xcli_browser::Browser;
use xcli_core::{Result, XCliError};
use xcli_webbridge::BrowserBridge;

// ---------------------------------------------------------------------------
// Data models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub keyword: String,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoteItem {
    pub id: String,
    pub title: String,
    pub desc: String,
    pub author: String,
    pub author_id: String,
    pub likes: String,
    pub cover: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchOutput {
    pub keyword: String,
    pub count: usize,
    pub notes: Vec<NoteItem>,
}

#[derive(Debug, Clone)]
pub struct ProfileOptions {
    pub user_id: String,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct UserInfo {
    pub nickname: String,
    pub user_id: String,
    pub avatar: String,
    pub bio: String,
    pub followers: String,
    pub following: String,
    pub notes_count: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileOutput {
    pub user: UserInfo,
    pub notes: Vec<NoteItem>,
}

#[derive(Debug, Clone)]
pub struct NoteOptions {
    pub note_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoteDetail {
    pub id: String,
    pub title: String,
    pub content: String,
    pub author: String,
    pub author_id: String,
    pub likes: String,
    pub collects: String,
    pub comments_count: String,
    pub images: Vec<String>,
    pub url: String,
    pub publish_time: String,
}

#[derive(Debug, Clone)]
pub struct CommentOptions {
    pub note_id: String,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommentItem {
    pub id: String,
    pub user: String,
    pub user_id: String,
    pub avatar: String,
    pub content: String,
    pub likes: String,
    pub time: String,
    pub replies: Vec<CommentItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommentsOutput {
    pub note_id: String,
    pub count: usize,
    pub comments: Vec<CommentItem>,
}

// ---------------------------------------------------------------------------
// Search notes
// ---------------------------------------------------------------------------

pub async fn search<B>(browser: &Browser<B>, options: SearchOptions) -> Result<SearchOutput>
where
    B: BrowserBridge,
{
    if options.keyword.trim().is_empty() {
        return Err(XCliError::MissingArgs(
            "search requires a keyword: xiaohongshu-cli search <keyword>".to_string(),
        ));
    }

    let limit = normalize_limit(options.limit);
    let url = search_url(&options.keyword);

    info!(step = "navigate", url = %url, "opening Xiaohongshu search");
    browser.goto(&url).await.map_err(map_error)?;

    // Wait for note cards to appear (SPA hydration).
    info!(step = "wait", "waiting for search results to render");
    browser
        .wait_for_js_truthy(search_ready_script(), Duration::from_secs(15))
        .await
        .map_err(|err| wait_error("search results", err))?;

    info!(step = "extract", limit, "extracting note list from DOM");
    let mut notes: Vec<NoteItem> = browser
        .eval(search_extract_script())
        .await
        .map_err(map_error)?;

    if notes.len() > limit {
        notes.truncate(limit);
    }

    if notes.is_empty() {
        return Err(XCliError::NoResults(
            "xiaohongshu returned no parseable results (selectors may have drifted or login is required)".to_string(),
        ));
    }

    Ok(SearchOutput {
        keyword: options.keyword,
        count: notes.len(),
        notes,
    })
}

// ---------------------------------------------------------------------------
// User profile
// ---------------------------------------------------------------------------

pub async fn profile<B>(browser: &Browser<B>, options: ProfileOptions) -> Result<ProfileOutput>
where
    B: BrowserBridge,
{
    if options.user_id.trim().is_empty() {
        return Err(XCliError::MissingArgs(
            "profile requires a user_id: xiaohongshu-cli profile <user_id>".to_string(),
        ));
    }

    let limit = normalize_limit(options.limit);
    let url = profile_url(&options.user_id);

    info!(step = "navigate", url = %url, "opening Xiaohongshu user profile");
    browser.goto(&url).await.map_err(map_error)?;

    info!(step = "wait", "waiting for profile to render");
    browser
        .wait_for_js_truthy(profile_ready_script(), Duration::from_secs(15))
        .await
        .map_err(|err| wait_error("profile", err))?;

    info!(step = "extract", "extracting user info and notes");
    let payload: ProfilePayload = browser
        .eval(profile_extract_script())
        .await
        .map_err(map_error)?;

    let mut notes = payload.notes;
    if notes.len() > limit {
        notes.truncate(limit);
    }

    Ok(ProfileOutput {
        user: payload.user,
        notes,
    })
}

// ---------------------------------------------------------------------------
// Note detail
// ---------------------------------------------------------------------------

pub async fn note<B>(browser: &Browser<B>, options: NoteOptions) -> Result<NoteDetail>
where
    B: BrowserBridge,
{
    if options.note_id.trim().is_empty() {
        return Err(XCliError::MissingArgs(
            "note requires a note_id: xiaohongshu-cli note <note_id>".to_string(),
        ));
    }

    let url = note_url(&options.note_id);

    info!(step = "navigate", url = %url, "opening Xiaohongshu note detail");
    browser.goto(&url).await.map_err(map_error)?;

    info!(step = "wait", "waiting for note detail to render");
    browser
        .wait_for_js_truthy(note_ready_script(), Duration::from_secs(15))
        .await
        .map_err(|err| wait_error("note", err))?;

    info!(step = "extract", "extracting note detail");
    let detail: NoteDetail = browser
        .eval(note_extract_script())
        .await
        .map_err(map_error)?;

    Ok(detail)
}

// ---------------------------------------------------------------------------
// Comments
// ---------------------------------------------------------------------------

pub async fn comments<B>(browser: &Browser<B>, options: CommentOptions) -> Result<CommentsOutput>
where
    B: BrowserBridge,
{
    if options.note_id.trim().is_empty() {
        return Err(XCliError::MissingArgs(
            "comments requires a note_id: xiaohongshu-cli comments <note_id>".to_string(),
        ));
    }

    let limit = normalize_limit(options.limit);
    let url = note_url(&options.note_id);

    info!(step = "navigate", url = %url, "opening Xiaohongshu note for comments");
    browser.goto(&url).await.map_err(map_error)?;

    info!(step = "wait", "waiting for comments to render");
    browser
        .wait_for_js_truthy(comments_ready_script(), Duration::from_secs(15))
        .await
        .map_err(|err| wait_error("comments", err))?;

    info!(step = "extract", limit, "extracting comments");
    let mut items: Vec<CommentItem> = browser
        .eval(comments_extract_script())
        .await
        .map_err(map_error)?;

    if items.len() > limit {
        items.truncate(limit);
    }

    Ok(CommentsOutput {
        note_id: options.note_id,
        count: items.len(),
        comments: items,
    })
}

// ---------------------------------------------------------------------------
// URL helpers
// ---------------------------------------------------------------------------

pub fn search_url(keyword: &str) -> String {
    format!(
        "https://www.xiaohongshu.com/search_result?keyword={}&source=web_explore",
        urlencoding::encode(keyword)
    )
}

pub fn profile_url(user_id: &str) -> String {
    format!("https://www.xiaohongshu.com/user/profile/{}", user_id)
}

pub fn note_url(note_id: &str) -> String {
    format!("https://www.xiaohongshu.com/explore/{}", note_id)
}

fn normalize_limit(limit: usize) -> usize {
    if limit == 0 {
        10
    } else {
        limit
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
            "xiaohongshu {} did not load within timeout: {}",
            what, other
        )),
    }
}

// ---------------------------------------------------------------------------
// JS extractor scripts
//
// Xiaohongshu uses hashed CSS class names, so we rely on:
//   - link href patterns (/explore/xxx, /user/profile/xxx)
//   - DOM tree structure (image + title + author + interaction counts)
//   - aria-labels or alt texts when available
// ---------------------------------------------------------------------------

fn search_ready_script() -> &'static str {
    r#"
    (() => {
      const links = document.querySelectorAll('a[href*="/explore/"]');
      return links.length > 0;
    })()
    "#
}

fn search_extract_script() -> &'static str {
    r#"
    (() => {
      const seen = new Set();
      const results = [];
      const links = document.querySelectorAll('a[href*="/explore/"]');
      for (const a of links) {
        const href = a.getAttribute('href') || '';
        const match = href.match(/\/explore\/([a-zA-Z0-9]+)/);
        if (!match) continue;
        const id = match[1];
        if (seen.has(id)) continue;
        seen.add(id);

        // Walk up to find the card container
        let card = a;
        for (let i = 0; i < 6; i++) {
          if (!card.parentElement) break;
          card = card.parentElement;
        }

        // Title: try nearby heading or the link text
        let title = '';
        const h3 = card.querySelector('h3');
        if (h3) {
          title = h3.innerText.trim();
        } else {
          title = a.innerText.trim().slice(0, 80);
        }

        // Description: any nearby paragraph-like text
        let desc = '';
        const p = card.querySelector('p, [class*="desc"]');
        if (p && p !== a) {
          desc = p.innerText.trim().slice(0, 200);
        }

        // Author: look for user profile link
        let author = '';
        let author_id = '';
        const userLink = card.querySelector('a[href*="/user/profile/"]');
        if (userLink) {
          author = userLink.innerText.trim();
          const um = userLink.getAttribute('href').match(/\/user\/profile\/([a-zA-Z0-9]+)/);
          if (um) author_id = um[1];
        }

        // Likes: look for numbers near heart/like icons
        let likes = '';
        const nums = card.querySelectorAll('span, div');
        for (const n of nums) {
          const t = n.innerText.trim();
          if (/^\d+[\d\.]*[kw]?$/i.test(t) && t.length < 10) {
            likes = t;
            break;
          }
        }

        // Cover image: first img inside the card
        let cover = '';
        const img = card.querySelector('img');
        if (img) cover = img.src || img.getAttribute('data-src') || '';

        results.push({
          id,
          title,
          desc,
          author,
          author_id,
          likes,
          cover,
          url: 'https://www.xiaohongshu.com' + href
        });
      }
      return results;
    })()
    "#
}

fn profile_ready_script() -> &'static str {
    r#"
    (() => {
      // Wait for either user nickname or note cards
      const hasName = !!document.querySelector('h1, [class*="nickname"], [class*="user-name"]');
      const hasNotes = document.querySelectorAll('a[href*="/explore/"]').length > 0;
      return hasName || hasNotes;
    })()
    "#
}

#[derive(Debug, Clone, Deserialize)]
struct ProfilePayload {
    #[serde(default)]
    user: UserInfo,
    #[serde(default)]
    notes: Vec<NoteItem>,
}

fn profile_extract_script() -> &'static str {
    r#"
    (() => {
      // ----- user info -----
      let nickname = '';
      let avatar = '';
      let bio = '';
      let followers = '';
      let following = '';
      let notes_count = '';

      // Try common patterns for nickname
      const h1 = document.querySelector('h1');
      if (h1) nickname = h1.innerText.trim();

      // Avatar: first large square image near top
      const imgs = document.querySelectorAll('img');
      for (const img of imgs) {
        const src = img.src || '';
        if (src.includes('avatar') || src.includes('user')) {
          avatar = src;
          break;
        }
        if (!avatar && (img.width > 48 || img.height > 48)) {
          avatar = src;
        }
      }

      // Bio: longer text block near top
      const texts = document.querySelectorAll('div, span, p');
      for (const t of texts) {
        const txt = t.innerText.trim();
        if (txt.length > 10 && txt.length < 300 && t.children.length <= 2) {
          bio = txt;
          break;
        }
      }

      // Stats: look for labels like "粉丝", "关注", "获赞与收藏".
      // Scope the search to the profile header area when possible to avoid
      // iterating every element on the page. We collect a small candidate
      // set from likely containers and only fall back to a bounded broader
      // query if nothing matched.
      let statsRoot =
        document.querySelector('[class*="user-info"], [class*="UserInfo"], [class*="profile"], header') ||
        document.body;
      let all = statsRoot.querySelectorAll('div, span, a');
      if (all.length < 4) {
        all = document.querySelectorAll('div, span, a');
      }
      for (const el of all) {
        const txt = el.innerText.trim();
        if (txt.includes('粉丝') && !followers) {
          const parent = el.parentElement;
          if (parent) {
            const sib = parent.querySelector('div, span');
            if (sib && sib !== el) followers = sib.innerText.trim();
          }
        }
        if (txt.includes('关注') && !following) {
          const parent = el.parentElement;
          if (parent) {
            const sib = parent.querySelector('div, span');
            if (sib && sib !== el) following = sib.innerText.trim();
          }
        }
        if ((txt.includes('笔记') || txt.includes('作品')) && !notes_count) {
          const parent = el.parentElement;
          if (parent) {
            const sib = parent.querySelector('div, span');
            if (sib && sib !== el) notes_count = sib.innerText.trim();
          }
        }
      }

      // Extract user_id from URL
      let user_id = '';
      const um = location.pathname.match(/\/user\/profile\/([a-zA-Z0-9]+)/);
      if (um) user_id = um[1];

      // ----- notes (same logic as search) -----
      const seen = new Set();
      const notes = [];
      const links = document.querySelectorAll('a[href*="/explore/"]');
      for (const a of links) {
        const href = a.getAttribute('href') || '';
        const match = href.match(/\/explore\/([a-zA-Z0-9]+)/);
        if (!match) continue;
        const id = match[1];
        if (seen.has(id)) continue;
        seen.add(id);

        let card = a;
        for (let i = 0; i < 6; i++) {
          if (!card.parentElement) break;
          card = card.parentElement;
        }

        let title = '';
        const h3 = card.querySelector('h3');
        if (h3) title = h3.innerText.trim();
        else title = a.innerText.trim().slice(0, 80);

        let desc = '';
        const p = card.querySelector('p, [class*="desc"]');
        if (p && p !== a) desc = p.innerText.trim().slice(0, 200);

        let author = nickname || '';
        let author_id = user_id || '';

        let likes = '';
        const nums = card.querySelectorAll('span, div');
        for (const n of nums) {
          const t = n.innerText.trim();
          if (/^\d+[\d\.]*[kw]?$/i.test(t) && t.length < 10) {
            likes = t;
            break;
          }
        }

        let cover = '';
        const img = card.querySelector('img');
        if (img) cover = img.src || img.getAttribute('data-src') || '';

        notes.push({
          id,
          title,
          desc,
          author,
          author_id,
          likes,
          cover,
          url: 'https://www.xiaohongshu.com' + href
        });
      }

      return {
        user: {
          nickname,
          user_id,
          avatar,
          bio,
          followers,
          following,
          notes_count
        },
        notes
      };
    })()
    "#
}

fn note_ready_script() -> &'static str {
    r#"
    (() => {
      // Wait for title or content to appear
      const hasTitle = !!document.querySelector('h1, [class*="title"]');
      const hasContent = !!document.querySelector('p, [class*="content"], [class*="desc"]');
      return hasTitle || hasContent;
    })()
    "#
}

fn note_extract_script() -> &'static str {
    r#"
    (() => {
      let title = '';
      let content = '';
      let author = '';
      let author_id = '';
      let likes = '';
      let collects = '';
      let comments_count = '';
      let publish_time = '';
      const images = [];

      // Title
      const h1 = document.querySelector('h1');
      if (h1) title = h1.innerText.trim();

      // Content: longest text block that looks like article body
      const textBlocks = document.querySelectorAll('p, div, span');
      let bestContent = '';
      for (const el of textBlocks) {
        const txt = el.innerText.trim();
        if (txt.length > bestContent.length && txt.length < 5000 && el.children.length <= 5) {
          bestContent = txt;
        }
      }
      content = bestContent.slice(0, 2000);

      // Author from user link
      const userLink = document.querySelector('a[href*="/user/profile/"]');
      if (userLink) {
        author = userLink.innerText.trim();
        const um = userLink.getAttribute('href').match(/\/user\/profile\/([a-zA-Z0-9]+)/);
        if (um) author_id = um[1];
      }

      // Extract note_id from URL
      let note_id = '';
      const nm = location.pathname.match(/\/explore\/([a-zA-Z0-9]+)/);
      if (nm) note_id = nm[1];

      // Interaction counts: look for numbers near icons or specific labels
      const all = document.querySelectorAll('div, span');
      for (const el of all) {
        const txt = el.innerText.trim();
        if (txt.includes('点赞') || txt.includes('喜欢') || txt.includes('♥') || txt.includes('❤')) {
          const parent = el.parentElement;
          if (parent) {
            const sib = parent.querySelector('span, div');
            if (sib && sib !== el && /^\d/.test(sib.innerText.trim())) {
              likes = sib.innerText.trim();
            }
          }
        }
        if (txt.includes('收藏') || txt.includes('⭐') || txt.includes('☆')) {
          const parent = el.parentElement;
          if (parent) {
            const sib = parent.querySelector('span, div');
            if (sib && sib !== el && /^\d/.test(sib.innerText.trim())) {
              collects = sib.innerText.trim();
            }
          }
        }
        if (txt.includes('评论') || txt.includes('💬')) {
          const parent = el.parentElement;
          if (parent) {
            const sib = parent.querySelector('span, div');
            if (sib && sib !== el && /^\d/.test(sib.innerText.trim())) {
              comments_count = sib.innerText.trim();
            }
          }
        }
      }

      // Fallback: look for raw numbers near common interaction areas
      if (!likes || !collects) {
        for (const el of all) {
          const t = el.innerText.trim();
          if (/^\d+[\d\.]*[kw]?$/i.test(t) && t.length < 10) {
            if (!likes) likes = t;
            else if (!collects) collects = t;
            else if (!comments_count) comments_count = t;
          }
        }
      }

      // Publish time: look for date patterns
      for (const el of all) {
        const t = el.innerText.trim();
        if (/\d{4}-\d{2}-\d{2}/.test(t) || /\d{2}-\d{2}/.test(t) || t.includes('前') || t.includes('天')) {
          publish_time = t;
          break;
        }
      }

      // Images: all images that look like note content
      const imgs = document.querySelectorAll('img');
      for (const img of imgs) {
        const src = img.src || img.getAttribute('data-src') || '';
        if (src && (src.includes('xiaohongshu') || src.includes('sns'))) {
          images.push(src);
        }
      }

      return {
        id: note_id,
        title,
        content,
        author,
        author_id,
        likes,
        collects,
        comments_count,
        images: images.slice(0, 20),
        url: location.href,
        publish_time
      };
    })()
    "#
}

fn comments_ready_script() -> &'static str {
    r#"
    (() => {
      // Comments may take a moment; look for comment-like structures
      const hasComments = document.querySelectorAll('div, li, section').length > 20;
      return hasComments;
    })()
    "#
}

fn comments_extract_script() -> &'static str {
    r#"
    (() => {
      const results = [];

      // Helper: derive comment body text from a dedicated DOM node when
      // possible (more robust than string-stripping the user name). Falls
      // back to the element's text minus the user-name node's text.
      function extractBody(el, userEl) {
        const bodyEl =
          el.querySelector('[class*="content"], [class*="Content"], [class*="text"], [class*="Text"]');
        if (bodyEl && bodyEl !== userEl && !bodyEl.contains(userEl)) {
          return bodyEl.innerText.trim().replace(/^[:：\s]+/, '');
        }
        // Build text by walking direct children and skipping the user-name
        // subtree, so a username substring inside the body is preserved.
        const parts = [];
        for (const child of el.childNodes) {
          if (child.nodeType === Node.TEXT_NODE) {
            const t = child.textContent.trim();
            if (t) parts.push(t);
          } else if (child.nodeType === Node.ELEMENT_NODE) {
            if (child === userEl || child.contains(userEl)) continue;
            const t = child.innerText ? child.innerText.trim() : '';
            if (t) parts.push(t);
          }
        }
        return parts.join(' ').replace(/^[:：\s]+/, '');
      }

      // Heuristic: scope to a comment-list container first to avoid scanning
      // every <div>/<li>/<section> on the page.
      const commentRoot =
        document.querySelector('[class*="comment"], [class*="Comment"], [id*="comment"]') ||
        document.body;
      const candidates = commentRoot.querySelectorAll(
        '[class*="comment-item"], [class*="CommentItem"], [class*="comment"] > div, [class*="comment"] > li, li, section'
      );
      for (const el of candidates) {
        const text = el.innerText.trim();
        if (text.length < 5 || text.length > 1000) continue;

        // Try to find a user name link or bold text inside
        const userEl = el.querySelector('a[href*="/user/profile/"], strong, b, [class*="name"]');
        if (!userEl) continue;

        const user = userEl.innerText.trim();
        if (!user || user.length > 50) continue;

        // Content: derived from a dedicated body node when available.
        let content = extractBody(el, userEl).slice(0, 500);

        // Avatar
        let avatar = '';
        const img = el.querySelector('img');
        if (img) avatar = img.src || '';

        // User ID
        let user_id = '';
        const userLink = el.querySelector('a[href*="/user/profile/"]');
        if (userLink) {
          const um = userLink.getAttribute('href').match(/\/user\/profile\/([a-zA-Z0-9]+)/);
          if (um) user_id = um[1];
        }

        // Likes / time
        let likes = '';
        let time = '';
        const spans = el.querySelectorAll('span, div');
        for (const s of spans) {
          const t = s.innerText.trim();
          if (/^\d+[\d\.]*[kw]?$/i.test(t) && t.length < 10 && !likes) {
            likes = t;
          }
          if ((t.includes('前') || t.includes('天') || /\d{2}-\d{2}/.test(t)) && !time) {
            time = t;
          }
        }

        // Replies: look for nested comment-like elements
        const replies = [];
        const replyEls = el.querySelectorAll('div, li');
        for (const rel of replyEls) {
          if (rel === el) continue;
          const rText = rel.innerText.trim();
          if (rText.length < 5 || rText.length > 500) continue;
          const rUser = rel.querySelector('a[href*="/user/profile/"], strong, b, [class*="name"]');
          if (!rUser) continue;
          const ru = rUser.innerText.trim();
          if (!ru) continue;
          let rc = extractBody(rel, rUser).slice(0, 300);
          if (!rc) rc = rText.replace(/^[:：\s]+/, '').slice(0, 300);
          replies.push({
            id: '',
            user: ru,
            user_id: '',
            avatar: '',
            content: rc,
            likes: '',
            time: '',
            replies: []
          });
        }

        results.push({
          id: '',
          user,
          user_id,
          avatar,
          content,
          likes,
          time,
          replies: replies.slice(0, 5)
        });
      }

      // Deduplicate by user+content
      const seen = new Set();
      const unique = [];
      for (const r of results) {
        const key = r.user + '|' + r.content.slice(0, 50);
        if (seen.has(key)) continue;
        seen.add(key);
        unique.push(r);
      }
      return unique;
    })()
    "#
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
    fn builds_search_url() {
        assert_eq!(
            search_url("穿搭"),
            "https://www.xiaohongshu.com/search_result?keyword=%E7%A9%BF%E6%90%AD&source=web_explore"
        );
    }

    #[test]
    fn builds_profile_url() {
        assert_eq!(
            profile_url("5f3a9b2c1d4e8f7a6b5c"),
            "https://www.xiaohongshu.com/user/profile/5f3a9b2c1d4e8f7a6b5c"
        );
    }

    #[test]
    fn builds_note_url() {
        assert_eq!(
            note_url("64f8a2b1c3d5e7f9a0b1"),
            "https://www.xiaohongshu.com/explore/64f8a2b1c3d5e7f9a0b1"
        );
    }

    #[tokio::test]
    async fn search_returns_parsed_notes() {
        let bridge = MockBridge::new(vec![
            json!(true), // wait_for_js_truthy
            json!([
                {
                    "id": "n1",
                    "title": "Summer outfits",
                    "desc": "Some description",
                    "author": "Alice",
                    "author_id": "u1",
                    "likes": "1.2k",
                    "cover": "https://example.com/c1.jpg",
                    "url": "https://www.xiaohongshu.com/explore/n1"
                },
                {
                    "id": "n2",
                    "title": "Winter coats",
                    "desc": "Another desc",
                    "author": "Bob",
                    "author_id": "u2",
                    "likes": "500",
                    "cover": "https://example.com/c2.jpg",
                    "url": "https://www.xiaohongshu.com/explore/n2"
                }
            ]),
        ]);
        let browser = Browser::new(bridge);

        let out = search(
            &browser,
            SearchOptions {
                keyword: "穿搭".to_string(),
                limit: 1,
            },
        )
        .await
        .unwrap();

        assert_eq!(out.keyword, "穿搭");
        assert_eq!(out.count, 1);
        assert_eq!(out.notes[0].id, "n1");
        assert_eq!(out.notes[0].title, "Summer outfits");
    }

    #[tokio::test]
    async fn search_rejects_empty_keyword() {
        let bridge = MockBridge::new(vec![]);
        let browser = Browser::new(bridge);

        let err = search(
            &browser,
            SearchOptions {
                keyword: "  ".to_string(),
                limit: 10,
            },
        )
        .await
        .unwrap_err();

        assert_eq!(err.code(), "missing_args");
    }

    #[tokio::test]
    async fn profile_returns_user_and_notes() {
        let bridge = MockBridge::new(vec![
            json!(true), // wait
            json!({
                "user": {
                    "nickname": "Alice",
                    "user_id": "u1",
                    "avatar": "https://example.com/a.jpg",
                    "bio": "Fashion blogger",
                    "followers": "10k",
                    "following": "200",
                    "notes_count": "150"
                },
                "notes": [
                    {
                        "id": "n1",
                        "title": "OOTD",
                        "desc": "",
                        "author": "Alice",
                        "author_id": "u1",
                        "likes": "1k",
                        "cover": "",
                        "url": "https://www.xiaohongshu.com/explore/n1"
                    }
                ]
            }),
        ]);
        let browser = Browser::new(bridge);

        let out = profile(
            &browser,
            ProfileOptions {
                user_id: "u1".to_string(),
                limit: 10,
            },
        )
        .await
        .unwrap();

        assert_eq!(out.user.nickname, "Alice");
        assert_eq!(out.notes.len(), 1);
    }

    #[tokio::test]
    async fn note_returns_detail() {
        let bridge = MockBridge::new(vec![
            json!(true), // wait
            json!({
                "id": "n1",
                "title": "Test Note",
                "content": "This is the body",
                "author": "Alice",
                "author_id": "u1",
                "likes": "100",
                "collects": "50",
                "comments_count": "20",
                "images": ["https://example.com/i1.jpg"],
                "url": "https://www.xiaohongshu.com/explore/n1",
                "publish_time": "2024-01-15"
            }),
        ]);
        let browser = Browser::new(bridge);

        let out = note(
            &browser,
            NoteOptions {
                note_id: "n1".to_string(),
            },
        )
        .await
        .unwrap();

        assert_eq!(out.title, "Test Note");
        assert_eq!(out.likes, "100");
    }

    #[tokio::test]
    async fn comments_returns_list() {
        let bridge = MockBridge::new(vec![
            json!(true), // wait
            json!([
                {
                    "id": "c1",
                    "user": "Bob",
                    "user_id": "u2",
                    "avatar": "",
                    "content": "Great post!",
                    "likes": "10",
                    "time": "2 days ago",
                    "replies": []
                }
            ]),
        ]);
        let browser = Browser::new(bridge);

        let out = comments(
            &browser,
            CommentOptions {
                note_id: "n1".to_string(),
                limit: 10,
            },
        )
        .await
        .unwrap();

        assert_eq!(out.note_id, "n1");
        assert_eq!(out.comments.len(), 1);
        assert_eq!(out.comments[0].content, "Great post!");
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
