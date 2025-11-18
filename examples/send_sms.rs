use africastalking::sms::SendSmsRequest;
use africastalking::{AfricasTalkingClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = AfricasTalkingClient::new().unwrap();

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
