use africastalking::sms::SendSmsRequest;
use africastalking::{AfricasTalkingClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = AfricasTalkingClient::new_content_type_json(None).unwrap();

    let sms = client.sms();

    let message = "Hello, this is a bulk SMS test using modern method!".to_string();
    let phone_numbers = vec!["254791715651".to_string()];

    let send_sms_response = sms.send_bulk_mordern(message, phone_numbers).await?;
    println!("{send_sms_response:#?}");

    Ok(())
}
