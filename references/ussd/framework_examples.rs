//! Example: USSD with Actix-web
//!
//! This demonstrates using the USSD module with Actix-web framework

use actix_web::{web, App, HttpResponse, HttpServer, Result};
use africastalking::ussd::{UssdRequest, UssdResponse, UssdMenu, UssdNotification};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server running on http://localhost:4949");

    HttpServer::new(|| {
        App::new()
            .route("/ussd", web::post().to(handle_ussd))
            .route("/ussd/notify", web::post().to(handle_notification))
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}

async fn handle_ussd(request: web::Json<UssdRequest>) -> Result<HttpResponse> {
    let response = match request.text.as_str() {
        "" => UssdMenu::new("Welcome! Choose an option:")
            .add_option("1", "Check Balance")
            .add_option("2", "Send Money")
            .add_option("3", "Buy Airtime")
            .build_continue(),

        "1" => {
            let balance = get_user_balance(&request.phone_number).await;
            UssdResponse::ends(format!("Your balance is KES {:.2}", balance))
        }

        "2" => UssdMenu::new("Enter recipient phone number:")
            .build_continue(),

        "3" => UssdResponse::continues("Enter amount for airtime (KES):"),

        _ => UssdResponse::ends("Invalid option selected."),
    };

    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(response.to_string()))
}

async fn handle_notification(notification: web::Json<UssdNotification>) -> Result<HttpResponse> {
    println!("Session completed: {:?}", notification);
    Ok(HttpResponse::Ok().finish())
}

async fn get_user_balance(_phone: &str) -> f64 {
    1500.00 // Mock data
}

// ============================================================================
// Example: USSD with Rocket
// ============================================================================

#[cfg(feature = "rocket-example")]
mod rocket_example {
    use rocket::{post, routes, serde::json::Json, State};
    use africastalking_rs::ussd::{UssdRequest, UssdResponse, UssdMenu};

    #[post("/ussd", data = "<request>")]
    fn handle_ussd(request: Json<UssdRequest>) -> String {
        let response = match request.text.as_str() {
            "" => UssdMenu::new("Main Menu")
                .add_option("1", "Account Info")
                .add_option("2", "Services")
                .build_continue(),

            "1" => UssdResponse::continues("Enter your account number:"),

            "2" => UssdMenu::new("Select Service:")
                .add_option("1", "Pay Bill")
                .add_option("2", "Buy Goods")
                .build_continue(),

            _ => UssdResponse::ends("Thank you for using our service."),
        };

        response.to_string()
    }

    #[rocket::launch]
    fn rocket() -> _ {
        rocket::build().mount("/", routes![handle_ussd])
    }
}

// ============================================================================
// Example: Generic handler (framework-agnostic)
// ============================================================================

/// Framework-agnostic USSD handler
/// 
/// This can be used with any web framework - just deserialize the request
/// and serialize the response according to your framework's conventions.
pub async fn generic_ussd_handler(request: UssdRequest) -> String {
    // Your business logic here
    let response = match request.text.as_str() {
        "" => initial_menu(),
        path if path.starts_with("1") => handle_account_menu(&request),
        path if path.starts_with("2") => handle_services_menu(&request),
        _ => UssdResponse::ends("Invalid option. Goodbye!"),
    };

    response.to_string()
}

fn initial_menu() -> UssdResponse {
    UssdMenu::new("Welcome to MyService")
        .add_option("1", "My Account")
        .add_option("2", "Services")
        .add_option("3", "Help")
        .build_continue()
}

fn handle_account_menu(request: &UssdRequest) -> UssdResponse {
    match request.text.as_str() {
        "1" => UssdMenu::new("Account Menu:")
            .add_option("1", "Balance")
            .add_option("2", "Statement")
            .add_option("3", "Profile")
            .build_continue(),
        "1*1" => {
            // Get balance
            UssdResponse::ends("Your balance: KES 5,000.00")
        }
        "1*2" => {
            UssdResponse::ends("Your statement has been sent to your phone via SMS.")
        }
        "1*3" => {
            UssdResponse::ends(format!(
                "Profile Info:\nPhone: {}\nAccount: Active",
                request.phone_number
            ))
        }
        _ => UssdResponse::ends("Invalid option."),
    }
}

fn handle_services_menu(request: &UssdRequest) -> UssdResponse {
    match request.text.as_str() {
        "2" => UssdMenu::new("Services:")
            .add_option("1", "Pay Bill")
            .add_option("2", "Buy Airtime")
            .add_option("3", "Send Money")
            .build_continue(),
        "2*1" => UssdResponse::continues("Enter bill number:"),
        "2*2" => UssdResponse::continues("Enter amount (KES):"),
        "2*3" => UssdResponse::continues("Enter recipient phone:"),
        _ => UssdResponse::ends("Service request received."),
    }
}

// ============================================================================
// Example: Router pattern for complex applications
// ============================================================================

/// Router-based approach for complex USSD applications
pub struct UssdRouter {
    routes: Vec<(Box<dyn Fn(&str) -> bool>, Box<dyn Fn(&UssdRequest) -> UssdResponse>)>,
}

impl UssdRouter {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    /// Add a route with a matcher function and handler
    pub fn add_route<M, H>(mut self, matcher: M, handler: H) -> Self
    where
        M: Fn(&str) -> bool + 'static,
        H: Fn(&UssdRequest) -> UssdResponse + 'static,
    {
        self.routes.push((Box::new(matcher), Box::new(handler)));
        self
    }

    /// Route the request to the appropriate handler
    pub fn route(&self, request: &UssdRequest) -> UssdResponse {
        for (matcher, handler) in &self.routes {
            if matcher(&request.text) {
                return handler(request);
            }
        }
        // Default fallback
        UssdResponse::ends("No route matched. Please try again.")
    }
}

/// Example usage of UssdRouter
pub fn create_router() -> UssdRouter {
    UssdRouter::new()
        // Initial menu
        .add_route(
            |text| text.is_empty(),
            |_| {
                UssdMenu::new("Main Menu")
                    .add_option("1", "Banking")
                    .add_option("2", "Loans")
                    .add_option("3", "Support")
                    .build_continue()
            },
        )
        // Banking submenu
        .add_route(
            |text| text == "1",
            |_| {
                UssdMenu::new("Banking:")
                    .add_option("1", "Check Balance")
                    .add_option("2", "Mini Statement")
                    .build_continue()
            },
        )
        // Check balance
        .add_route(
            |text| text == "1*1",
            |req| UssdResponse::ends(format!("Balance for {}: KES 10,000", req.phone_number)),
        )
        // Mini statement
        .add_route(
            |text| text == "1*2",
            |_| UssdResponse::ends("Last 5 transactions:\n1. +500\n2. -200\n3. +1000"),
        )
        // Loans menu
        .add_route(
            |text| text == "2",
            |_| {
                UssdMenu::new("Loans:")
                    .add_option("1", "Request Loan")
                    .add_option("2", "Loan Balance")
                    .build_continue()
            },
        )
        // Support
        .add_route(
            |text| text == "3",
            |_| UssdResponse::ends("Support: Call 0800-123-456 or email support@example.com"),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router() {
        let router = create_router();
        let req = UssdRequest::new("session1", "*123#", "+254700000000", "", "63902");
        
        let response = router.route(&req);
        assert!(response.is_continuing());
        assert!(response.to_string().contains("Main Menu"));
    }
}
