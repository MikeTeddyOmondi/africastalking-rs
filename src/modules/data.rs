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
    pub async fn request<M>(&self, request: MobileDataRequest<M>) -> Result<MobileDataResponseList>
    where
        M: Serialize,
    {
        let headers = self.get_data_request_headers();
        self.client
            .post("/mobile/data/request", &request, Some(headers))
            .await
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
pub struct MobileDataRequest<M>
where
    M: Serialize,
{
    pub user_name: String,
    pub product_name: String,
    pub recipients: Vec<Recipient<M>>,
}

#[derive(Debug, Serialize)]
pub struct Recipient<M>
where
    M: Serialize,
{
    pub phone_number: String,
    pub quantity: u32,
    pub unit: String,
    pub valididty: String,
    medata: M,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MobileDataResponse {
    #[serde(rename = "phoneNumber")]
    pub phone_number: String,
    pub provider: String,
    pub status: String,
    #[serde(rename = "ReciptransactionId")]
    pub transaction_id: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MobileDataResponseList {
    #[serde(default)]
    pub responses: Vec<MobileDataResponse>,
}
