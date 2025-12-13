# Voice Module Documentation

Complete guide for building voice applications with africastalking-rs.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Making Calls](#making-calls)
- [Handling Callbacks](#handling-callbacks)
- [Action Builder](#action-builder)
- [IVR Patterns](#ivr-patterns)
- [Best Practices](#best-practices)

## Overview

The Voice module enables you to:
- Make outbound calls programmatically
- Build IVR (Interactive Voice Response) systems
- Handle inbound calls
- Record calls
- Manage call queues
- Create conference calls

**Note**: Voice callbacks require HTTPS endpoints in production.

## Quick Start

### 1. Make an Outbound Call

```rust
use africastalking::{AfricasTalkingClient, Config};
use africastalking::voice::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new("api_key", "username");
    let client = AfricasTalkingClient::new(config);
    let voice = client.voice();

    let request = MakeCallRequest::new(
        "+254711XXXYYY",  // Your AT number
        vec!["+254722XXXYYY"]  // Recipient
    );

    let response = voice.make_call(request).await?;
    println!("Call queued: {}", response.entries[0].status);
    
    Ok(())
}
```

### 2. Handle Voice Callback

```rust
use axum::{routing::post, Router, Form};
use africastalking::voice::ActionBuilder;

async fn voice_callback(Form(callback): Form<VoiceCallback>) -> String {
    ActionBuilder::new()
        .say("Welcome to our service", None)
        .build()
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/voice", post(voice_callback));
    
    // Serve app...
}
```

## Making Calls

### Simple Call

```rust
let request = MakeCallRequest::new(
    "+254711XXXYYY",
    vec!["+254722XXXYYY"]
);

let response = voice.make_call(request).await?;
```

### Bulk Calls

```rust
let recipients = vec![
    "+254711XXXYYY",
    "+254722XXXYYY",
    "+254733XXXYYY",
];

let request = MakeCallRequest::new("+254700XXXYYY", recipients)
    .with_client_request_id("campaign-123");

let response = voice.make_call(request).await?;

for entry in response.entries {
    match entry.status {
        CallStatus::Queued => println!("✓ {}", entry.phone_number),
        _ => println!("✗ {}: {}", entry.phone_number, entry.status),
    }
}
```

### Check Queue Status

```rust
let request = QueueStatusRequest::new(vec![
    "+254711XXXYYY",
    "+254722XXXYYY",
]);

let response = voice.get_queued_calls(request).await?;
println!("Queued: {}", response.num_queued_calls);
```

### Upload Media

```rust
let request = UploadMediaRequest::new(
    "https://example.com/audio.mp3",
    "+254711XXXYYY"
);

voice.upload_media(request).await?;
```

## Handling Callbacks

### Callback Payload

Africa's Talking sends this to your callback URL:

```rust
#[derive(Deserialize)]
struct VoiceCallback {
    #[serde(rename = "isActive")]
    is_active: String,  // "1" = ongoing, "0" = ended
    
    #[serde(rename = "sessionId")]
    session_id: String,
    
    direction: String,  // "Inbound" or "Outbound"
    
    #[serde(rename = "callerNumber")]
    caller_number: String,
    
    #[serde(rename = "destinationNumber")]
    destination_number: String,
    
    #[serde(rename = "dtmfDigits", default)]
    dtmf_digits: String,  // User input from GetDigits
    
    #[serde(rename = "recordingUrl", default)]
    recording_url: String,  // From Record action
}
```

### Basic Handler (Axum)

```rust
async fn handle_call(Form(callback): Form<VoiceCallback>) -> impl IntoResponse {
    let xml = ActionBuilder::new()
        .say("Thank you for calling", None)
        .build();

    (
        [(axum::http::header::CONTENT_TYPE, "application/xml")],
        xml
    )
}
```

### Return XML Response

All handlers MUST return XML with Content-Type `application/xml`.

## Action Builder

The ActionBuilder creates XML responses that control call flow.

### Say (Text-to-Speech)

```rust
ActionBuilder::new()
    .say("Welcome to our service", None)
    .build()
```

With attributes:

```rust
ActionBuilder::new()
    .say("Hello", Some(SayAttributes {
        voice: Some("female".into()),
        play_beep: Some(true),
    }))
    .build()
```

### Play Audio

```rust
ActionBuilder::new()
    .play("https://example.com/audio.mp3")
    .build()
```

### Get Digits (DTMF Input)

```rust
ActionBuilder::new()
    .get_digits(
        GetDigitsAction::new()
            .say("Press 1 for English, 2 for Swahili", None)
            .num_digits(1)
            .finish_on_key('#')
            .timeout(30)
            .callback_url("https://yourapp.com/voice/input")
    )
    .build()
```

**Important**: User input is sent back to your callback URL in `dtmfDigits` field.

### Dial

```rust
ActionBuilder::new()
    .say("Connecting you to support", None)
    .dial(
        DialAction::new(vec!["+254711SUPPORT"])
            .record(true)
            .max_duration(3600)
    )
    .build()
```

Dial multiple numbers sequentially:

```rust
ActionBuilder::new()
    .dial(
        DialAction::new(vec!["+254711XXX", "+254722XXX"])
            .sequential(true)
    )
    .build()
```

### Record

```rust
ActionBuilder::new()
    .record(
        RecordAction::new()
            .say("Please leave a message after the beep", None)
            .play_beep(true)
            .max_length(60)
            .finish_on_key('#')
            .trim_silence(true)
            .callback_url("https://yourapp.com/voice/recording")
    )
    .build()
```

Recording URL is sent to callback in `recordingUrl` field.

### Enqueue (Call Queue)

```rust
ActionBuilder::new()
    .say("All agents are busy", None)
    .enqueue(Some(EnqueueAttributes {
        hold_music: Some("https://example.com/hold.mp3".into()),
        name: Some("support-queue".into()),
    }))
    .build()
```

### Dequeue (Remove from Queue)

```rust
ActionBuilder::new()
    .say("Connecting to customer", None)
    .dequeue("+254711AGENT", Some("support-queue".into()))
    .build()
```

### Conference

```rust
ActionBuilder::new()
    .say("Joining conference", None)
    .conference()
    .build()
```

### Redirect

```rust
ActionBuilder::new()
    .redirect("https://yourapp.com/voice/another-handler")
    .build()
```

### Reject

```rust
ActionBuilder::new()
    .reject()
    .build()
```

## IVR Patterns

### Simple Menu

```rust
async fn simple_menu(Form(callback): Form<VoiceCallback>) -> impl IntoResponse {
    let xml = if callback.dtmf_digits.is_empty() {
        // Show menu
        ActionBuilder::new()
            .get_digits(
                GetDigitsAction::new()
                    .say("Press 1 for sales, 2 for support", None)
                    .num_digits(1)
            )
            .build()
    } else {
        // Handle choice
        match callback.dtmf_digits.as_str() {
            "1" => ActionBuilder::new()
                .dial(DialAction::new(vec!["+254711SALES"]))
                .build(),
            "2" => ActionBuilder::new()
                .dial(DialAction::new(vec!["+254722SUPPORT"]))
                .build(),
            _ => ActionBuilder::new()
                .say("Invalid option", None)
                .redirect("https://yourapp.com/voice")
                .build(),
        }
    };

    (
        [(axum::http::header::CONTENT_TYPE, "application/xml")],
        xml
    )
}
```

### Stateful IVR

For multi-step flows, maintain session state:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct IvrSession {
    step: Step,
    data: HashMap<String, String>,
}

#[derive(Clone, PartialEq)]
enum Step {
    MainMenu,
    CollectingInput,
    Processing,
}

type Sessions = Arc<RwLock<HashMap<String, IvrSession>>>;

async fn stateful_ivr(
    sessions: Sessions,
    Form(callback): Form<VoiceCallback>,
) -> impl IntoResponse {
    let mut sessions = sessions.write().await;
    let session = sessions.entry(callback.session_id.clone())
        .or_insert_with(|| IvrSession {
            step: Step::MainMenu,
            data: HashMap::new(),
        });

    let xml = match session.step {
        Step::MainMenu => {
            session.step = Step::CollectingInput;
            ActionBuilder::new()
                .get_digits(
                    GetDigitsAction::new()
                        .say("Enter your account number", None)
                        .num_digits(10)
                )
                .build()
        }
        Step::CollectingInput => {
            session.data.insert("account".into(), callback.dtmf_digits);
            session.step = Step::Processing;
            
            let balance = "KES 5,000"; // Fetch from DB
            ActionBuilder::new()
                .say(format!("Your balance is {}", balance), None)
                .build()
        }
        Step::Processing => {
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
```

### Recording Voicemail

```rust
async fn voicemail(Form(callback): Form<VoiceCallback>) -> impl IntoResponse {
    let xml = if callback.recording_url.is_empty() {
        // Start recording
        ActionBuilder::new()
            .say("Please leave your message after the beep", None)
            .record(
                RecordAction::new()
                    .play_beep(true)
                    .max_length(120)
                    .finish_on_key('#')
                    .callback_url("https://yourapp.com/voice/voicemail")
            )
            .build()
    } else {
        // Recording complete
        // Save callback.recording_url to database
        println!("Recording saved: {}", callback.recording_url);
        
        ActionBuilder::new()
            .say("Thank you for your message", None)
            .build()
    };

    (
        [(axum::http::header::CONTENT_TYPE, "application/xml")],
        xml
    )
}
```

## Best Practices

### 1. Always Use HTTPS

Production callback URLs MUST use HTTPS.

### 2. Handle Timeouts

Users may not respond. Always set timeouts:

```rust
.get_digits(
    GetDigitsAction::new()
        .say("Press 1", None)
        .timeout(30)  // 30 second timeout
)
```

### 3. Provide Fallbacks

```rust
.get_digits(
    GetDigitsAction::new()
        .say("Press 1 for English, 2 for Swahili, or stay on the line", None)
        .timeout(10)
)
// If timeout, redirect or dial agent
```

### 4. Keep Prompts Short

Bad: "Thank you for calling Example Company customer support. We value your business. Please select from the following options..."

Good: "Press 1 for sales, 2 for support"

### 5. Validate DTMF Input

```rust
match callback.dtmf_digits.as_str() {
    "1" | "2" | "3" => { /* valid */ }
    _ => {
        ActionBuilder::new()
            .say("Invalid option", None)
            .redirect("https://yourapp.com/voice/menu")
            .build()
    }
}
```

### 6. Session Management

Use session IDs to track state:

```rust
// Store in Redis for production
let key = format!("voice:session:{}", callback.session_id);
redis.set(key, serde_json::to_string(&session_data)?).await?;
```

### 7. Error Handling

```rust
let xml = match process_request(&callback).await {
    Ok(response) => response,
    Err(e) => {
        log::error!("Error processing call: {}", e);
        ActionBuilder::new()
            .say("We're experiencing technical difficulties", None)
            .say("Please try again later", None)
            .build()
    }
};
```

### 8. Logging

Log all callbacks for debugging:

```rust
log::info!(
    "Voice callback - Session: {}, Direction: {}, DTMF: {}",
    callback.session_id,
    callback.direction,
    callback.dtmf_digits
);
```

### 9. Testing

Use Africa's Talking sandbox for testing:

```rust
let config = Config::new("api_key", "username")
    .environment(Environment::Sandbox);
```

### 10. Audio File Requirements

- Format: MP3 or WAV
- Max size: 5MB
- Must be publicly accessible via HTTPS
- Clear audio quality (avoid background noise)

## Common Use Cases

### 1. OTP Verification

```rust
async fn otp_call(Form(callback): Form<VoiceCallback>) -> impl IntoResponse {
    let otp = "1234"; // Generate and store in DB
    
    let xml = ActionBuilder::new()
        .say(format!("Your verification code is {}", otp), None)
        .say(format!("I repeat, your code is {}", otp), None)
        .build();

    (
        [(axum::http::header::CONTENT_TYPE, "application/xml")],
        xml
    )
}
```

### 2. Survey

```rust
async fn survey(
    sessions: Sessions,
    Form(callback): Form<VoiceCallback>
) -> impl IntoResponse {
    // Track questions and responses
    // Store in database when complete
}
```

### 3. Appointment Reminder

```rust
async fn reminder(Form(callback): Form<VoiceCallback>) -> impl IntoResponse {
    let xml = ActionBuilder::new()
        .say("This is a reminder for your appointment tomorrow at 2 PM", None)
        .get_digits(
            GetDigitsAction::new()
                .say("Press 1 to confirm, 2 to cancel", None)
                .num_digits(1)
        )
        .build();

    (
        [(axum::http::header::CONTENT_TYPE, "application/xml")],
        xml
    )
}
```

## WebRTC (Browser Calling)

**Note**: WebRTC requires a separate WASM client library. The Rust server handles callbacks same as regular calls, but caller will be identified as `username.clientname`.

```rust
// Callback from browser call
if callback.caller_number.starts_with("username.") {
    // This is a browser client
    let client_name = callback.caller_number.split('.').last().unwrap();
    println!("Call from browser client: {}", client_name);
}
```

## Troubleshooting

### Calls Not Connecting

- Check callback URL is HTTPS
- Verify URL is publicly accessible
- Ensure callback returns valid XML
- Check AT dashboard for error messages

### DTMF Not Working

- Verify `callback_url` is set in GetDigits
- Check `dtmf_digits` field in callback payload
- Ensure timeout is reasonable (10-30 seconds)

### Audio Not Playing

- Verify audio URL is HTTPS
- Check file size < 5MB
- Test URL in browser
- Ensure format is MP3 or WAV

## Additional Resources

- [AT Voice Docs](https://developers.africastalking.com/docs/voice)
- [Crate Repository](https://github.com/MikeTeddyOmondi/africastalking-rs)
