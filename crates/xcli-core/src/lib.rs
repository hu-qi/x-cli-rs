#[derive(Debug, thiserror::Error)]
pub enum XCliError {
    #[error("invalid args: {0}")]
    InvalidArgs(String),

    #[error("kimi-webbridge daemon unreachable at {0}")]
    DaemonUnreachable(String),

    #[error("kimi-webbridge daemon is not running")]
    DaemonNotRunning,

    #[error("Chrome WebBridge extension is not connected")]
    ExtensionNotConnected,

    #[error("generation failed: {0}")]
    GenerateFailed(String),

    #[error("browser action failed: {0}")]
    BrowserActionFailed(String),
}

impl XCliError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidArgs(_) => "invalid_args",
            Self::DaemonUnreachable(_) => "daemon_unreachable",
            Self::DaemonNotRunning => "daemon_not_running",
            Self::ExtensionNotConnected => "extension_not_connected",
            Self::GenerateFailed(_) => "generate_failed",
            Self::BrowserActionFailed(_) => "browser_action_failed",
        }
    }
}

pub type Result<T> = std::result::Result<T, XCliError>;
