# USSD Module Documentation

Complete guide for building USSD applications with `africastalking-rs`.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Core Concepts](#core-concepts)
- [Examples](#examples)
- [Best Practices](#best-practices)
- [Testing](#testing)
- [Deployment](#deployment)

## Overview

The USSD module provides a type-safe, ergonomic API for building USSD applications that work with Africa's Talking. It's designed to be:

- **Framework-agnostic**: Works with Axum, Actix-web, Rocket, or any Rust web framework
- **Type-safe**: Leverages Rust's type system to prevent common errors
- **Ergonomic**: Simple, intuitive API with helpful builder patterns
- **Production-ready**: Includes session management, error handling, and testing utilities

## Quick Start

### Basic USSD Handler (Axum)

```rust
use axum::{routing::post, Form, Router};
use africastalking_rs::ussd::{UssdRequest, UssdResponse, UssdMenu};

async fn handle_ussd(Form(request): Form<UssdRequest>) -> String {
    let response = match request.text.as_str() {
        "" => UssdMenu::new("Welcome! Choose:")
            .add_option("1", "Account")
            .add_option("2", "Help")
            .build_continue(),
        
        "1" => UssdResponse::ends("Your account: ACC12345"),
        "2" => UssdResponse::ends("Call 0800-123-456"),
        
        _ => UssdResponse::ends("Invalid option"),
    };
    
    response.to_string()
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/ussd", post(handle_ussd));
    // ... serve app
}
```

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
africastalking-rs = "0.1"
axum = "0.7"  # or your preferred framework
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

## Core Concepts

### UssdRequest

The `UssdRequest` struct represents incoming USSD interactions:

```rust
pub struct UssdRequest {
    pub session_id: String,     // Unique session identifier
    pub service_code: String,   // Your USSD code (e.g., *384*123#)
    pub phone_number: String,   // User's phone number
    pub text: String,           // User input (e.g., "1*2*3")
    pub network_code: String,   // Telco identifier
}
```

**Helper methods:**

```rust
// Check if this is the first request
if request.is_initial() {
    // Show main menu
}

// Get navigation depth (number of choices made)
let depth = request.depth();  // 0, 1, 2, 3...

// Get the most recent input
if let Some(input) = request.current_input() {
    // Process latest choice
}

// Get full navigation path
let path = request.navigation_path();  // ["1", "2", "3"]

// Match specific paths
if request.matches_path("1*2*3") {
    // Handle this exact path
}

// Check path prefix
if request.starts_with_path("1*2") {
    // Handle all paths starting with "1*2"
}
```

### UssdResponse

Responses must start with either `CON` (continue) or `END` (terminate):

```rust
// Continue session - expects more input
let response = UssdResponse::continues("Enter your PIN:");

// End session - final message
let response = UssdResponse::ends("Transaction successful!");

// Check response type
if response.is_continuing() { /* ... */ }
if response.is_ending() { /* ... */ }

// Get the formatted response string
let output = response.to_string();  // "CON Enter your PIN:"
```

### UssdMenu

Builder pattern for creating menus:

```rust
let menu = UssdMenu::new("Main Menu")
    .add_option("1", "Check Balance")
    .add_option("2", "Send Money")
    .add_option("3", "Buy Airtime")
    .build_continue();

// Add multiple options at once
let menu = UssdMenu::new("Services")
    .add_options(vec![
        ("1", "Option A"),
        ("2", "Option B"),
        ("3", "Option C"),
    ])
    .build_continue();

// Build as terminal response
let menu = UssdMenu::new("Thank you!")
    .add_option("1", "Rate us")
    .build_end();
```

### UssdNotification

End-of-session notifications:

```rust
#[derive(Deserialize)]
pub struct UssdNotification {
    pub date: String,
    pub session_id: String,
    pub service_code: String,
    pub network_code: String,
    pub phone_number: String,
    pub status: UssdSessionStatus,  // Success, Incomplete, Failed
    pub cost: String,
    pub duration_in_millis: String,
    pub hops_count: i32,
    pub input: String,
    pub last_app_response: String,
    pub error_message: Option<String>,
}
```

## Examples

### 1. Simple Menu Navigation

```rust
async fn handle_ussd(Json(req): Json<UssdRequest>) -> String {
    let response = match req.text.as_str() {
        // Level 0: Main menu
        "" => UssdMenu::new("Choose service:")
            .add_option("1", "Banking")
            .add_option("2", "Airtime")
            .build_continue(),
        
        // Level 1: Banking submenu
        "1" => UssdMenu::new("Banking:")
            .add_option("1", "Balance")
            .add_option("2", "Statement")
            .build_continue(),
        
        // Level 2: Show balance
        "1*1" => {
            let balance = get_balance(&req.phone_number).await;
            UssdResponse::ends(format!("Balance: KES {:.2}", balance))
        }
        
        // Level 2: Send statement
        "1*2" => UssdResponse::ends("Statement sent via SMS"),
        
        // Level 1: Airtime
        "2" => UssdResponse::continues("Enter amount (KES):"),
        
        _ => UssdResponse::ends("Invalid option"),
    };
    
    response.to_string()
}
```

### 2. Collecting User Input

```rust
async fn handle_ussd(Json(req): Json<UssdRequest>) -> String {
    let response = match req.depth() {
        // Step 1: Ask for recipient
        0 => UssdResponse::continues("Enter recipient phone:"),
        
        // Step 2: Ask for amount
        1 => {
            if is_valid_phone(req.current_input().unwrap()) {
                UssdResponse::continues("Enter amount (KES):")
            } else {
                UssdResponse::ends("Invalid phone number")
            }
        }
        
        // Step 3: Confirm and process
        2 => {
            match req.current_input().unwrap().parse::<f64>() {
                Ok(amount) if amount > 0.0 => {
                    let path = req.navigation_path();
                    let recipient = path[0];
                    process_transfer(recipient, amount).await;
                    UssdResponse::ends("Transfer successful!")
                }
                _ => UssdResponse::ends("Invalid amount"),
            }
        }
        
        _ => UssdResponse::ends("Error occurred"),
    };
    
    response.to_string()
}
```

### 3. Session State Management

For complex flows, maintain state between requests:

```rust
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Clone)]
struct SessionData {
    step: Step,
    data: HashMap<String, String>,
}

#[derive(Clone)]
enum Step {
    Initial,
    CollectingName,
    CollectingAmount,
    Confirming,
}

type SessionStore = Arc<RwLock<HashMap<String, SessionData>>>;

async fn handle_ussd(
    State(store): State<SessionStore>,
    Json(req): Json<UssdRequest>,
) -> String {
    let mut sessions = store.write().await;
    let session = sessions.entry(req.session_id.clone())
        .or_insert_with(|| SessionData {
            step: Step::Initial,
            data: HashMap::new(),
        });
    
    let response = match session.step {
        Step::Initial => {
            session.step = Step::CollectingName;
            UssdResponse::continues("Enter recipient name:")
        }
        Step::CollectingName => {
            if let Some(name) = req.current_input() {
                session.data.insert("name".into(), name.into());
                session.step = Step::CollectingAmount;
                UssdResponse::continues("Enter amount:")
            } else {
                UssdResponse::ends("Invalid input")
            }
        }
        // ... handle other steps
    };
    
    response.to_string()
}
```

### 4. Handling Notifications

```rust
async fn handle_notification(
    Json(notification): Json<UssdNotification>,
) -> StatusCode {
    match notification.status {
        UssdSessionStatus::Success => {
            log_successful_session(&notification).await;
        }
        UssdSessionStatus::Failed => {
            log::error!("Session failed: {:?}", notification.error_message);
            alert_support_team(&notification).await;
        }
        UssdSessionStatus::Incomplete => {
            track_dropout(&notification).await;
        }
    }
    
    StatusCode::OK
}
```

### 5. Router Pattern (Advanced)

For large applications, use a router:

```rust
struct UssdRouter {
    routes: Vec<(Box<dyn Fn(&str) -> bool>, Handler)>,
}

impl UssdRouter {
    fn route(&self, req: &UssdRequest) -> UssdResponse {
        for (matcher, handler) in &self.routes {
            if matcher(&req.text) {
                return handler(req);
            }
        }
        UssdResponse::ends("No route found")
    }
}

fn create_router() -> UssdRouter {
    UssdRouter::new()
        .add_route(|t| t.is_empty(), initial_menu)
        .add_route(|t| t == "1", banking_menu)
        .add_route(|t| t.starts_with("1*"), handle_banking)
        .add_route(|t| t == "2", services_menu)
        // ... more routes
}
```

## Best Practices

### 1. Keep Menus Simple

- Maximum 6-8 options per menu
- Use clear, concise labels
- Avoid special characters (telcos can't render them)

```rust
// Good
UssdMenu::new("Services:")
    .add_option("1", "Balance")
    .add_option("2", "Statement")

// Bad - too many options, unclear labels
UssdMenu::new("Services:")
    .add_option("1", "Check account balance & history")
    .add_option("2", "View statement (last 30 days)")
    .add_option("3", "Transfer $$$ to another account")
    // ... 10 more options
```

### 2. Validate User Input

Always validate before processing:

```rust
match req.current_input() {
    Some(input) if input.len() == 10 && input.chars().all(|c| c.is_numeric()) => {
        // Valid phone number
    }
    _ => return UssdResponse::ends("Invalid phone number"),
}
```

### 3. Provide Clear Error Messages

```rust
// Good - specific and helpful
UssdResponse::ends("Amount must be between 1 and 50,000 KES")

// Bad - vague
UssdResponse::ends("Error")
```

### 4. Handle Edge Cases

```rust
// Timeout protection
if req.depth() > 10 {
    return UssdResponse::ends("Session too long. Please start again.");
}

// Empty input
if req.current_input().map(|s| s.trim().is_empty()).unwrap_or(true) {
    return UssdResponse::ends("No input received");
}
```

### 5. Session Timeout

USSD sessions timeout after ~30 seconds of inactivity. Design flows accordingly:

```rust
// Bad - collecting too much data
// Step 1: Enter name
// Step 2: Enter age
// Step 3: Enter address
// Step 4: Enter email
// Step 5: Enter phone
// Step 6: Confirm  <- User likely timed out

// Good - collect minimal data
// Step 1: Enter phone
// Step 2: Confirm & complete
// Send detailed form via SMS link
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_request() {
        let req = UssdRequest::new(
            "sess123",
            "*123#",
            "+254700000000",
            "",
            "63902"
        );
        
        assert!(req.is_initial());
        assert_eq!(req.depth(), 0);
    }

    #[test]
    fn test_navigation() {
        let req = UssdRequest::new(
            "sess123",
            "*123#",
            "+254700000000",
            "1*2*3",
            "63902"
        );
        
        assert_eq!(req.depth(), 3);
        assert_eq!(req.current_input(), Some("3"));
        assert!(req.matches_path("1*2*3"));
    }

    #[tokio::test]
    async fn test_handler() {
        let req = UssdRequest::new(
            "test",
            "*123#",
            "+254700000000",
            "1",
            "63902"
        );
        
        let response = handle_ussd(Json(req)).await;
        assert!(response.starts_with("CON"));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_flow() {
    let app = create_app();
    
    // Initial request
    let req1 = UssdRequest::new("sess1", "*123#", "+254700000000", "", "63902");
    let resp1 = send_request(&app, req1).await;
    assert!(resp1.contains("Main Menu"));
    
    // Select option 1
    let req2 = UssdRequest::new("sess1", "*123#", "+254700000000", "1", "63902");
    let resp2 = send_request(&app, req2).await;
    assert!(resp2.contains("Balance"));
}
```

## Deployment

### Production Checklist

- [ ] Set up HTTPS (required by Africa's Talking)
- [ ] Configure callback URLs in AT dashboard
- [ ] Set up session storage (Redis recommended)
- [ ] Implement logging and monitoring
- [ ] Add rate limiting
- [ ] Test with actual USSD codes
- [ ] Monitor notification endpoint

### Example Docker Deployment

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/ussd-app /usr/local/bin/
EXPOSE 3000
CMD ["ussd-app"]
```

### Environment Configuration

```bash
# .env
USSD_CALLBACK_URL=https://yourdomain.com/ussd
USSD_NOTIFICATION_URL=https://yourdomain.com/ussd/notify
AT_API_KEY=your_api_key
AT_USERNAME=your_username
REDIS_URL=redis://localhost:6379
```

## Network Codes

The module includes all Africa's Talking network codes:

```rust
use africastalking_rs::ussd::NetworkCode;

let code = NetworkCode::from_code("63902");
println!("{}", code.name());     // "Safaricom Kenya"
println!("{}", code.country());  // "Kenya"
```

Supported networks span Ghana, Nigeria, Rwanda, Ethiopia, Kenya, Tanzania, Uganda, Zambia, Malawi, and South Africa.

## Additional Resources

- [Africa's Talking USSD Docs](https://developers.africastalking.com/docs/ussd)
- [Crate Repository](https://github.com/MikeTeddyOmondi/africastalking-rs)
- [API Reference](https://docs.rs/africastalking-rs)

## License

This module is part of the africastalking-rs crate.
