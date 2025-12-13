//! USSD module implementation for AfricaTalking
//!
//! This module provides abstractions for handling USSD sessions and notifications.
//! It's designed to work with any Rust web framework (Axum, Actix, Rocket, etc.)
//!
//! # Examples
//!
//! ## Basic usage with Axum
//!
//! ```no_run
//! use axum::{Router, routing::post, Json, response::IntoResponse};
//! use africastalking::ussd::{UssdRequest, UssdResponse, UssdMenu};
//!
//! async fn handle_ussd(Json(payload): Json<UssdRequest>) -> impl IntoResponse {
//!     let response = match payload.text.as_str() {
//!         "" => UssdResponse::continues("CON Welcome\n1. Account\n2. Phone"),
//!         "1" => UssdResponse::continues("CON Choose:\n1. Account number"),
//!         "2" => UssdResponse::ends(format!("Your phone: {}", payload.phone_number)),
//!         "1*1" => UssdResponse::ends("Your account: ACC100101"),
//!         _ => UssdResponse::ends("Invalid option"),
//!     };
//!     
//!     response.to_string()
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

/// USSD request payload from Africa's Talking
///
/// This represents the data sent by Africa's Talking when a user
/// interacts with your USSD application.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UssdRequest {
    /// Unique session identifier maintained throughout the USSD session
    pub session_id: String,

    /// The USSD code assigned to your application (e.g., *384*123#)
    pub service_code: String,

    /// Phone number of the user interacting with the USSD application
    pub phone_number: String,

    /// User input concatenated with asterisks (*).
    /// Empty string for the first request in a session.
    /// Example progression: "" -> "1" -> "1*2" -> "1*2*3"
    pub text: String,

    /// Network code identifying the user's telco provider
    pub network_code: String,
}

impl UssdRequest {
    /// Creates a new USSD request (useful for testing)
    pub fn new(
        session_id: impl Into<String>,
        service_code: impl Into<String>,
        phone_number: impl Into<String>,
        text: impl Into<String>,
        network_code: impl Into<String>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            service_code: service_code.into(),
            phone_number: phone_number.into(),
            text: text.into(),
            network_code: network_code.into(),
        }
    }

    /// Checks if this is the first request in the session
    pub fn is_initial(&self) -> bool {
        self.text.is_empty()
    }

    /// Gets the navigation depth (number of choices made)
    pub fn depth(&self) -> usize {
        if self.text.is_empty() {
            0
        } else {
            self.text.matches('*').count() + 1
        }
    }

    /// Gets the most recent user input
    pub fn current_input(&self) -> Option<&str> {
        if self.text.is_empty() {
            None
        } else {
            self.text.split('*').last()
        }
    }

    /// Gets the full navigation path as a vector
    pub fn navigation_path(&self) -> Vec<&str> {
        if self.text.is_empty() {
            vec![]
        } else {
            self.text.split('*').collect()
        }
    }

    /// Checks if the current path matches a pattern
    pub fn matches_path(&self, pattern: &str) -> bool {
        self.text == pattern
    }

    /// Checks if the current path starts with a pattern
    pub fn starts_with_path(&self, pattern: &str) -> bool {
        self.text.starts_with(pattern)
    }
}

/// USSD response to send back to Africa's Talking
///
/// Responses must begin with either "CON" (continue session) or "END" (terminate session).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UssdResponse {
    /// The response type and message
    content: UssdResponseType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum UssdResponseType {
    /// Continue the session - expects more user input
    Continue(String),
    /// End the session - no more input expected
    End(String),
}

impl UssdResponse {
    /// Creates a response that continues the USSD session
    pub fn continues(message: impl Into<String>) -> Self {
        Self {
            content: UssdResponseType::Continue(message.into()),
        }
    }

    /// Creates a response that ends the USSD session
    pub fn ends(message: impl Into<String>) -> Self {
        Self {
            content: UssdResponseType::End(message.into()),
        }
    }

