//! Voice Module Examples

use africastalking::{AfricasTalkingClient, Config, Environment};
use africastalking::voice::*;
use axum::{
    routing::post,
    Router,
    Form,
    response::IntoResponse,
};
use serde::Deserialize;

// ============================================================================
// Example 1: Making Outbound Calls
// ============================================================================

async fn example_make_call() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new("your_api_key", "your_username")
        .environment(Environment::Sandbox);
    
    let client = AfricasTalkingClient::new(config);
    let voice = client.voice();

    // Simple call to one number
    let request = MakeCallRequest::new(
        "+254711XXXYYY", // Your AT phone number
        vec!["+254722XXXYYY"], // Recipient
    );

    let response = voice.make_call(request).await?;
    
    for entry in response.entries {
        match entry.status {
            CallStatus::Queued => {
                println!("✓ Call queued to {}: {}", 
                    entry.phone_number, 
                    entry.session_id.unwrap()
                );
            }
            _ => {
                println!("✗ Call to {} failed: {}", 
                    entry.phone_number, 
                    entry.status
                );
            }
        }
    }

    Ok(())
}

// ============================================================================
// Example 2: Bulk Calling with Tracking
// ============================================================================

async fn example_bulk_call() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new("your_api_key", "your_username");
    let client = AfricasTalkingClient::new(config);
    let voice = client.voice();

    // Call multiple recipients
    let recipients = vec![
        "+254711XXXYYY",
        "+254722XXXYYY",
        "+254733XXXYYY",
    ];

    let request = MakeCallRequest::new("+254700XXXYYY", recipients)
        .with_client_request_id("campaign-12345");

    let response = voice.make_call(request).await?;

    let successful = response.entries.iter()
        .filter(|e| e.status == CallStatus::Queued)
        .count();
    
    println!("Successfully queued {}/{} calls", 
        successful, 
        response.entries.len()
    );

    Ok(())
}

// ============================================================================
// Example 3: Checking Call Queue Status
// ============================================================================

async fn example_queue_status() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new("your_api_key", "your_username");
    let client = AfricasTalkingClient::new(config);
    let voice = client.voice();

    let request = QueueStatusRequest::new(vec![
        "+254711XXXYYY",
        "+254722XXXYYY",
    ]);

    let response = voice.get_queued_calls(request).await?;

    println!("Total queued calls: {}", response.num_queued_calls);
    
    for number in response.phone_numbers {
        println!("{}: {} queued", 
            number.phone_number, 
            number.num_queued_calls
        );
    }

    Ok(())
}

// ============================================================================
// Example 4: Upload Media File
// ============================================================================

async fn example_upload_media() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new("your_api_key", "your_username");
    let client = AfricasTalkingClient::new(config);
    let voice = client.voice();

    let request = UploadMediaRequest::new(
        "https://example.com/audio/greeting.mp3",
        "+254711XXXYYY",
    );

    let response = voice.upload_media(request).await?;
    println!("Upload status: {}", response.status);

    Ok(())
}

// ============================================================================
// Example 5: Simple IVR with ActionBuilder
// ============================================================================

async fn handle_ivr_callback(Form(callback): Form<VoiceCallback>) -> impl IntoResponse {
    // Initial call - show menu
    if callback.dtmf_digits.is_empty() {
        let xml = ActionBuilder::new()
            .say("Welcome to our service", None)
            .get_digits(
                GetDigitsAction::new()
                    .say("Press 1 for sales, 2 for support, or 3 to leave a message", None)
                    .num_digits(1)
                    .finish_on_key('#')
                    .timeout(30)
            )
            .build();
        
        return (
            [(axum::http::header::CONTENT_TYPE, "application/xml")],
            xml
        );
    }

    // Handle user input
    let xml = match callback.dtmf_digits.as_str() {
        "1" => {
            // Transfer to sales
            ActionBuilder::new()
                .say("Connecting you to sales", None)
                .dial(DialAction::new(vec!["+254711SALES"]))
                .build()
        }
        "2" => {
            // Transfer to support
            ActionBuilder::new()
                .say("Connecting you to support", None)
                .dial(DialAction::new(vec!["+254722SUPPORT"]))
                .build()
        }
        "3" => {
            // Record message
            ActionBuilder::new()
                .say("Please leave your message after the beep", None)
                .record(
                    RecordAction::new()
                        .play_beep(true)
                        .max_length(60)
                        .finish_on_key('#')
                        .callback_url("https://yourapp.com/voice/recording")
                )
                .build()
        }
        _ => {
            // Invalid input
            ActionBuilder::new()
                .say("Invalid option selected", None)
                .redirect("https://yourapp.com/voice/callback")
                .build()
        }
    };

    (
        [(axum::http::header::CONTENT_TYPE, "application/xml")],
        xml
    )
}

// ============================================================================
// Example 6: Advanced IVR with State Management
// ============================================================================

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct IvrState {
    step: IvrStep,
    data: HashMap<String, String>,
}

