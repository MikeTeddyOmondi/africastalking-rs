//! Voice module implementation for AfricasTalking
//!
//! Build dynamic voice applications for call centers, authentication, surveys, and more.
//!
//! # Features
//!
//! - Make outbound calls
//! - Build XML responses with ActionBuilder
//! - Query call queue status
//! - Upload media files
//!
//! # Examples
//!
//! ```no_run
//! use africastalking::{AfricasTalkingClient, Config, Result};
//! use africastalking::voice::{MakeCallRequest, ActionBuilder, GetDigitsAction};
//!
//! # async fn make_outbound_call() -> Result<()> {
//! let config = Config::new("api_key", "username");
//! let client = AfricasTalkingClient::new(config)?;
//! let voice = client.voice();
//!
//! // Make a call
//! let call = MakeCallRequest::new("+254711XXXYYY", vec!["+254722XXXYYY"])
//!     .with_client_request_id("request-123");
//!
//! let response = voice.make_call(call).await?;
//!
//! // Build XML response
//! let xml = ActionBuilder::new()
//!     .say("Hello, welcome to our service", None)
//!     .get_digits(
//!         GetDigitsAction::new()
//!             .say("Press 1 for support", None)
//!             .finish_on_key('#')
//!             .num_digits(1),
//!     )
//!     .build();
//! # Ok(())
//! # }
//! ```

use crate::{Result, client::AfricasTalkingClient};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Voice module for making calls and handling voice interactions
#[derive(Debug, Clone)]
pub struct VoiceModule {
    client: AfricasTalkingClient,
}

impl VoiceModule {
    pub(crate) fn new(client: AfricasTalkingClient) -> Self {
        Self { client }
    }

    /// Make an outbound call
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use africastalking::voice::*;
    /// # async fn make_outbound_call(voice: &VoiceModule) -> africastalking::Result<()> {
    /// let request = MakeCallRequest::new(
    ///     "+254711XXXYYY",
    ///     vec!["+254722XXXYYY", "+254733XXXYYY"]
    /// );
    ///
    /// let response = voice.make_call(request).await?;
    /// for entry in response.entries {
    ///     println!("Call to {}: {}", entry.phone_number, entry.status);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn make_call(&self, request: MakeCallRequest) -> Result<MakeCallResponse> {
        self.client.post_json("/call", &request).await
    }

    /// Get the number of queued calls for specific phone numbers
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use africastalking::voice::*;
    /// # async fn get_queue_status(voice: &VoiceModule) -> africastalking::Result<()> {
    /// let request = QueueStatusRequest::new(vec![
    ///     "+254711XXXYYY",
    ///     "+254722XXXYYY",
    /// ]);
    ///
    /// let response = voice.get_queued_calls(request).await?;
    /// println!("Queued calls: {}", response.num_queued_calls);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_queued_calls(
        &self,
        request: QueueStatusRequest,
    ) -> Result<QueueStatusResponse> {
        self.client.post_json("/queueStatus", &request).await
    }

    /// Upload a media file for use in voice calls
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use africastalking::voice::*;
    /// # async fn upload_media(voice: &VoiceModule) -> africastalking::Result<()> {
    /// let request = UploadMediaRequest::new(
    ///     "https://example.com/audio.mp3",
    ///     "+254711XXXYYY",
    /// );
    ///
    /// voice.upload_media(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload_media(&self, request: UploadMediaRequest) -> Result<UploadMediaResponse> {
        self.client.post_json("/mediaUpload", &request).await
    }
}

#[derive(Debug, Deserialize)]
pub struct VoiceCallback {
    #[serde(rename = "isActive")]
    pub is_active: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub direction: String,
    #[serde(rename = "callerNumber")]
    pub caller_number: String,
    #[serde(rename = "destinationNumber")]
    pub destination_number: String,
    #[serde(rename = "dtmfDigits", default)]
    pub dtmf_digits: String,
}

