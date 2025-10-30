use africastalking::sms::SendSmsRequest;
use africastalking::{AfricasTalkingClient, AfricasTalkingError, Config, Environment, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env (ignoring “file not found” errors)
    dotenvy::dotenv().ok();

    // Now you can read env vars:
    let api_key = std::env::var("AFRICASTALKING_API_KEY").map_err(|_| {
        AfricasTalkingError::Config("AFRICASTALKING_API_KEY config not set".to_string())
    })?;
    let username = std::env::var("AFRICASTALKING_USERNAME").map_err(|_| {
        AfricasTalkingError::Config("AFRICASTALKING_USERNAME config not set".to_string())
    })?;

    let config = Config::new(api_key, username.clone()).environment(Environment::Sandbox);

    let client = AfricasTalkingClient::new(config).unwrap();

    let sms = client.sms();

    // send sms
    let request = SendSmsRequest {
        to: "254717135176".to_string(), // vec!["254717135176".to_string()].join(","),
        message: "Hello, AfricasTalking!".to_string(),
        from: None,
        bulk_sms_mode: None,
        enqueue: None,
        keyword: None,
        link_id: None,
        retry_duration_in_hours: None,
    };

    let send_sms_response = sms.send(request).await?;
    println!("{send_sms_response:#?}");

    Ok(())
}