    /// Checks if this response continues the session
    pub fn is_continuing(&self) -> bool {
        matches!(self.content, UssdResponseType::Continue(_))
    }

    /// Checks if this response ends the session
    pub fn is_ending(&self) -> bool {
        matches!(self.content, UssdResponseType::End(_))
    }

    /// Gets the message content without the CON/END prefix
    pub fn message(&self) -> &str {
        match &self.content {
            UssdResponseType::Continue(msg) | UssdResponseType::End(msg) => msg,
        }
    }
}

impl fmt::Display for UssdResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.content {
            UssdResponseType::Continue(msg) => write!(f, "CON {}", msg),
            UssdResponseType::End(msg) => write!(f, "END {}", msg),
        }
    }
}

/// Builder for creating USSD menus
///
/// # Examples
///
/// ```
/// use africastalking::ussd::UssdMenu;
///
/// let menu = UssdMenu::new("What would you like to check?")
///     .add_option("1", "My account")
///     .add_option("2", "My phone number")
///     .build_continue();
/// ```
#[derive(Debug, Clone)]
pub struct UssdMenu {
    title: String,
    options: Vec<(String, String)>,
}

impl UssdMenu {
    /// Creates a new menu with a title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            options: Vec::new(),
        }
    }

    /// Adds an option to the menu
    pub fn add_option(mut self, key: impl Into<String>, label: impl Into<String>) -> Self {
        self.options.push((key.into(), label.into()));
        self
    }

    /// Adds multiple options at once
    pub fn add_options<I, K, L>(mut self, options: I) -> Self
    where
        I: IntoIterator<Item = (K, L)>,
        K: Into<String>,
        L: Into<String>,
    {
        for (key, label) in options {
            self.options.push((key.into(), label.into()));
        }
        self
    }

    /// Builds a response that continues the session
    pub fn build_continue(self) -> UssdResponse {
        UssdResponse::continues(self.format_menu())
    }

    /// Builds a response that ends the session
    pub fn build_end(self) -> UssdResponse {
        UssdResponse::ends(self.format_menu())
    }

    fn format_menu(&self) -> String {
        let mut result = self.title.clone();
        for (key, label) in &self.options {
            result.push_str(&format!("\n{}. {}", key, label));
        }
        result
    }
}

/// USSD notification received at the end of a session
///
/// Africa's Talking sends this to your notification callback URL
/// when a USSD session completes.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UssdNotification {
    /// Timestamp when the notification was sent (UTC)
    /// Format: yyyy-MM-dd HH:mm:ss
    pub date: String,

    /// Session identifier (same as in UssdRequest)
    pub session_id: String,

    /// The USSD code for this application
    pub service_code: String,

    /// Network code identifying the telco
    pub network_code: String,

    /// Phone number of the user
    pub phone_number: String,

    /// Session completion status
    pub status: UssdSessionStatus,

    /// Cost of the USSD session
    pub cost: String,

    /// Duration of the session in milliseconds
    #[serde(rename = "durationInMillis")]
    pub duration_in_millis: String,

    /// Number of steps the user went through
    pub hops_count: i32,

    /// All user inputs concatenated with asterisks
    pub input: String,

    /// Last response from your callback URL before session ended
    pub last_app_response: String,

    /// Error message for failed sessions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// Status of a completed USSD session
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum UssdSessionStatus {
    /// Session was terminated while more input was expected
    Incomplete,
    /// Session reached its expected end
    Success,
    /// Session terminated due to server error
    Failed,
}

impl fmt::Display for UssdSessionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Incomplete => write!(f, "Incomplete"),
            Self::Success => write!(f, "Success"),
            Self::Failed => write!(f, "Failed"),
        }
    }
}

