use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::value::RawValue;
use xcli_core::{Result, XCliError};

#[derive(Debug, Clone)]
pub struct WebBridgeClient {
    base_url: String,
    session: String,
    http: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStatus {
    pub running: bool,
    pub extension_connected: bool,
    #[serde(default)]
    pub extension_version: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CommandResponse<'a> {
    ok: bool,
    #[serde(borrow)]
    data: Option<&'a RawValue>,
    error: Option<CommandError>,
}

#[derive(Debug, Deserialize)]
struct CommandError {
    code: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct EvaluateWrapper<'a> {
    #[serde(default)]
    r#type: Option<String>,
    #[serde(borrow)]
    value: &'a RawValue,
}

#[async_trait]
pub trait BrowserBridge: Send + Sync {
    async fn status(&self) -> Result<BridgeStatus>;
    async fn navigate(&self, url: &str) -> Result<()>;
    async fn eval<T>(&self, javascript: &str) -> Result<T>
    where
        T: DeserializeOwned + Send;
}

impl WebBridgeClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self::with_session(base_url, "x-cli-rs")
    }

    pub fn with_session(base_url: impl Into<String>, session: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            session: session.into(),
            http: reqwest::Client::new(),
        }
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path.trim_start_matches('/'))
    }

    async fn command(&self, action: &str, args: serde_json::Value) -> Result<Option<String>> {
        let payload = serde_json::json!({
            "action": action,
            "session": self.session,
            "args": args,
        });

        let body = self
            .http
            .post(self.endpoint("command"))
            .json(&payload)
            .send()
            .await
            .map_err(|_| XCliError::DaemonUnreachable(self.base_url.clone()))?
            .error_for_status()
            .map_err(|err| XCliError::BrowserActionFailed(err.to_string()))?
            .text()
            .await
            .map_err(|err| XCliError::BrowserActionFailed(err.to_string()))?;

        parse_command_response(&body)
    }
}

fn parse_command_response(body: &str) -> Result<Option<String>> {
    let response: CommandResponse<'_> = serde_json::from_str(body)
        .map_err(|err| XCliError::BrowserActionFailed(format!("parse command response: {err}")))?;

    if !response.ok {
        if let Some(error) = response.error {
            return Err(XCliError::BrowserActionFailed(format!(
                "{}: {}",
                error.code, error.message
            )));
        }
        return Err(XCliError::BrowserActionFailed(
            "daemon returned ok=false with no error body".to_string(),
        ));
    }

    Ok(response.data.map(|value| value.get().to_string()))
}

fn parse_evaluate_data<T>(data: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let wrapper: EvaluateWrapper<'_> = serde_json::from_str(data).map_err(|err| {
        XCliError::BrowserActionFailed(format!("parse evaluate wrapper: {err} (raw={data})"))
    })?;

    serde_json::from_str(wrapper.value.get()).map_err(|err| {
        let type_hint = wrapper.r#type.as_deref().unwrap_or("unknown");
        XCliError::BrowserActionFailed(format!(
            "parse evaluate value: {err} (type={type_hint}, raw={})",
            wrapper.value.get()
        ))
    })
}

#[async_trait]
impl BrowserBridge for WebBridgeClient {
    async fn status(&self) -> Result<BridgeStatus> {
        self.http
            .get(self.endpoint("status"))
            .send()
            .await
            .map_err(|_| XCliError::DaemonUnreachable(self.base_url.clone()))?
            .error_for_status()
            .map_err(|_| XCliError::DaemonNotRunning)?
            .json::<BridgeStatus>()
            .await
            .map_err(|err| XCliError::BrowserActionFailed(err.to_string()))
    }

    async fn navigate(&self, url: &str) -> Result<()> {
        self.command(
            "navigate",
            serde_json::json!({ "url": url, "newTab": false }),
        )
        .await?;
        Ok(())
    }

    async fn eval<T>(&self, javascript: &str) -> Result<T>
    where
        T: DeserializeOwned + Send,
    {
        let data = self
            .command("evaluate", serde_json::json!({ "code": javascript }))
            .await?
            .ok_or_else(|| {
                XCliError::BrowserActionFailed("evaluate returned no data".to_string())
            })?;

        parse_evaluate_data(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_command_success_with_data() {
        let body = "{\"ok\":true,\"data\":{\"type\":\"string\",\"value\":\"hello\"}}";
        let data = parse_command_response(body).unwrap().unwrap();

        assert_eq!(data, "{\"type\":\"string\",\"value\":\"hello\"}");
    }

    #[test]
    fn parses_command_success_without_data() {
        let body = "{\"ok\":true}";
        let data = parse_command_response(body).unwrap();

        assert!(data.is_none());
    }

    #[test]
    fn parses_command_error_body() {
        let body = "{\"ok\":false,\"error\":{\"code\":\"bad_selector\",\"message\":\"not found\"}}";
        let err = parse_command_response(body).unwrap_err();

        assert_eq!(err.code(), "browser_action_failed");
        assert!(err.to_string().contains("bad_selector: not found"));
    }

    #[test]
    fn parses_evaluate_string_value() {
        let data = "{\"type\":\"string\",\"value\":\"hello\"}";
        let value: String = parse_evaluate_data(data).unwrap();

        assert_eq!(value, "hello");
    }

    #[test]
    fn parses_evaluate_object_value() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Payload {
            src: String,
            alt: Option<String>,
        }

        let data =
            "{\"type\":\"object\",\"value\":{\"src\":\"https://example.com/a.png\",\"alt\":null}}";
        let value: Payload = parse_evaluate_data(data).unwrap();

        assert_eq!(
            value,
            Payload {
                src: "https://example.com/a.png".to_string(),
                alt: None,
            }
        );
    }

    #[test]
    fn reports_evaluate_value_parse_error_with_type_hint() {
        let data = "{\"type\":\"number\",\"value\":123}";
        let err = parse_evaluate_data::<String>(data).unwrap_err();

        assert_eq!(err.code(), "browser_action_failed");
        assert!(err.to_string().contains("type=number"));
    }
}
