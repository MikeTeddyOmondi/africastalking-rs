//! Common types used across the SDK

use serde::{Deserialize, Serialize};

/// Standard response wrapper for most API calls
#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    #[serde(flatten)]
    pub data: T,
}

/// Standard error response from the API
#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    #[serde(rename = "ErrorMessage")]
    pub error_message: String,
    #[serde(rename = "ErrorCode")]
    pub error_code: Option<String>,
}

/// Pagination information for list responses
#[derive(Debug, Deserialize, Serialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub total: u32,
    pub total_pages: u32,
}

/// Currency types supported by AfricasTalking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Currency {
    #[serde(rename = "KES")]
    Kes,
    #[serde(rename = "USD")]
    Usd,
    #[serde(rename = "UGX")]
    Ugx,
    #[serde(rename = "TZS")]
    Tzs,
    #[serde(rename = "RWF")]
    Rwf,
    #[serde(rename = "ZMW")]
    Zmw,
    #[serde(rename = "NGN")]
    Ngn,
    #[serde(rename = "GHS")]
    Ghs,
}

impl Currency {
    pub fn as_str(&self) -> &'static str {
        match self {
            Currency::Kes => "KES",
            Currency::Usd => "USD",
            Currency::Ugx => "UGX",
            Currency::Tzs => "TZS",
            Currency::Rwf => "RWF",
            Currency::Zmw => "ZMW",
            Currency::Ngn => "NGN",
            Currency::Ghs => "GHS",
        }
    }
}

/// Phone number with country code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneNumber {
    pub number: String,
    pub country_code: Option<String>,
}

impl PhoneNumber {
    pub fn new<S: Into<String>>(number: S) -> Self {
        Self {
            number: number.into(),
            country_code: None,
        }
    }
    
    pub fn with_country_code<S: Into<String>>(number: S, country_code: S) -> Self {
        Self {
            number: number.into(),
            country_code: Some(country_code.into()),
        }
    }
}
