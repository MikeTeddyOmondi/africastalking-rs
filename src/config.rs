//! Configuration management for the AfricasTalking SDK

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

    /// Get the base domain
    fn base_domain(&self) -> &'static str {
        match self {
            Environment::Sandbox => "sandbox.africastalking.com",
            Environment::Production => "africastalking.com",
        }
    }
}

/// API endpoints that may use different domains
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endpoint {
    /// Standard API endpoints (api domain)
    Standard,
    /// Mobile data endpoints (bundles domain)
    MobileData,
    /// Voice endpoints (voice domain)
    Voice,
    /// Insights endpoints (insights domain)
    Insights,
    /// Content endpoints (content domain)
    Content,
}

impl Endpoint {
    /// Get the full URL for this endpoint
    pub fn build_url(&self, environment: &Environment, path: &str) -> String {
        let domain = environment.base_domain();
        match self {
            Endpoint::Standard => {
                format!("https://api.{}{}", domain, path)
            }
            Endpoint::MobileData => {
                format!("https://bundles.{}{}", domain, path)
            }
            Endpoint::Voice => {
                format!("https://voice.{}{}", domain, path)
            }
            Endpoint::Insights => {
                format!("https://insights.{}{}", domain, path)
            }
            Endpoint::Content => {
                // Content uses version1 path in sandbox, but is a separate domain in production
                match environment {
                    Environment::Sandbox => format!("https://api.{}/version1{}", domain, path),
                    Environment::Production => format!("https://content.{}/version1{}", domain, path),
                }
            }
        }
    }
}

/// Internal mapping of paths to endpoint types
#[derive(Debug, Clone)]
struct EndpointMap;

impl EndpointMap {
    /// Get the endpoint type for a given path
    fn get(&self, path: &str) -> Endpoint {
        if path.contains("mobile/data") {
            Endpoint::MobileData
        } else if path.contains("voice") {
            Endpoint::Voice
        } else if path.contains("insights") {
            Endpoint::Insights
        } else if path.contains("content") {
            Endpoint::Content
        } else {
            Endpoint::Standard
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
    /// Map of endpoint paths to their endpoint types
    endpoint_map: EndpointMap,
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
            endpoint_map: EndpointMap,
        }
    }

    /// Build a full URL for a given endpoint path
    pub fn build_url(&self, path: &str) -> String {
        let endpoint = self.endpoint_map.get(path);
        endpoint.build_url(&self.environment, path)
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
