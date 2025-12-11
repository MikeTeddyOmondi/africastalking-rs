//! Example: Advanced USSD with session state management
//!
//! This example demonstrates how to maintain session state across multiple
//! user interactions using Redis or in-memory storage.

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Form, Router,
};
use africastalking_rs::ussd::{UssdRequest, UssdResponse, UssdMenu};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Session data that persists across USSD interactions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SessionData {
    /// Current state in the flow
    state: FlowState,
    /// Collected data from the user
    data: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum FlowState {
    Initial,
    CollectingName,
    CollectingAmount,
    Confirming,
    Complete,
}

impl Default for FlowState {
    fn default() -> Self {
        Self::Initial
    }
}

/// Application state with session storage
#[derive(Clone)]
struct AppState {
    // In production, use Redis or another persistent store
    sessions: Arc<RwLock<HashMap<String, SessionData>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get_session(&self, session_id: &str) -> SessionData {
        self.sessions
            .read()
            .await
            .get(session_id)
            .cloned()
            .unwrap_or_default()
    }

    async fn update_session(&self, session_id: &str, data: SessionData) {
        self.sessions
            .write()
            .await
            .insert(session_id.to_string(), data);
    }

    async fn clear_session(&self, session_id: &str) {
        self.sessions.write().await.remove(session_id);
    }
}

#[tokio::main]
async fn main() {
    let state = AppState::new();

    let app = Router::new()
        .route("/ussd", post(handle_stateful_ussd))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4949")
        .await
        .unwrap();
    println!("Server running on http://localhost:4949");

    axum::serve(listener, app).await.unwrap();
}

/// Stateful USSD handler with session management
async fn handle_stateful_ussd(
    State(state): State<AppState>,
    Form(request): Form<UssdRequest>,
) -> Response {
    // Get or create session data
    let mut session = state.get_session(&request.session_id).await;

    // Process based on current state
    let response = match session.state {
        FlowState::Initial => {
            session.state = FlowState::CollectingName;
            state.update_session(&request.session_id, session).await;

            UssdResponse::continues("Welcome to Money Transfer\n\nPlease enter recipient's name:")
        }

        FlowState::CollectingName => {
            if let Some(name) = request.current_input() {
                if name.trim().is_empty() {
                    UssdResponse::continues("Name cannot be empty.\nPlease enter recipient's name:")
                } else {
                    session.data.insert("recipient_name".to_string(), name.to_string());
                    session.state = FlowState::CollectingAmount;
                    state.update_session(&request.session_id, session).await;

                    UssdResponse::continues("Enter amount to send (KES):")
                }
            } else {
                UssdResponse::ends("Invalid input. Please try again.")
            }
        }

        FlowState::CollectingAmount => {
            if let Some(amount_str) = request.current_input() {
                match amount_str.parse::<f64>() {
                    Ok(amount) if amount > 0.0 && amount <= 100000.0 => {
                        session.data.insert("amount".to_string(), amount.to_string());
                        session.state = FlowState::Confirming;
                        state.update_session(&request.session_id, session.clone()).await;

                        let name = session.data.get("recipient_name").unwrap();
                        UssdMenu::new(format!(
                            "Confirm transfer:\n\nRecipient: {}\nAmount: KES {:.2}\n\nConfirm?",
                            name, amount
                        ))
                        .add_option("1", "Yes, send money")
                        .add_option("2", "Cancel")
                        .build_continue()
                    }
                    Ok(_) => {
                        UssdResponse::continues("Amount must be between 1 and 100,000.\nEnter amount:")
                    }
                    Err(_) => UssdResponse::continues("Invalid amount.\nEnter amount (KES):"),
                }
            } else {
                UssdResponse::ends("Invalid input. Please try again.")
            }
        }

        FlowState::Confirming => {
            if let Some(choice) = request.current_input() {
                match choice {
                    "1" => {
                        // Process the transaction
                        let name = session.data.get("recipient_name").unwrap();
                        let amount: f64 = session.data.get("amount").unwrap().parse().unwrap();

                        // Clear session after completion
                        state.clear_session(&request.session_id).await;

                        // In production, process the actual transaction here
                        UssdResponse::ends(format!(
                            "Success!\n\nSent KES {:.2} to {}\n\nTransaction ID: TXN{}",
                            amount,
                            name,
                            chrono::Utc::now().timestamp()
                        ))
                    }
                    "2" => {
                        state.clear_session(&request.session_id).await;
                        UssdResponse::ends("Transaction cancelled.")
                    }
                    _ => UssdResponse::ends("Invalid option. Transaction cancelled."),
                }
            } else {
                UssdResponse::ends("Invalid input. Transaction cancelled.")
            }
        }

        FlowState::Complete => UssdResponse::ends("Session already completed."),
    };

    (
        [(axum::http::header::CONTENT_TYPE, "text/plain")],
        response.to_string(),
    )
        .into_response()
}
