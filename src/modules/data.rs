use crate::{client::AfricasTalkingClient, error::Result};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

/// SMS module for sending and managing SMS messages
#[derive(Debug, Clone)]
pub struct DataModule {
    client: AfricasTalkingClient,
}

impl DataModule {
    pub(crate) fn new(client: AfricasTalkingClient) -> Self {
        Self { client }
    }

    /// Send SMS to one or more recipients
    pub async fn send(&self, request: MobileDataRequest) -> Result<MobileDataResponseList> {
        // let headers = self.get_data_request_headers();
        self.client.post("/mobile/data/request", &request).await
    }

    /**
     * Get headers for data request API.
     * @return HeaderMap the headers for the request.
     */
    fn get_data_request_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("Accept", HeaderValue::from_static("application/json"));
        headers.insert(
            "apiKey",
            HeaderValue::from_str(&self.client.config.api_key).unwrap(),
        );
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers
    }
}

#[derive(Debug, Serialize)]
pub struct MobileDataRequest {
    #[serde(rename = "username")]
    pub user_name: String,
    #[serde(rename = "productName")]
    pub product_name: String,
    pub recipients: Vec<Recipient>,
}

#[derive(Debug, Serialize)]
pub struct RecipientMetadata {
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
}

#[derive(Debug, Serialize)]
pub struct Recipient {
    #[serde(rename = "phoneNumber")]
    pub phone_number: String,
    pub quantity: u32,
    pub unit: String,
    pub validity: String,
    #[serde(rename = "isPromoBundle")]
    pub is_promo_bundle: bool,
    pub metadata: RecipientMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MobileDataResponse {
    #[serde(rename = "errorMessage", skip_serializing_if = "Option::is_none")]
    error_message: Option<String>,

    #[serde(rename = "phoneNumber", skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    #[serde(rename = "transactionId", skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MobileDataResponseList {
    #[serde(default)]
    pub entries: Vec<MobileDataResponse>,
    #[serde(rename = "errorMessage", skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}
