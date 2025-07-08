//! Main client implementation for the AfricasTalking SDK

use crate::{
    config::Config,
    error::{AfricasTalkingError, ApiErrorResponse, Result},
    modules::*,
};
use reqwest::{header::HeaderMap, Client as HttpClient, Method, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tokio::time::sleep;

/// Main client for interacting with the AfricasTalking API
#[derive(Debug, Clone)]
pub struct AfricasTalkingClient {
    http_client: HttpClient,
    config: Config,
}

impl AfricasTalkingClient {
    /// Create a new client with the given configuration
    pub fn new(config: Config) -> Result<Self> {
        config.validate()?;
        
        let mut headers = HeaderMap::new();
        headers.insert("Accept", "application/json".parse().unwrap());
        headers.insert("Content-Type", "application/x-www-form-urlencoded".parse().unwrap());
        headers.insert("ApiKey", config.api_key.parse().unwrap());
        
        if let Some(user_agent) = &config.user_agent {
            headers.insert("User-Agent", user_agent.parse().unwrap());
        }
        
        let http_client = HttpClient::builder()
            .timeout(config.timeout)
            .default_headers(headers)
            .build()
            .map_err(AfricasTalkingError::Http)?;
        
        Ok(Self {
            http_client,
            config,
        })
    }
    
    /// Get the SMS module
    pub fn sms(&self) -> SmsModule {
        SmsModule::new(self.clone())
    }
    
    /// Get the Airtime module
    pub fn airtime(&self) -> AirtimeModule {
        AirtimeModule::new(self.clone())
    }
    
    /// Get the Application module
    pub fn application(&self) -> ApplicationModule {
        ApplicationModule::new(self.clone())
    }
    
    // Add more modules as they're implemented
    // pub fn voice(&self) -> VoiceModule { ... }
    // pub fn payments(&self) -> PaymentsModule { ... }
    // pub fn data(&self) -> DataModule { ... }
    
    /// Make a POST request to the API
    pub(crate) async fn post<T, R>(&self, endpoint: &str, payload: &T) -> Result<R>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        self.request(Method::POST, endpoint, Some(payload)).await
    }
    
    /// Make a GET request to the API
    pub(crate) async fn get<R>(&self, endpoint: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        self.request::<(), R>(Method::GET, endpoint, None).await
    }
    
    /// Make a request with retry logic
    async fn request<T, R>(&self, method: Method, endpoint: &str, payload: Option<&T>) -> Result<R>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let mut attempts = 0;
        let max_attempts = self.config.max_retries + 1;
        
        loop {
            attempts += 1;
            
            match self.make_request(&method, endpoint, payload).await {
                Ok(response) => return self.handle_response(response).await,
                Err(e) if attempts < max_attempts && e.is_retryable() => {
                    let delay = Duration::from_millis(1000 * attempts as u64);
                    sleep(delay).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }
    
    /// Make a single HTTP request
    async fn make_request<T>(
        &self,
        method: &Method,
        endpoint: &str,
        payload: Option<&T>,
    ) -> Result<Response>
    where
        T: Serialize,
    {
        let url = format!("{}{}?senderId={}", self.config.environment.base_url(), endpoint, self.config.username);
        let mut request = self.http_client.request(method.clone(), &url);
        
        // Add username to all requests
        let mut form_data = vec![("username".to_string(), self.config.username.clone())];
        
        if let Some(payload) = payload {
            // Convert payload to form data
            let payload_str = serde_json::to_string(payload)?;
            let payload_map: std::collections::HashMap<String, serde_json::Value> = 
                serde_json::from_str(&payload_str)?;
            
            for (key, value) in payload_map {
                let value_str = match value {
                    serde_json::Value::String(s) => s,
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => serde_json::to_string(&value)?,
                };
                form_data.push((key, value_str));
            }
        }
        
        request = request.form(&form_data);
        
        let response = request.send().await?;
        Ok(response)
    }
    
    /// Handle the HTTP response
    async fn handle_response<R>(&self, response: Response) -> Result<R>
    where
        R: DeserializeOwned,
    {
        let status = response.status();
        let response_text = response.text().await?;
        
        // Handle rate limiting
        if status == 429 {
            return Err(AfricasTalkingError::RateLimit { retry_after: 60 });
        }
        
        // Try to parse as error response first
        if !status.is_success() {
            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&response_text) {
                return Err(AfricasTalkingError::api_error(
                    error_response.error_message,
                    error_response.error_code.unwrap_or_else(|| status.to_string()),
                    error_response.more_info,
                ));
            }
            
            return Err(AfricasTalkingError::api_error(
                format!("HTTP {status}: {response_text}"),
                status.to_string(),
                None,
            ));
        }
        
        // Parse successful response
        serde_json::from_str::<R>(&response_text).map_err(|e| {
            eprintln!("Failed to parse response: {response_text}");
            AfricasTalkingError::Serialization(e)
        })
    }
}
