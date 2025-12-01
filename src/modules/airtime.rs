// src/modules/airtime.rs
//! Airtime module implementation

use crate::{client::AfricasTalkingClient, error::Result, Currency};
use serde::{Deserialize, Serialize};

/// Airtime module for sending airtime
#[derive(Debug, Clone)]
pub struct AirtimeModule {
    client: AfricasTalkingClient,
}

impl AirtimeModule {
    pub(crate) fn new(client: AfricasTalkingClient) -> Self {
        Self { client }
    }
    
    /// Send airtime to recipients
    pub async fn send(&self, request: SendAirtimeRequest) -> Result<SendAirtimeResponse> {
        self.client.post("/version1/airtime/send", &request).await
    }
}

#[derive(Debug, Serialize)]
pub struct SendAirtimeRequest {
    pub recipients: Vec<AirtimeRecipient>,
}

#[derive(Debug, Serialize)]
pub struct AirtimeRecipient {
    #[serde(rename = "phoneNumber")]
    pub phone_number: String,
    #[serde(rename = "currencyCode")]
    pub currency_code: String,
    pub amount: String,
}

impl AirtimeRecipient {
    pub fn new<S: Into<String>>(phone_number: S, amount: S, currency: Currency) -> Self {
        Self {
            phone_number: phone_number.into(),
            currency_code: currency.as_str().to_string(),
            amount: amount.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SendAirtimeResponse {
    #[serde(rename = "errorMessage")]
    pub error_message: String,
    #[serde(rename = "numSent")]
    pub num_sent: u32,
    #[serde(rename = "totalAmount")]
    pub total_amount: String,
    #[serde(rename = "totalDiscount")]
    pub total_discount: String,
    #[serde(rename = "responses")]
    pub responses: Vec<AirtimeResponse>,
}

#[derive(Debug, Deserialize)]
pub struct AirtimeResponse {
    #[serde(rename = "phoneNumber")]
    pub phone_number: String,
    #[serde(rename = "amount")]
    pub amount: String,
    #[serde(rename = "status")]
    pub status: String,
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(rename = "discount")]
    pub discount: String,
    #[serde(rename = "errorMessage")]
    pub error_message: String,
}
