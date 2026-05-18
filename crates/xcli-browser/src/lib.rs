use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;
use xcli_core::{Result, XCliError};
use xcli_webbridge::BrowserBridge;

pub struct Browser<B> {
    bridge: B,
}

impl<B> Browser<B>
where
    B: BrowserBridge,
{
    pub fn new(bridge: B) -> Self {
        Self { bridge }
    }

    pub async fn ensure_ready(&self) -> Result<()> {
        let status = self.bridge.status().await?;
        if !status.running {
            return Err(XCliError::DaemonNotRunning);
        }
        if !status.extension_connected {
            return Err(XCliError::ExtensionNotConnected);
        }
        Ok(())
    }

    pub async fn goto(&self, url: &str) -> Result<()> {
        self.bridge.navigate(url).await
    }

    pub async fn eval<T>(&self, javascript: &str) -> Result<T>
    where
        T: DeserializeOwned + Send,
    {
        self.bridge.eval(javascript).await
    }

    pub async fn click(&self, selector: &str) -> Result<()> {
        let js = format!(
            r#"
            (() => {{
              const el = document.querySelector({selector:?});
              if (!el) throw new Error('selector not found: {selector}');
              el.click();
              return true;
            }})()
            "#,
        );
        self.eval::<bool>(&js).await.map(|_| ())
    }

    pub async fn insert_text(&self, selector: &str, text: &str) -> Result<()> {
        let js = contenteditable_insert_script(selector, text);
        self.eval::<bool>(&js).await.map(|_| ())
    }

    pub async fn wait_for_js_truthy(&self, javascript: &str, timeout: Duration) -> Result<()> {
        let start = Instant::now();
        loop {
            if self.eval::<bool>(javascript).await? {
                return Ok(());
            }
            if start.elapsed() >= timeout {
                return Err(XCliError::BrowserActionFailed("wait timeout".to_string()));
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
}

fn contenteditable_insert_script(selector: &str, text: &str) -> String {
    format!(
        r#"
        (() => {{
          const selector = {selector:?};
          const text = {text:?};
          const el = document.querySelector(selector);
          if (!el) throw new Error(`selector not found: ${{selector}}`);

          el.focus();

          const beforeInput = new InputEvent('beforeinput', {{
            bubbles: true,
            cancelable: true,
            inputType: 'insertText',
            data: text,
          }});
          el.dispatchEvent(beforeInput);

          if (el.isContentEditable || el.getAttribute('contenteditable') === 'true') {{
            const selection = window.getSelection();
            const range = document.createRange();
            range.selectNodeContents(el);
            range.deleteContents();

            const textNode = document.createTextNode(text);
            range.insertNode(textNode);
            range.setStartAfter(textNode);
            range.setEndAfter(textNode);

            selection.removeAllRanges();
            selection.addRange(range);
          }} else if ('value' in el) {{
            el.value = text;
            const valueSetter = Object.getOwnPropertyDescriptor(el.__proto__, 'value')?.set;
            if (valueSetter) valueSetter.call(el, text);
          }} else {{
            el.textContent = text;
          }}

          el.dispatchEvent(new InputEvent('input', {{
            bubbles: true,
            composed: true,
            inputType: 'insertText',
            data: text,
          }}));
          el.dispatchEvent(new Event('change', {{ bubbles: true }}));

          return true;
        }})()
        "#,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contenteditable_insert_script_escapes_prompt_text() {
        let script = contenteditable_insert_script("#prompt-textarea", "hello `world` ${x}");

        assert!(script.contains("#prompt-textarea"));
        assert!(script.contains("hello `world` ${x}"));
        assert!(script.contains("beforeinput"));
        assert!(script.contains("window.getSelection"));
        assert!(script.contains("InputEvent('input'"));
    }

    #[test]
    fn contenteditable_insert_script_supports_plain_inputs() {
        let script = contenteditable_insert_script("textarea[name=q]", "rust");

        assert!(script.contains("'value' in el"));
        assert!(script.contains("valueSetter.call"));
        assert!(script.contains("Event('change'"));
    }
}
