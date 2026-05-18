#[derive(Debug, thiserror::Error)]
pub enum XCliError {
    #[error("missing args: {0}")]
    MissingArgs(String),

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

    #[error("search failed: {0}")]
    SearchFailed(String),

    #[error("consent required: {0}")]
    ConsentRequired(String),

    #[error("no results: {0}")]
    NoResults(String),

    #[error("browser action failed: {0}")]
    BrowserActionFailed(String),
}

impl XCliError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingArgs(_) => "missing_args",
            Self::InvalidArgs(_) => "invalid_args",
            Self::DaemonUnreachable(_) => "daemon_unreachable",
            Self::DaemonNotRunning => "daemon_not_running",
            Self::ExtensionNotConnected => "extension_not_connected",
            Self::GenerateFailed(_) => "generate_failed",
            Self::SearchFailed(_) => "search_failed",
            Self::ConsentRequired(_) => "consent_required",
            Self::NoResults(_) => "no_results",
            Self::BrowserActionFailed(_) => "browser_action_failed",
        }
    }
}

pub type Result<T> = std::result::Result<T, XCliError>;