#[derive(Clone, PartialEq)]
enum IvrStep {
    MainMenu,
    CollectingAccountNumber,
    CollectingPin,
    ProcessingRequest,
}

type SessionStore = Arc<RwLock<HashMap<String, IvrState>>>;

async fn handle_advanced_ivr(
    sessions: SessionStore,
    Form(callback): Form<VoiceCallback>,
) -> impl IntoResponse {
    let mut sessions = sessions.write().await;
    let session = sessions.entry(callback.session_id.clone())
        .or_insert_with(|| IvrState {
            step: IvrStep::MainMenu,
            data: HashMap::new(),
        });

    let xml = match session.step {
        IvrStep::MainMenu if callback.dtmf_digits.is_empty() => {
            ActionBuilder::new()
                .say("Welcome to mobile banking", None)
                .get_digits(
                    GetDigitsAction::new()
                        .say("Press 1 to check balance, 2 to transfer money", None)
                        .num_digits(1)
                )
                .build()
        }
        
        IvrStep::MainMenu => {
            match callback.dtmf_digits.as_str() {
                "1" => {
                    session.step = IvrStep::CollectingAccountNumber;
                    ActionBuilder::new()
                        .get_digits(
                            GetDigitsAction::new()
                                .say("Please enter your account number", None)
                                .num_digits(10)
                                .finish_on_key('#')
                        )
                        .build()
                }
                "2" => {
                    session.step = IvrStep::CollectingAccountNumber;
                    session.data.insert("action".into(), "transfer".into());
                    ActionBuilder::new()
                        .get_digits(
                            GetDigitsAction::new()
                                .say("Enter recipient account number", None)
                                .num_digits(10)
                                .finish_on_key('#')
                        )
                        .build()
                }
                _ => {
                    ActionBuilder::new()
                        .say("Invalid option", None)
                        .redirect("https://yourapp.com/voice/callback")
                        .build()
                }
            }
        }

        IvrStep::CollectingAccountNumber => {
            session.data.insert("account".into(), callback.dtmf_digits.clone());
            session.step = IvrStep::CollectingPin;
            
            ActionBuilder::new()
                .get_digits(
                    GetDigitsAction::new()
                        .say("Please enter your 4-digit PIN", None)
                        .num_digits(4)
                        .finish_on_key('#')
                )
                .build()
        }

        IvrStep::CollectingPin => {
            session.data.insert("pin".into(), callback.dtmf_digits.clone());
            session.step = IvrStep::ProcessingRequest;
            
            // Verify PIN and process request
            let account = session.data.get("account").unwrap();
            let balance = "KES 5,000"; // Mock - fetch from DB in production
            
            ActionBuilder::new()
                .say(format!("Your account {} balance is {}", account, balance), None)
                .say("Thank you for using our service", None)
                .build()
        }

        IvrStep::ProcessingRequest => {
            // End call
            ActionBuilder::new()
                .say("Goodbye", None)
                .build()
        }
    };

    (
        [(axum::http::header::CONTENT_TYPE, "application/xml")],
        xml
    )
}

// ============================================================================
// Example 7: Call Queue Management
// ============================================================================

async fn handle_queue_callback(Form(callback): Form<VoiceCallback>) -> impl IntoResponse {
    let xml = if callback.direction == "Inbound" {
        // Add caller to queue
        ActionBuilder::new()
            .say("All our agents are currently busy", None)
            .say("Please hold while we connect you to the next available agent", None)
            .enqueue(Some(EnqueueAttributes {
                hold_music: Some("https://example.com/hold-music.mp3".into()),
                name: Some("support-queue".into()),
            }))
            .build()
    } else {
        // Agent answering - dequeue caller
        ActionBuilder::new()
            .say("Connecting to customer", None)
            .dequeue(callback.caller_number, Some("support-queue".into()))
            .build()
    };

    (
        [(axum::http::header::CONTENT_TYPE, "application/xml")],
        xml
    )
}

// ============================================================================
// Example 8: Conference Call
// ============================================================================

async fn handle_conference_callback(Form(callback): Form<VoiceCallback>) -> impl IntoResponse {
    let xml = ActionBuilder::new()
        .say("Welcome to the conference call", None)
        .conference()
        .build();

    (
        [(axum::http::header::CONTENT_TYPE, "application/xml")],
        xml
    )
}

// ============================================================================
// Example 9: Complete Axum Server
// ============================================================================

#[tokio::main]
async fn main() {
    let sessions: SessionStore = Arc::new(RwLock::new(HashMap::new()));

    let app = Router::new()
        .route("/voice/callback", post(handle_ivr_callback))
        .route("/voice/advanced", post({
            let sessions = Arc::clone(&sessions);
            move |callback| handle_advanced_ivr(Arc::clone(&sessions), callback)
        }))
        .route("/voice/queue", post(handle_queue_callback))
        .route("/voice/conference", post(handle_conference_callback));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5959")
        .await
        .unwrap();
    
    println!("Voice server running on http://localhost:5959");
    axum::serve(listener, app).await.unwrap();
}
