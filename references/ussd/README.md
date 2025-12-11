# Africa's Talking USSD Module for Rust

A complete, production-ready USSD module implementation for the `africastalking-rs` crate.

## 📦 What's Included

### Core Module (`ussd.rs`)
- **`UssdRequest`** - Type-safe USSD request handling with helper methods
- **`UssdResponse`** - Builder for CON/END responses  
- **`UssdMenu`** - Ergonomic menu builder with fluent API
- **`UssdNotification`** - End-of-session notification handling
- **`NetworkCode`** - Complete telco network code enumeration
- **`UssdSessionStatus`** - Session completion status types

### Examples

#### 1. **Basic Axum Example** (`examples/axum_example.rs`)
Simple USSD application demonstrating:
- Route-based navigation
- Menu building
- Terminal vs continuing responses
- Notification handling

#### 2. **Advanced Stateful Example** (`examples/stateful_example.rs`)
Production-grade implementation with:
- Session state management
- Multi-step data collection
- Confirmation flows
- Redis/in-memory session storage

#### 3. **Framework Examples** (`examples/framework_examples.rs`)
Demonstrates usage with:
- **Axum** - Modern async framework
- **Actix-web** - High-performance framework
- **Rocket** - Type-safe framework
- **Generic handler** - Framework-agnostic approach
- **Router pattern** - Scalable routing for complex apps

### Documentation
- **Comprehensive guide** (`USSD_DOCUMENTATION.md`)
- **Inline examples** in module code
- **Full test coverage** (`tests/ussd_tests.rs`)

## 🚀 Quick Start

```rust
use axum::{routing::post, Form, Router};
use africastalking_rs::ussd::{UssdRequest, UssdResponse, UssdMenu};

async fn handle_ussd(Form(req): Form<UssdRequest>) -> String {
    let response = match req.text.as_str() {
        "" => UssdMenu::new("Welcome! Choose:")
            .add_option("1", "Check Balance")
            .add_option("2", "Send Money")
            .build_continue(),
        
        "1" => UssdResponse::ends("Balance: KES 1,000"),
        "2" => UssdResponse::continues("Enter recipient phone:"),
        
        _ => UssdResponse::ends("Invalid option"),
    };
    
    response.to_string()
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/ussd", post(handle_ussd));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4949").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

## 🎯 Key Features

### Type Safety
```rust
// Request helpers prevent errors
if request.is_initial() { /* show main menu */ }
let depth = request.depth();
let path = request.navigation_path();
```

### Ergonomic API
```rust
// Fluent menu builder
UssdMenu::new("Services")
    .add_option("1", "Banking")
    .add_option("2", "Airtime")
    .build_continue()
```

### Network Code Support
```rust
let network = NetworkCode::from_code("63902");
println!("{}", network.name());     // "Safaricom Kenya"
println!("{}", network.country());  // "Kenya"
```

### Session Management
```rust
// Track state across interactions
let session = get_session(&request.session_id).await;
match session.state {
    Step::CollectingName => { /* ... */ }
    Step::CollectingAmount => { /* ... */ }
    Step::Confirming => { /* ... */ }
}
```

## 📋 Design Principles

1. **Framework Agnostic** - Works with any Rust web framework
2. **Zero Runtime** - No background processes or daemons
3. **Type Safe** - Leverages Rust's type system
4. **Ergonomic** - Simple, intuitive API
5. **Production Ready** - Handles edge cases, includes tests

## 🧪 Testing

Comprehensive test suite covering:
- ✅ Request parsing and navigation
- ✅ Response building
- ✅ Menu construction
- ✅ Network code mapping
- ✅ Serialization/deserialization
- ✅ Integration scenarios
- ✅ Edge cases

Run tests:
```bash
cargo test
```

## 📚 API Overview

### `UssdRequest`
```rust
pub struct UssdRequest {
    pub session_id: String,
    pub service_code: String,
    pub phone_number: String,
    pub text: String,           // "1*2*3" format
    pub network_code: String,
}

// Methods
is_initial() -> bool
depth() -> usize
current_input() -> Option<&str>
navigation_path() -> Vec<&str>
matches_path(&str) -> bool
starts_with_path(&str) -> bool
```

### `UssdResponse`
```rust
// Creates a continuing response (CON)
UssdResponse::continues("Enter PIN:");

// Creates a terminal response (END)
UssdResponse::ends("Thank you!");

