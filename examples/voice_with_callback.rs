use africastalking::voice::{MakeCallRequest, ActionBuilder, VoiceCallback};
use africastalking::{AfricasTalkingClient, AfricasTalkingError, Config, Environment, Result};
use axum::{Router, routing::post, Form, response::IntoResponse};

// =============================================================================
// STEP 1: Start the callback server (this must be running FIRST)
// =============================================================================
// This server receives requests from AfricasTalking when calls connect.
// You must configure this URL in AT Dashboard: Voice > Phone Numbers > Callback URL
// Example: https://yourserver.com/voice/callback
// 
// NOTE: Must use HTTPS in production, can use ngrok for testing locally
// =============================================================================

async fn handle_voice_callback(Form(callback): Form<VoiceCallback>) -> impl IntoResponse {
    println!("📞 Call received from AT: {:#?}", callback);
    
    // When AT connects the call, tell it what to do via XML
    let xml = ActionBuilder::new()
        .say("Hello! This is an automated call from AfricasTalking Voice API", None)
        .say("Thank you for answering. Goodbye!", None)
        .build();
    
    // Must return XML with correct content type
    (
        [(axum::http::header::CONTENT_TYPE, "application/xml")],
        xml
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let api_key = std::env::var("AFRICASTALKING_API_KEY").map_err(|_| {
        AfricasTalkingError::Config("AFRICASTALKING_API_KEY not set".to_string())
    })?;
    let username = std::env::var("AFRICASTALKING_USERNAME").map_err(|_| {
        AfricasTalkingError::Config("AFRICASTALKING_USERNAME not set".to_string())
    })?;

    let config = Config::new(api_key, username.clone()).environment(Environment::Sandbox);
    let client = AfricasTalkingClient::new(config).unwrap();

    // =============================================================================
    // STEP 2: Start the Axum server in a background task
    // =============================================================================
    // This runs your callback handler that AT will POST to
    // =============================================================================
    
    tokio::spawn(async {
        let app = Router::new()
            .route("/voice/callback", post(handle_voice_callback));
        
        let listener = tokio::net::TcpListener::bind("0.0.0.0:5959")
            .await
            .unwrap();
        
        println!("🎧 Voice callback server running on http://localhost:5959");
        println!("📝 Set callback URL in AT Dashboard to: https://example.com/voice/callback");
        
        axum::serve(listener, app).await.unwrap();
    });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // =============================================================================
    // STEP 3: Make the outbound call
    // =============================================================================
    // This initiates the call. The flow is:
    // 1. This code sends request to AT API
    // 2. AT queues the call and responds immediately with session ID
    // 3. AT starts dialing the recipient number
    // 4. When recipient answers, AT makes POST to YOUR callback URL (Step 1)
    // 5. Your server responds with XML telling AT what to say/do
    // 6. AT executes those actions (plays voice, etc.)
    // 7. Call ends, AT sends final notification with duration/cost
    // =============================================================================

    let voice = client.voice();

    let request = MakeCallRequest::new(
        "+254711XXXYYY",      // Your AT phone number (from)
        vec!["+254717135176"] // Recipient number (to)
    )
    .with_client_request_id("demo-call-001");

    println!("📤 Initiating call...");
    let response = voice.make_call(request).await?;
    
    println!("✅ Call Response: {:#?}", response);
    
    for entry in &response.entries {
        match entry.status {
            africastalking::voice::CallStatus::Queued => {
                println!("📞 Call queued to: {}", entry.phone_number);
                println!("🆔 Session ID: {}", entry.session_id.as_ref().unwrap());
                println!("⏳ Waiting for AT to call your callback URL...");
            }
            _ => {
                println!("❌ Call failed: {} - {}", entry.phone_number, entry.status);
            }
        }
    }

    // Keep the program running so callback server stays alive
    println!("\n⏳ Server running. Press Ctrl+C to exit.");
    tokio::signal::ctrl_c().await.unwrap();

    Ok(())
}
