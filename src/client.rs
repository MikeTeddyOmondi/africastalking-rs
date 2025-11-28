//! Main client implementation for the AfricasTalking SDK

use crate::{
    config::Config,
    error::{AfricasTalkingError, ApiErrorResponse, Result},
    modules::*,
};
use reqwest::{Client as HttpClient, Method, Response, header::HeaderMap};
use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;
use tokio::time::sleep;

/// Main client for interacting with the AfricasTalking API
#[derive(Debug, Clone)]
pub struct AfricasTalkingClient {
    pub http_client: HttpClient,
    pub config: Config,
}

impl AfricasTalkingClient {
    /// Create a new client with the given configuration
    pub fn new(config: Config) -> Result<Self> {
        config.validate()?;

        let http_client = HttpClient::builder()
            .timeout(config.timeout)
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

    /// Get the Data module
    pub fn data(&self) -> DataModule {
        DataModule::new(self.clone())
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
    pub(crate) async fn post<T, R>(
        &self,
        endpoint: &str,
        payload: &T,
        headers: Option<HeaderMap>,
    ) -> Result<R>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        self.request(Method::POST, endpoint, Some(payload), headers)
            .await
    }

    /// Make a GET request to the API
    pub(crate) async fn get<R>(&self, endpoint: &str, headers: Option<HeaderMap>) -> Result<R>
    where
        R: DeserializeOwned,
    {
        self.request::<(), R>(Method::GET, endpoint, None, headers)
            .await
    }

    /// Make a request with retry logic
    async fn request<T, R>(
        &self,
        method: Method,
        endpoint: &str,
        payload: Option<&T>,
        headers: Option<HeaderMap>,
    ) -> Result<R>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let mut attempts = 0;
        let max_attempts = self.config.max_retries + 1;

        loop {
            attempts += 1;

            match self
                .make_request(&method, endpoint, payload, headers.clone())
                .await
            {
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
        headers: Option<HeaderMap>,
    ) -> Result<Response>
    where
        T: Serialize,
    {
        let url = self.get_url(endpoint);
        let mut request = self.http_client.request(method.clone(), &url);

        //check if headers have been submitted.
        if let Some(headers) = headers {
            //check if payload has been submitted.
            if let Some(payload) = payload {
                // Check if the special header exists
                if let Some(content_type) = headers.get("Content-Type") {
                    if content_type.to_str().unwrap_or("") == "application/x-www-form-urlencoded" {
                        
                        // Use form data
                        let form_data = self.construct_form_data(Some(payload));
                        request = request.form(&form_data);
                    } else {
                        // Use JSON body
                        request = request.json(payload);
                    }
                } else {
                    // set json body to the request
                    request = request.json(payload);
                }
            }

            // "application/x-www-form-urlencoded"

            // finally add all headers to the request
            request = request.headers(headers);
        }

        let response = request.send().await?;
        Ok(response)
    }

    /**
     * Compute the url for the request.
     * @param endpoint The API endpoint.
     * @return String The full URL for the request.
     */
    fn get_url(&self, endpoint: &str) -> String {
        if endpoint.contains("mobile/data/request") {
            let base = self.config.environment.base_url().replace("api", "bundles");
            return format!("{}{}", base, endpoint);
        }
        format!("{}{}", self.config.environment.base_url(), endpoint)
    }

    /**
     * Construct form data for the request/payload.
     * @param payload The payload to include in the form data.
     * @return Vec<(String, String)> The constructed form data.
     */
    fn construct_form_data<T>(&self, payload: Option<&T>) -> Vec<(String, String)>
    where
        T: Serialize,
    {
        // Add username to all requests
        let mut form_data: Vec<(String, String)> =
            vec![("username".to_string(), self.config.username.clone())];

        if let Some(payload) = payload {
            // Convert payload to form data
            let payload_str = serde_json::to_string(payload).unwrap();
            let payload_map: std::collections::HashMap<String, serde_json::Value> =
                serde_json::from_str(&payload_str).unwrap();

            for (key, value) in payload_map {
                let value_str = match value {
                    serde_json::Value::String(s) => s,
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => serde_json::to_string(&value).unwrap(),
                };
                form_data.push((key, value_str));
            }
        }

        form_data
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
                    error_response
                        .error_code
                        .unwrap_or_else(|| status.to_string()),
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

    pub fn get_sms_apis_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("Accept", "application/json".parse().unwrap());
        headers.insert(
            "Content-Type",
            "application/x-www-form-urlencoded".parse().unwrap(),
        );
        headers.insert("ApiKey", self.config.api_key.parse().unwrap());

        if let Some(user_agent) = self.config.user_agent.clone() {
            headers.insert("User-Agent", user_agent.parse().unwrap());
        }
        headers
    }
}
