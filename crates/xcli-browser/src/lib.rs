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
        let js = format!(
            r#"
            (() => {{
              const el = document.querySelector({selector:?});
              if (!el) throw new Error('selector not found: {selector}');
              el.focus();
              el.textContent = {text:?};
              el.dispatchEvent(new InputEvent('input', {{ bubbles: true, inputType: 'insertText', data: {text:?} }}));
              return true;
            }})()
            "#,
        );
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