/// Request to make an outbound call
#[derive(Debug, Clone, Serialize)]
pub struct MakeCallRequest {
    /// Your AfricasTalking application username
    pub username: String,

    /// Your AfricasTalking phone number (in international format)
    #[serde(rename = "from")]
    pub call_from: String,

    /// Comma-separated recipients' phone numbers
    #[serde(rename = "to")]
    pub call_to: String,

    /// Optional client request ID for tagging
    #[serde(rename = "clientRequestId", skip_serializing_if = "Option::is_none")]
    pub client_request_id: Option<String>,
}

impl MakeCallRequest {
    /// Create a new call request
    ///
    /// # Arguments
    ///
    /// * `from` - Your AfricasTalking phone number (e.g., "+254711XXXYYY")
    /// * `to` - Vec of recipient phone numbers
    pub fn new(from: impl Into<String>, to: Vec<impl Into<String>>) -> Self {
        Self {
            username: String::new(), // Will be set by client
            call_from: from.into(),
            call_to: to
                .into_iter()
                .map(|s| s.into())
                .collect::<Vec<_>>()
                .join(","),
            client_request_id: None,
        }
    }

    /// Add a client request ID for tagging
    pub fn with_client_request_id(mut self, id: impl Into<String>) -> Self {
        self.client_request_id = Some(id.into());
        self
    }
}

/// Response from making a call
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MakeCallResponse {
    /// List of call entries, one per phone number
    pub entries: Vec<CallEntry>,

    /// Error message if the entire request failed
    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
}

/// Individual call entry in the response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallEntry {
    /// Phone number that was called
    pub phone_number: String,

    /// Status of the call request
    pub status: CallStatus,

    /// Unique session ID (None if error occurred)
    pub session_id: Option<String>,
}

/// Status of a call request
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum CallStatus {
    /// Call request accepted and queued
    Queued,
    /// Invalid phone number format
    InvalidPhoneNumber,
    /// Destination not supported
    DestinationNotSupported,
    /// Insufficient account balance
    InsufficientCredit,
}

impl fmt::Display for CallStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Queued => write!(f, "Queued"),
            Self::InvalidPhoneNumber => write!(f, "Invalid Phone Number"),
            Self::DestinationNotSupported => write!(f, "Destination Not Supported"),
            Self::InsufficientCredit => write!(f, "Insufficient Credit"),
        }
    }
}

/// Request to get queued calls status
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueStatusRequest {
    /// AfricasTalking Application username
    pub username: String,

    /// List of phone numbers to query
    pub phone_numbers: Vec<String>,
}

impl QueueStatusRequest {
    /// Create a new queue status request
    pub fn new(phone_numbers: Vec<impl Into<String>>) -> Self {
        Self {
            username: String::new(), // Will be set by client
            phone_numbers: phone_numbers.into_iter().map(|s| s.into()).collect(),
        }
    }
}

/// Response from queue status request
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct QueueStatusResponse {
    /// Status of the request
    pub status: String,

    /// Number of queued calls
    pub num_queued_calls: u32,

    /// List of phone numbers with their queue status
    pub phone_numbers: Vec<QueuedNumber>,

    /// Error message if request failed
    pub error_message: Option<String>,
}

/// Queued number details
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct QueuedNumber {
    /// Phone number
    pub phone_number: String,

    /// Number of queued calls for this number
    pub num_queued_calls: u32,
}

/// Request to upload media file
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadMediaRequest {
    /// AfricasTalking Application username
    pub username: String,

    /// HTTPS URL to the media file
    pub url: String,

    /// Phone number associated with upload
    pub phone_number: String,
}

impl UploadMediaRequest {
    /// Create a new media upload request
    ///
    /// # Arguments
    ///
    /// * `url` - HTTPS URL to media file
    /// * `phone_number` - Associated phone number
    pub fn new(url: impl Into<String>, phone_number: impl Into<String>) -> Self {
        Self {
            username: String::new(), // Will be set by client
            url: url.into(),
            phone_number: phone_number.into(),
        }
    }
}

