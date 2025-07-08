/// Configuration management for the AfricasTalking SDK
use crate::error::{AfricasTalkingError, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Environment configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    /// Sandbox environment for testing
    Sandbox,
    /// Production environment
    Production,
}

impl Environment {
    /// Get the base URL for the environment
    pub fn base_url(&self) -> &'static str {
        match self {
            Environment::Sandbox => "https://api.sandbox.africastalking.com",
            Environment::Production => "https://api.africastalking.com",
        }
    }
}

/// Configuration for the AfricasTalking client
#[derive(Debug, Clone)]
pub struct Config {
    /// API key for authentication
    pub api_key: String,
    /// Username for the application
    pub username: String,
    /// Environment (sandbox or production)
    pub environment: Environment,
    /// Request timeout duration
    pub timeout: Duration,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Custom user agent string
    pub user_agent: Option<String>,
}

impl Config {
    /// Create a new configuration
    pub fn new<S: Into<String>>(api_key: S, username: S) -> Self {
        Self {
            api_key: api_key.into(),
            username: username.into(),
            environment: Environment::Sandbox,
            timeout: Duration::from_secs(30),
            max_retries: 3,
            user_agent: None,
        }
    }

    /// Set the environment
    pub fn environment(mut self, env: Environment) -> Self {
        self.environment = env;
        self
    }

    /// Set the timeout duration
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set maximum retry attempts
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set custom user agent
    pub fn user_agent<S: Into<String>>(mut self, user_agent: S) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.api_key.is_empty() {
            return Err(AfricasTalkingError::config("API key cannot be empty"));
        }

        if self.username.is_empty() {
            return Err(AfricasTalkingError::config("Username cannot be empty"));
        }

        if self.timeout.as_secs() == 0 {
            return Err(AfricasTalkingError::config(
                "Timeout must be greater than 0",
            ));
        }

        Ok(())
    }
}
