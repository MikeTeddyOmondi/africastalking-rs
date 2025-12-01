use africastalking::data::{
    DataUnits, DataValidity, MobileDataRequest, Recipient, RecipientMetadata,
};
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

    let config = Config::new(api_key, username.clone()).environment(Environment::Production);

    let client = AfricasTalkingClient::new(config).unwrap();

    let data = client.data();

    let receipient_metadata = RecipientMetadata {
        transaction_id: "txn_1234d2232dfsdrwrewr56".to_string(),
    };

    let recipient = Recipient {
        phone_number: "+254791725651".to_string(),
        quantity: 50,
        unit: DataUnits::MB,
        validity: DataValidity::Day,
        is_promo_bundle: false,
        metadata: receipient_metadata,
    };
    // send sms
    let request = MobileDataRequest {
        user_name: "rust-sdk".to_string(),
        product_name: "datatest".to_string(),
        recipients: vec![recipient],
    };
    let send_data_response = data.send(request).await?;
    println!("{send_data_response:#?}");
    Ok(())
}
