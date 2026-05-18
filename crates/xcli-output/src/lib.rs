use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct JsonResponse<T>
where
    T: Serialize,
{
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonError>,
}

#[derive(Debug, Serialize)]
pub struct JsonError {
    pub code: String,
    pub message: String,
}

impl<T> JsonResponse<T>
where
    T: Serialize,
{
    pub fn ok(data: T) -> Self {
        Self {
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            ok: false,
            data: None,
            error: Some(JsonError {
                code: code.into(),
                message: message.into(),
            }),
        }
    }
}

pub fn print_json<T>(value: &T) -> Result<(), serde_json::Error>
where
    T: Serialize,
{
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_success_without_error_field() {
        let response = JsonResponse::ok(json!({ "path": "/tmp/image.png" }));
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["ok"], true);
        assert_eq!(value["data"]["path"], "/tmp/image.png");
        assert!(value.get("error").is_none());
    }

    #[test]
    fn serializes_error_without_data_field() {
        let response = JsonResponse::<()>::error("invalid_args", "prompt must not be empty");
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["code"], "invalid_args");
        assert_eq!(value["error"]["message"], "prompt must not be empty");
        assert!(value.get("data").is_none());
    }
}
