use std::fmt;

use crate::{client::AfricasTalkingClient, error::Result};
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
        self.client.post_json("/mobile/data/request", &request).await
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

// The available data validity classes.
#[derive(Debug, Serialize, Deserialize)]
pub enum DataValidity {
    Day,
    Week,
    Month,
}

impl fmt::Display for DataValidity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let validity_str = match self {
            DataValidity::Day => "Day",
            DataValidity::Week => "Week",
            DataValidity::Month => "Month",
        };
        write!(f, "{}", validity_str)
    }
}

// The avaibale data packages/units.
#[derive(Debug, Serialize, Deserialize)]
pub enum DataUnits {
    MB,
    GB,
}

impl fmt::Display for DataUnits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let unit_str = match self {
            DataUnits::MB => "MB",
            DataUnits::GB => "GB",
        };
        write!(f, "{}", unit_str)
    }
}

#[derive(Debug, Serialize)]
pub struct Recipient {
    #[serde(rename = "phoneNumber")]
    pub phone_number: String,
    pub quantity: u32,
    pub unit: DataUnits,
    pub validity: DataValidity,
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