/// Response from media upload
#[derive(Debug, Clone, Deserialize)]
pub struct UploadMediaResponse {
    /// Status message
    pub status: String,

    /// Error message if upload failed
    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
}

/// ActionBuilder for creating XML voice action responses
///
/// Used to construct XML that tells AT how to handle a call.
///
/// # Example
///
/// ```
/// use africastalking::voice::{ActionBuilder, GetDigitsAction};
///
/// let xml = ActionBuilder::new()
///     .say("Welcome to our service", None)
///     .play("https://example.com/music.mp3")
///     .get_digits(
///         GetDigitsAction::new()
///             .say("Press 1 for support", None)
///             .num_digits(1)
///     )
///     .build();
///
/// assert!(xml.contains("<Say>Welcome to our service</Say>"));
/// ```
#[derive(Debug, Clone)]
pub struct ActionBuilder {
    xml: String,
    finalized: bool,
}

impl ActionBuilder {
    /// Create a new action builder
    pub fn new() -> Self {
        Self {
            xml: r#"<?xml version="1.0" encoding="UTF-8"?><Response>"#.to_string(),
            finalized: false,
        }
    }

    /// Text-to-speech action
    ///
    /// # Arguments
    ///
    /// * `text` - Text to speak
    /// * `attributes` - Optional attributes (voice, playBeep)
    pub fn say(mut self, text: impl Into<String>, attributes: Option<SayAttributes>) -> Self {
        self.ensure_not_finalized();

        self.xml.push_str("<Say");
        if let Some(attrs) = attributes {
            if let Some(voice) = attrs.voice {
                self.xml.push_str(&format!(r#" voice="{}""#, voice));
            }
            if let Some(beep) = attrs.play_beep {
                self.xml.push_str(&format!(r#" playBeep="{}""#, beep));
            }
        }
        self.xml.push('>');
        self.xml.push_str(&escape_xml(&text.into()));
        self.xml.push_str("</Say>");
        self
    }

    /// Play audio file
    pub fn play(mut self, url: impl Into<String>) -> Self {
        self.ensure_not_finalized();
        self.xml
            .push_str(&format!(r#"<Play url="{}"/>"#, url.into()));
        self
    }

    /// Get DTMF digits from user
    pub fn get_digits(mut self, action: GetDigitsAction) -> Self {
        self.ensure_not_finalized();
        self.xml.push_str(&action.to_xml());
        self
    }

    /// Dial phone numbers or SIP addresses
    pub fn dial(mut self, action: DialAction) -> Self {
        self.ensure_not_finalized();
        self.xml.push_str(&action.to_xml());
        self
    }

    /// Record the call
    pub fn record(mut self, action: RecordAction) -> Self {
        self.ensure_not_finalized();
        self.xml.push_str(&action.to_xml());
        self
    }

    /// Add caller to a queue
    pub fn enqueue(mut self, attributes: Option<EnqueueAttributes>) -> Self {
        self.ensure_not_finalized();

        self.xml.push_str("<Enqueue");
        if let Some(attrs) = attributes {
            if let Some(music) = attrs.hold_music {
                self.xml.push_str(&format!(r#" holdMusic="{}""#, music));
            }
            if let Some(name) = attrs.name {
                self.xml.push_str(&format!(r#" name="{}""#, name));
            }
        }
        self.xml.push_str("/>");
        self
    }

    /// Remove caller from queue and bridge to agent
    pub fn dequeue(mut self, phone_number: impl Into<String>, name: Option<String>) -> Self {
        self.ensure_not_finalized();

        self.xml.push_str(&format!(
            r#"<Dequeue phoneNumber="{}""#,
            phone_number.into()
        ));
        if let Some(n) = name {
            self.xml.push_str(&format!(r#" name="{}""#, n));
        }
        self.xml.push_str("/>");
        self
    }

    /// Redirect to another URL
    pub fn redirect(mut self, url: impl Into<String>) -> Self {
        self.ensure_not_finalized();
        self.xml
            .push_str(&format!("<Redirect>{}</Redirect>", url.into()));
        self
    }

    /// Add caller to a conference
    pub fn conference(mut self) -> Self {
        self.ensure_not_finalized();
        self.xml.push_str("<Conference/>");
        self
    }

    /// Reject the call
    pub fn reject(mut self) -> Self {
        self.ensure_not_finalized();
        self.xml.push_str("<Reject/>");
        self
    }

    /// Build the final XML string
    pub fn build(mut self) -> String {
        self.finalized = true;
        self.xml.push_str("</Response>");
        self.xml
    }

    fn ensure_not_finalized(&self) {
        if self.finalized {
            panic!("ActionBuilder has already been finalized");
        }
    }
}

impl Default for ActionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Action Attributes and Helper Types

/// Attributes for Say action
#[derive(Debug, Clone)]
pub struct SayAttributes {
    /// Voice to use (male/female)
    pub voice: Option<String>,
    /// Play beep before speaking
    pub play_beep: Option<bool>,
}

/// GetDigits action builder
#[derive(Debug, Clone)]
pub struct GetDigitsAction {
    finish_on_key: Option<char>,
    num_digits: Option<u32>,
    timeout: Option<u32>,
    callback_url: Option<String>,
    say_text: Option<(String, Option<SayAttributes>)>,
    play_url: Option<String>,
}

impl GetDigitsAction {
    pub fn new() -> Self {
        Self {
            finish_on_key: None,
            num_digits: None,
            timeout: None,
            callback_url: None,
            say_text: None,
            play_url: None,
        }
    }

    pub fn finish_on_key(mut self, key: char) -> Self {
        self.finish_on_key = Some(key);
        self
    }

    pub fn num_digits(mut self, num: u32) -> Self {
        self.num_digits = Some(num);
        self
    }

    pub fn timeout(mut self, seconds: u32) -> Self {
        self.timeout = Some(seconds);
        self
    }

    pub fn callback_url(mut self, url: impl Into<String>) -> Self {
        self.callback_url = Some(url.into());
        self
    }

    pub fn say(mut self, text: impl Into<String>, attrs: Option<SayAttributes>) -> Self {
        self.say_text = Some((text.into(), attrs));
        self
    }

    pub fn play(mut self, url: impl Into<String>) -> Self {
        self.play_url = Some(url.into());
        self
    }

    fn to_xml(&self) -> String {
        let mut xml = String::from("<GetDigits");

        if let Some(key) = self.finish_on_key {
            xml.push_str(&format!(r#" finishOnKey="{}""#, key));
        }
        if let Some(num) = self.num_digits {
            xml.push_str(&format!(r#" numDigits="{}""#, num));
        }
        if let Some(timeout) = self.timeout {
            xml.push_str(&format!(r#" timeout="{}""#, timeout));
        }
        if let Some(ref url) = self.callback_url {
            xml.push_str(&format!(r#" callbackUrl="{}""#, url));
        }

        xml.push('>');

        if let Some((text, attrs)) = &self.say_text {
            xml.push_str("<Say");
            if let Some(attrs) = attrs {
                if let Some(ref voice) = attrs.voice {
                    xml.push_str(&format!(r#" voice="{}""#, voice));
                }
                if let Some(beep) = attrs.play_beep {
                    xml.push_str(&format!(r#" playBeep="{}""#, beep));
                }
            }
            xml.push('>');
            xml.push_str(&escape_xml(text));
            xml.push_str("</Say>");
        } else if let Some(ref url) = self.play_url {
            xml.push_str(&format!(r#"<Play url="{}"/>"#, url));
        }

        xml.push_str("</GetDigits>");
        xml
    }
}

impl Default for GetDigitsAction {
    fn default() -> Self {
        Self::new()
    }
}

/// Dial action builder
#[derive(Debug, Clone)]
pub struct DialAction {
    phone_numbers: String,
    caller_id: Option<String>,
    record: Option<bool>,
    sequential: Option<bool>,
    max_duration: Option<u32>,
    ring_back_tone: Option<String>,
}

impl DialAction {
    pub fn new(phone_numbers: Vec<impl Into<String>>) -> Self {
        Self {
            phone_numbers: phone_numbers
                .into_iter()
                .map(|s| s.into())
                .collect::<Vec<_>>()
                .join(","),
            caller_id: None,
            record: None,
            sequential: None,
            max_duration: None,
            ring_back_tone: None,
        }
    }

    pub fn caller_id(mut self, id: impl Into<String>) -> Self {
        self.caller_id = Some(id.into());
        self
    }

    pub fn record(mut self, enable: bool) -> Self {
        self.record = Some(enable);
        self
    }

    pub fn sequential(mut self, enable: bool) -> Self {
        self.sequential = Some(enable);
        self
    }

    pub fn max_duration(mut self, seconds: u32) -> Self {
        self.max_duration = Some(seconds);
        self
    }

    pub fn ring_back_tone(mut self, url: impl Into<String>) -> Self {
        self.ring_back_tone = Some(url.into());
        self
    }

    fn to_xml(&self) -> String {
        let mut xml = format!(r#"<Dial phoneNumbers="{}""#, self.phone_numbers);

        if let Some(ref id) = self.caller_id {
            xml.push_str(&format!(r#" callerId="{}""#, id));
        }
        if let Some(rec) = self.record {
            xml.push_str(&format!(r#" record="{}""#, rec));
        }
        if let Some(seq) = self.sequential {
            xml.push_str(&format!(r#" sequential="{}""#, seq));
        }
        if let Some(dur) = self.max_duration {
            xml.push_str(&format!(r#" maxDuration="{}""#, dur));
        }
        if let Some(ref tone) = self.ring_back_tone {
            xml.push_str(&format!(r#" ringBackTone="{}""#, tone));
        }

        xml.push_str("/>");
        xml
    }
}

/// Record action builder
#[derive(Debug, Clone)]
pub struct RecordAction {
    finish_on_key: Option<char>,
    max_length: Option<u32>,
    timeout: Option<u32>,
    play_beep: Option<bool>,
    trim_silence: Option<bool>,
    callback_url: Option<String>,
    say_text: Option<(String, Option<SayAttributes>)>,
    play_url: Option<String>,
}

impl RecordAction {
    pub fn new() -> Self {
        Self {
            finish_on_key: None,
            max_length: None,
            timeout: None,
            play_beep: None,
            trim_silence: None,
            callback_url: None,
            say_text: None,
            play_url: None,
        }
    }

    pub fn finish_on_key(mut self, key: char) -> Self {
        self.finish_on_key = Some(key);
        self
    }

    pub fn max_length(mut self, seconds: u32) -> Self {
        self.max_length = Some(seconds);
        self
    }

    pub fn timeout(mut self, seconds: u32) -> Self {
        self.timeout = Some(seconds);
        self
    }

    pub fn play_beep(mut self, enable: bool) -> Self {
        self.play_beep = Some(enable);
        self
    }

    pub fn trim_silence(mut self, enable: bool) -> Self {
        self.trim_silence = Some(enable);
        self
    }

    pub fn callback_url(mut self, url: impl Into<String>) -> Self {
        self.callback_url = Some(url.into());
        self
    }

    pub fn say(mut self, text: impl Into<String>, attrs: Option<SayAttributes>) -> Self {
        self.say_text = Some((text.into(), attrs));
        self
    }

    pub fn play(mut self, url: impl Into<String>) -> Self {
        self.play_url = Some(url.into());
        self
    }

    fn to_xml(&self) -> String {
        let mut xml = String::from("<Record");

        if let Some(key) = self.finish_on_key {
            xml.push_str(&format!(r#" finishOnKey="{}""#, key));
        }
        if let Some(len) = self.max_length {
            xml.push_str(&format!(r#" maxLength="{}""#, len));
        }
        if let Some(timeout) = self.timeout {
            xml.push_str(&format!(r#" timeout="{}""#, timeout));
        }
        if let Some(beep) = self.play_beep {
            xml.push_str(&format!(r#" playBeep="{}""#, beep));
        }
        if let Some(trim) = self.trim_silence {
            xml.push_str(&format!(r#" trimSilence="{}""#, trim));
        }
        if let Some(ref url) = self.callback_url {
            xml.push_str(&format!(r#" callbackUrl="{}""#, url));
        }

        xml.push('>');

        if let Some((text, attrs)) = &self.say_text {
            xml.push_str("<Say");
            if let Some(attrs) = attrs {
                if let Some(ref voice) = attrs.voice {
                    xml.push_str(&format!(r#" voice="{}""#, voice));
                }
                if let Some(beep) = attrs.play_beep {
                    xml.push_str(&format!(r#" playBeep="{}""#, beep));
                }
            }
            xml.push('>');
            xml.push_str(&escape_xml(text));
            xml.push_str("</Say>");
        } else if let Some(ref url) = self.play_url {
            xml.push_str(&format!(r#"<Play url="{}"/>"#, url));
        }

        xml.push_str("</Record>");
        xml
    }
}

impl Default for RecordAction {
    fn default() -> Self {
        Self::new()
    }
}

/// Attributes for Enqueue action
#[derive(Debug, Clone)]
pub struct EnqueueAttributes {
    pub hold_music: Option<String>,
    pub name: Option<String>,
}

/// Voice Module Helper Functions

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_builder_say() {
        let xml = ActionBuilder::new().say("Hello World", None).build();

        assert!(xml.contains(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(xml.contains("<Response>"));
        assert!(xml.contains("<Say>Hello World</Say>"));
        assert!(xml.contains("</Response>"));
    }

    #[test]
    fn test_action_builder_play() {
        let xml = ActionBuilder::new()
            .play("https://example.com/audio.mp3")
            .build();

        assert!(xml.contains(r#"<Play url="https://example.com/audio.mp3"/>"#));
    }

    #[test]
    fn test_action_builder_get_digits() {
        let xml = ActionBuilder::new()
            .get_digits(
                GetDigitsAction::new()
                    .say("Press 1", None)
                    .num_digits(1)
                    .finish_on_key('#'),
            )
            .build();

        assert!(xml.contains(r#"<GetDigits"#));
        assert!(xml.contains(r#"numDigits="1""#));
        assert!(xml.contains("finishOnKey=\"#\""));
        assert!(xml.contains("<Say>Press 1</Say>"));
    }

    #[test]
    fn test_action_builder_dial() {
        let xml = ActionBuilder::new()
            .dial(DialAction::new(vec!["+254711XXXYYY", "+254722XXXYYY"]).record(true))
            .build();

        assert!(xml.contains(r#"<Dial phoneNumbers="+254711XXXYYY,+254722XXXYYY""#));
        assert!(xml.contains(r#"record="true""#));
    }

    #[test]
    fn test_xml_escaping() {
        let xml = ActionBuilder::new().say("Test <>&\"'", None).build();

        assert!(xml.contains("Test &lt;&gt;&amp;&quot;&apos;"));
    }

    #[test]
    #[should_panic(expected = "finalized")]
    fn test_finalized_builder_panics() {
        let builder = ActionBuilder::new().build();
        let _ = ActionBuilder {
            xml: builder,
            finalized: true,
        }
        .say("test", None);
    }
}