/// Network code identifiers for various telcos across Africa
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkCode {
    // Ghana
    AirtelTigoGhana,
    VodafoneGhana,
    MtnGhana,

    // Nigeria
    AirtelNigeria,
    MtnNigeria,
    GloNigeria,
    EtisalatNigeria,

    // Rwanda
    MtnRwanda,
    TigoRwanda,
    AirtelRwanda,

    // Ethiopia
    EthioTelecom,

    // Kenya
    SafaricomKenya,
    AirtelKenya,
    OrangeKenya,
    EquitelKenya,

    // Tanzania
    TigoTanzania,
    VodacomTanzania,
    AirtelTanzania,

    // Uganda
    AirtelUganda,
    MtnUganda,
    AfricellUganda,

    // Zambia
    AirtelZambia,
    MtnZambia,

    // Malawi
    TnmMalawi,
    AirtelMalawi,

    // South Africa
    VodacomSouthAfrica,
    TelkomSouthAfrica,
    CellcSouthAfrica,
    MtnSouthAfrica,

    // Sandbox
    Athena,

    // Unknown/Custom
    Unknown(String),
}

impl NetworkCode {
    /// Parses a network code string into a NetworkCode enum
    pub fn from_code(code: &str) -> Self {
        match code {
            "62006" => Self::AirtelTigoGhana,
            "62002" => Self::VodafoneGhana,
            "62001" => Self::MtnGhana,
            "62120" => Self::AirtelNigeria,
            "62130" => Self::MtnNigeria,
            "62150" => Self::GloNigeria,
            "62160" => Self::EtisalatNigeria,
            "63510" => Self::MtnRwanda,
            "63513" => Self::TigoRwanda,
            "63514" => Self::AirtelRwanda,
            "63601" => Self::EthioTelecom,
            "63902" => Self::SafaricomKenya,
            "63903" => Self::AirtelKenya,
            "63907" => Self::OrangeKenya,
            "63999" => Self::EquitelKenya,
            "64002" => Self::TigoTanzania,
            "64004" => Self::VodacomTanzania,
            "64005" => Self::AirtelTanzania,
            "64101" => Self::AirtelUganda,
            "64110" => Self::MtnUganda,
            "64114" => Self::AfricellUganda,
            "64501" => Self::AirtelZambia,
            "64502" => Self::MtnZambia,
            "65001" => Self::TnmMalawi,
            "65010" => Self::AirtelMalawi,
            "65501" => Self::VodacomSouthAfrica,
            "65502" => Self::TelkomSouthAfrica,
            "65507" => Self::CellcSouthAfrica,
            "65510" => Self::MtnSouthAfrica,
            "99999" => Self::Athena,
            _ => Self::Unknown(code.to_string()),
        }
    }

    /// Gets the telco name
    pub fn name(&self) -> &str {
        match self {
            Self::AirtelTigoGhana => "AirtelTigo Ghana",
            Self::VodafoneGhana => "Vodafone Ghana",
            Self::MtnGhana => "MTN Ghana",
            Self::AirtelNigeria => "Airtel Nigeria",
            Self::MtnNigeria => "MTN Nigeria",
            Self::GloNigeria => "Glo Nigeria",
            Self::EtisalatNigeria => "Etisalat Nigeria",
            Self::MtnRwanda => "MTN Rwanda",
            Self::TigoRwanda => "Tigo Rwanda",
            Self::AirtelRwanda => "Airtel Rwanda",
            Self::EthioTelecom => "EthioTelecom Ethiopia",
            Self::SafaricomKenya => "Safaricom Kenya",
            Self::AirtelKenya => "Airtel Kenya",
            Self::OrangeKenya => "Orange Kenya",
            Self::EquitelKenya => "Equitel Kenya",
            Self::TigoTanzania => "Tigo Tanzania",
            Self::VodacomTanzania => "Vodacom Tanzania",
            Self::AirtelTanzania => "Airtel Tanzania",
            Self::AirtelUganda => "Airtel Uganda",
            Self::MtnUganda => "MTN Uganda",
            Self::AfricellUganda => "Africell Uganda",
            Self::AirtelZambia => "Airtel Zambia",
            Self::MtnZambia => "MTN Zambia",
            Self::TnmMalawi => "TNM Malawi",
            Self::AirtelMalawi => "Airtel Malawi",
            Self::VodacomSouthAfrica => "Vodacom South Africa",
            Self::TelkomSouthAfrica => "Telkom South Africa",
            Self::CellcSouthAfrica => "CellC South Africa",
            Self::MtnSouthAfrica => "MTN South Africa",
            Self::Athena => "Athena (Sandbox)",
            Self::Unknown(_) => "Unknown Network",
        }
    }