// Methods
is_continuing() -> bool
is_ending() -> bool
message() -> &str
to_string() -> String  // "CON ..." or "END ..."
```

### `UssdMenu`
```rust
UssdMenu::new("Title")
    .add_option("1", "Option 1")
    .add_option("2", "Option 2")
    .add_options(vec![("3", "Option 3")])
    .build_continue()  // or .build_end()
```

### `UssdNotification`
```rust
pub struct UssdNotification {
    pub date: String,
    pub session_id: String,
    pub status: UssdSessionStatus,  // Success | Incomplete | Failed
    pub cost: String,
    pub duration_in_millis: String,
    pub hops_count: i32,
    pub input: String,
    pub last_app_response: String,
    pub error_message: Option<String>,
    // ... more fields
}
```

## 🌍 Supported Networks

Covers all Africa's Talking networks across:
- 🇬🇭 Ghana (AirtelTigo, Vodafone, MTN)
- 🇳🇬 Nigeria (Airtel, MTN, Glo, Etisalat)
- 🇷🇼 Rwanda (MTN, Tigo, Airtel)
- 🇪🇹 Ethiopia (EthioTelecom)
- 🇰🇪 Kenya (Safaricom, Airtel, Orange, Equitel)
- 🇹🇿 Tanzania (Tigo, Vodacom, Airtel)
- 🇺🇬 Uganda (Airtel, MTN, Africell)
- 🇿🇲 Zambia (Airtel, MTN)
- 🇲🇼 Malawi (TNM, Airtel)
- 🇿🇦 South Africa (Vodacom, Telkom, CellC, MTN)

## 🎨 Architecture Patterns

### Simple Pattern (Small Apps)
```rust
match request.text.as_str() {
    "" => show_main_menu(),
    "1" => handle_option_1(),
    "1*1" => handle_nested_option(),
    _ => invalid_option(),
}
```

### State Pattern (Medium Apps)
```rust
struct Session {
    state: State,
    data: HashMap<String, String>,
}

match session.state {
    State::Initial => { /* ... */ }
    State::Collecting => { /* ... */ }
    State::Confirming => { /* ... */ }
}
```

### Router Pattern (Large Apps)
```rust
UssdRouter::new()
    .add_route(|t| t.is_empty(), initial_handler)
    .add_route(|t| t.starts_with("1"), banking_handler)
    .add_route(|t| t.starts_with("2"), services_handler)
    .route(&request)
```

## 🔧 Integration with africastalking-rs

This module follows the same patterns as the SMS module:

```rust
// SMS Module (existing)
pub struct SmsModule {
    client: AfricasTalkingClient,
}

impl SmsModule {
    pub async fn send(&self, request: SendSmsRequest) -> Result<SendSmsResponse>;
}

// USSD Module (this implementation)
// Note: USSD is primarily a webhook-based API, not a client API
// The module provides request/response types for handling webhooks
```

## 📦 File Structure

```
africastalking-rs/
├── src/
│   └── modules/
│       └── ussd.rs              # Main module (replace existing file)
├── examples/
│   ├── axum_example.rs          # Basic Axum usage
│   ├── stateful_example.rs      # Advanced with state
│   └── framework_examples.rs    # Multiple frameworks
├── tests/
│   └── ussd_tests.rs           # Comprehensive tests
└── docs/
    └── USSD_DOCUMENTATION.md    # Full guide
```

## 🚢 Deployment Checklist

- [ ] Set up HTTPS endpoint
- [ ] Configure webhook URLs in Africa's Talking dashboard
- [ ] Implement session storage (Redis recommended)
- [ ] Add logging and monitoring
- [ ] Test with actual USSD codes
- [ ] Monitor notification endpoint
- [ ] Set up error alerting

## 💡 Best Practices

1. **Keep menus simple** - Max 6-8 options
2. **Validate all input** - Phone numbers, amounts, etc.
3. **Handle timeouts** - Sessions expire after ~30s
4. **Clear error messages** - Tell users what went wrong
5. **Test thoroughly** - Use sandbox environment first
6. **Monitor sessions** - Track completion rates

## 🤝 Contributing

This module is ready to be integrated into the africastalking-rs repository.

To use:
1. Replace `src/modules/ussd.rs` with the new implementation
2. Add examples to the `examples/` directory
3. Add tests to the `tests/` directory
4. Update main crate documentation

## 📄 License

Same as africastalking-rs main crate.

## 🙏 Credits

Built following Africa's Talking USSD API documentation and inspired by the existing SMS module architecture.

---

**Ready for production use! 🚀**
