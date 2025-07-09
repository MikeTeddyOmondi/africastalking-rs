//! SMS module implementation

use crate::{client::AfricasTalkingClient, error::Result};
use serde::{Deserialize, Serialize};

/// SMS module for sending and managing SMS messages
#[derive(Debug, Clone)]
pub struct SmsModule {
    client: AfricasTalkingClient,
}

impl SmsModule {
    pub(crate) fn new(client: AfricasTalkingClient) -> Self {
        Self { client }
    }

    /// Send SMS to one or more recipients
    pub async fn send(&self, request: SendSmsRequest) -> Result<SendSmsResponse> {
        self.client.post("/version1/messaging", &request).await
    }

    /// Fetch SMS messages
    pub async fn fetch_messages(
        &self,
        last_received_id: Option<u32>,
    ) -> Result<FetchMessagesResponse> {
        let endpoint = if let Some(id) = last_received_id {
            format!("/version1/messaging?lastReceivedId={id}")
        } else {
            "/version1/messaging".to_string()
        };

        self.client.get(&endpoint).await
    }
}

#[derive(Debug, Serialize)]
pub struct SendSmsRequest {
    pub to: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bulkSMSMode")]
    pub bulk_sms_mode: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enqueue: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "linkId")]
    pub link_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "retryDurationInHours")]
    pub retry_duration_in_hours: Option<u32>,
}

impl SendSmsRequest {
    pub fn new<S: Into<String>>(to: Vec<S>, message: S) -> Self {
        Self {
            to: to.into_iter().map(|s| s.into()).collect(),
            message: message.into(),
            from: None,
            bulk_sms_mode: None,
            enqueue: None,
            keyword: None,
            link_id: None,
            retry_duration_in_hours: None,
        }
    }

    pub fn from<S: Into<String>>(mut self, from: S) -> Self {
        self.from = Some(from.into());
        self
    }

    pub fn bulk_mode(mut self, enabled: bool) -> Self {
        self.bulk_sms_mode = Some(if enabled { 1 } else { 0 });
        self
    }
}

#[derive(Debug, Deserialize)]
pub struct SendSmsResponse {
    #[serde(rename = "SMSMessageData")]
    pub sms_message_data: SmsMessageData,
}

#[derive(Debug, Deserialize)]
pub struct SmsMessageData {
    #[serde(rename = "Message")]
    pub message: String,
    #[serde(rename = "Recipients")]
    pub recipients: Vec<SmsRecipient>,
}

#[derive(Debug, Deserialize)]
pub struct SmsRecipient {
    #[serde(rename = "statusCode")]
    pub status_code: u32,
    #[serde(rename = "number")]
    pub number: String,
    #[serde(rename = "status")]
    pub status: String,
    #[serde(rename = "cost")]
    pub cost: String,
    #[serde(rename = "messageId")]
    pub message_id: String,
}

#[derive(Debug, Deserialize)]
pub struct FetchMessagesResponse {
    #[serde(rename = "SMSMessageData")]
    pub sms_message_data: FetchSmsMessageData,
}

#[derive(Debug, Deserialize)]
pub struct FetchSmsMessageData {
    #[serde(rename = "Messages")]
    pub messages: Vec<SmsMessage>,
}

#[derive(Debug, Deserialize)]
pub struct SmsMessage {
    #[serde(rename = "id")]
    pub id: u32,
    #[serde(rename = "text")]
    pub text: String,
    #[serde(rename = "from")]
    pub from: String,
    #[serde(rename = "to")]
    pub to: String,
    #[serde(rename = "date")]
    pub date: String,
    #[serde(rename = "linkId")]
    pub link_id: Option<String>,
}
