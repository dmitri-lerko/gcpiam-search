use thiserror::Error;

/// Errors that can occur during scraping
#[derive(Error, Debug)]
pub enum ScraperError {
    #[error("GCP authentication failed: {0}")]
    GcpAuthError(String),

    #[error("GCP rate limit exceeded: {0}")]
    GcpRateLimitError(String),

    #[error("GCP API error: {0}")]
    GcpApiError(String),

    #[error("Data validation error: {0}")]
    DataValidationError(String),

    #[error("File I/O error: {0}")]
    FileIoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Environment error: {0}")]
    EnvError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl ScraperError {
    /// Check if this is an authentication error
    pub fn is_auth_error(&self) -> bool {
        matches!(self, ScraperError::GcpAuthError(_))
    }

    /// Check if this is a rate limit error
    pub fn is_rate_limit_error(&self) -> bool {
        matches!(self, ScraperError::GcpRateLimitError(_))
    }

    /// Get remediation advice for this error
    pub fn remediation_advice(&self) -> Option<&'static str> {
        match self {
            ScraperError::GcpAuthError(_) => Some(
                "Ensure GOOGLE_APPLICATION_CREDENTIALS is set correctly.\n\
                 Service account needs: roles/iam.roleViewer, roles/iam.securityReviewer",
            ),
            ScraperError::GcpRateLimitError(_) => Some(
                "Rate limit exceeded. The scraper will retry automatically.\n\
                 Consider reducing concurrent requests or waiting before retrying.",
            ),
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, ScraperError>;
