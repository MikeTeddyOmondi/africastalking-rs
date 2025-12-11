//! Example: USSD application with Axum
//!
//! This demonstrates building a complete USSD application using the africastalking-rs
//! USSD module with the Axum web framework.

use axum::{
    Form, Router, extract::State, response::{IntoResponse, Response}, routing::post
};
use africastalking::ussd::{UssdRequest, UssdResponse, UssdMenu, UssdNotification};
use std::sync::Arc;
use tokio::net::TcpListener;

/// Application state that can hold session data, database connections, etc.
#[derive(Clone)]
struct AppState {
    // Add your database connection, cache, etc. here
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {});

    let app = Router::new()
        .route("/ussd", post(handle_ussd))
        .route("/ussd/notify", post(handle_ussd_notification))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:4949").await.unwrap();
    println!("Server running on http://localhost:4949");
    
    axum::serve(listener, app).await.unwrap();
}

/// Main USSD handler - responds to user interactions
async fn handle_ussd(
    State(_state): State<Arc<AppState>>,
    Form(request): Form<UssdRequest>,
) -> Response {
    // Log the request for debugging
    println!("USSD Request: {:?}", request);

    // Route based on the user's navigation path
    let response = route_ussd_request(&request).await;

    // Return as plain text (required by Africa's Talking)
    (
        [(axum::http::header::CONTENT_TYPE, "text/plain")],
        response.to_string(),
    )
        .into_response()
}

/// Route USSD requests based on user input
async fn route_ussd_request(request: &UssdRequest) -> UssdResponse {
    // Initial request - show main menu
    if request.is_initial() {
        return UssdMenu::new("What would you like to check?")
            .add_option("1", "My account")
            .add_option("2", "My phone number")
            .add_option("3", "Help")
            .build_continue();
    }

    // Route based on the navigation path
    match request.text.as_str() {
        // User selected option 1 from main menu
        "1" => UssdMenu::new("Choose account information")
            .add_option("1", "Account number")
            .add_option("2", "Account balance")
            .build_continue(),

        // User selected option 2 from main menu - terminal response
        "2" => UssdResponse::ends(format!(
            "Your phone number is {}",
            request.phone_number
        )),

        // User selected option 3 - show help
        "3" => UssdResponse::ends("For support, call 0800-123-456 or visit our website."),

        // User navigated to 1 -> 1 (Account -> Account Number)
        "1*1" => {
            // In a real app, you'd fetch this from a database
            let account_number = fetch_account_number(&request.phone_number).await;
            UssdResponse::ends(format!("Your account number is {}", account_number))
        }

        // User navigated to 1 -> 2 (Account -> Balance)
        "1*2" => {
            let balance = fetch_account_balance(&request.phone_number).await;
            UssdResponse::ends(format!("Your account balance is KES {:.2}", balance))
        }

        // Invalid input
        _ => UssdResponse::ends("Invalid option. Please try again."),
    }
}

/// Handle end-of-session notifications
async fn handle_ussd_notification(
    State(_state): State<Arc<AppState>>,
    Form(notification): Form<UssdNotification>,
) -> Response {
    // Log the notification
    println!("USSD Notification: {:?}", notification);

    // You can store analytics, update user records, etc.
    match notification.status {
        africastalking::ussd::UssdSessionStatus::Success => {
            println!(
                "Successful session for {} - Duration: {}ms, Hops: {}",
                notification.phone_number, notification.duration_in_millis, notification.hops_count
            );
        }
        africastalking::ussd::UssdSessionStatus::Incomplete => {
            println!("Incomplete session for {}", notification.phone_number);
        }
        africastalking::ussd::UssdSessionStatus::Failed => {
            println!(
                "Failed session for {}: {:?}",
                notification.phone_number, notification.error_message
            );
        }
    }

    // Return 200 OK to acknowledge receipt
    axum::http::StatusCode::OK.into_response()
}

// Mock functions - replace with actual database queries
async fn fetch_account_number(phone_number: &str) -> String {
    // In production, query your database
    format!("ACC{}", &phone_number[phone_number.len() - 6..])
}

async fn fetch_account_balance(_phone_number: &str) -> f64 {
    // In production, query your database
    1234.56
}
