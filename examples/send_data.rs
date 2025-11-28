use africastalking::data::{MobileDataRequest, Recipient};
use africastalking::{AfricasTalkingClient, AfricasTalkingError, Config, Environment, Result};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Metadata {
    pub transaction_id: String,
}

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

    let config = Config::new(api_key, username.clone()).environment(Environment::Production);

    let client = AfricasTalkingClient::new(config).unwrap();

    let data = client.data();

    let receipient_metadata = Metadata {
        transaction_id: "txn_1234d2232dfsdrwrewr56".to_string(),
    };

    let recipient = Recipient {
        phone_number: "+254791725651".to_string(),
        quantity: 50,
        unit: "MB".to_string(),
        validity: "Day".to_string(),
        is_promo_bundle: false,
        metadata: receipient_metadata,
    };
    // send sms
    let request = MobileDataRequest {
        user_name: "rust-sdk".to_string(),
        product_name: "datatest".to_string(),
        recipients: vec![recipient],
    };
    let send_data_response = data.request(request).await?;
    println!("{send_data_response:#?}");
    Ok(())
}
