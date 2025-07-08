//! Error handling for the AfricasTalking SDK

use serde::{Deserialize, Serialize};

/// Main error type for the AfricasTalking SDK
#[derive(Debug, thiserror::Error)]
pub enum AfricasTalkingError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// API returned an error response
    #[error("API error: {message} (code: {code})")]
    Api {
        message: String,
        code: String,
        more_info: Option<String>,
    },

    /// JSON serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Validation error for request parameters
    #[error("Validation error: {0}")]
    Validation(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded. Try again after {retry_after} seconds")]
    RateLimit { retry_after: u64 },

    /// Network timeout
    #[error("Request timeout")]
    Timeout,

    /// Generic internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, AfricasTalkingError>;

/// Standard API error response structure
#[derive(Debug, Deserialize, Serialize)]
pub struct ApiErrorResponse {
    #[serde(rename = "ErrorMessage")]
    pub error_message: String,
    #[serde(rename = "ErrorCode")]
    pub error_code: Option<String>,
    #[serde(rename = "MoreInfo")]
    pub more_info: Option<String>,
}

impl AfricasTalkingError {
    /// Create an API error from response
    pub fn api_error(message: String, code: String, more_info: Option<String>) -> Self {
        Self::Api {
            message,
            code,
            more_info,
        }
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation(message.into())
    }

    /// Create a configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config(message.into())
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AfricasTalkingError::Http(_)
                | AfricasTalkingError::Timeout
                | AfricasTalkingError::RateLimit { .. }
        )
    }
}