    /// Gets the country code
    pub fn country(&self) -> &str {
        match self {
            Self::AirtelTigoGhana | Self::VodafoneGhana | Self::MtnGhana => "Ghana",
            Self::AirtelNigeria | Self::MtnNigeria | Self::GloNigeria | Self::EtisalatNigeria => {
                "Nigeria"
            }
            Self::MtnRwanda | Self::TigoRwanda | Self::AirtelRwanda => "Rwanda",
            Self::EthioTelecom => "Ethiopia",
            Self::SafaricomKenya | Self::AirtelKenya | Self::OrangeKenya | Self::EquitelKenya => {
                "Kenya"
            }
            Self::TigoTanzania | Self::VodacomTanzania | Self::AirtelTanzania => "Tanzania",
            Self::AirtelUganda | Self::MtnUganda | Self::AfricellUganda => "Uganda",
            Self::AirtelZambia | Self::MtnZambia => "Zambia",
            Self::TnmMalawi | Self::AirtelMalawi => "Malawi",
            Self::VodacomSouthAfrica
            | Self::TelkomSouthAfrica
            | Self::CellcSouthAfrica
            | Self::MtnSouthAfrica => "South Africa",
            Self::Athena => "Sandbox",
            Self::Unknown(_) => "Unknown",
        }
    }
}

impl fmt::Display for NetworkCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ussd_request_initial() {
        let req = UssdRequest::new("session123", "*384*123#", "+254712345678", "", "63902");
        assert!(req.is_initial());
        assert_eq!(req.depth(), 0);
        assert_eq!(req.current_input(), None);
    }

    #[test]
    fn test_ussd_request_navigation() {
        let req = UssdRequest::new("session123", "*384*123#", "+254712345678", "1*2*3", "63902");
        assert!(!req.is_initial());
        assert_eq!(req.depth(), 3);
        assert_eq!(req.current_input(), Some("3"));
        assert_eq!(req.navigation_path(), vec!["1", "2", "3"]);
        assert!(req.matches_path("1*2*3"));
        assert!(req.starts_with_path("1*2"));
    }

    #[test]
    fn test_ussd_response_continue() {
        let response = UssdResponse::continues("Welcome to service");
        assert!(response.is_continuing());
        assert!(!response.is_ending());
        assert_eq!(response.to_string(), "CON Welcome to service");
    }

    #[test]
    fn test_ussd_response_end() {
        let response = UssdResponse::ends("Thank you");
        assert!(!response.is_continuing());
        assert!(response.is_ending());
        assert_eq!(response.to_string(), "END Thank you");
    }

    #[test]
    fn test_ussd_menu_builder() {
        let menu = UssdMenu::new("Main Menu")
            .add_option("1", "Account")
            .add_option("2", "Balance")
            .build_continue();

        assert_eq!(menu.to_string(), "CON Main Menu\n1. Account\n2. Balance");
    }

    #[test]
    fn test_network_code_parsing() {
        let code = NetworkCode::from_code("63902");
        assert_eq!(code, NetworkCode::SafaricomKenya);
        assert_eq!(code.name(), "Safaricom Kenya");
        assert_eq!(code.country(), "Kenya");
    }

    #[test]
    fn test_network_code_unknown() {
        let code = NetworkCode::from_code("12345");
        assert!(matches!(code, NetworkCode::Unknown(_)));
        assert_eq!(code.name(), "Unknown Network");
    }
}
