use africastalking::data::{
    DataUnits, DataValidity, MobileDataRequest, Recipient, RecipientMetadata,
};
use africastalking::{AfricasTalkingClient, AfricasTalkingError, Config, Environment, Result};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // to be used as transation id metadata
    let random_string = Uuid::new_v4();

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
        transaction_id: random_string.to_string(),
    };

    let recipient = Recipient {
        phone_number: "+254700000000".to_string(),
        quantity: 50,
        unit: DataUnits::MB,
        validity: DataValidity::Day,
        is_promo_bundle: false,
        metadata: receipient_metadata.clone(),
    };

    let request = MobileDataRequest {
        user_name: "rust-sdk".to_string(),
        product_name: "datatest".to_string(),
        recipients: vec![recipient],
    };

    let send_data_response = data.send(request).await?;
    println!("Send data response: {send_data_response:#?}");

    // query/find transaction.
    let transaction = client
        .data()
        .find_transaction("ATPid_b9379b671fee8ccf24b2c74f94da0ceb".to_string())
        .await?;

    println!("Queried transaction : {transaction:#?}");

    let wallet_balance = client.data().query_wallet_balance().await?;
    println!("Wallet balance : {wallet_balance:#?}");
    Ok(())
}
