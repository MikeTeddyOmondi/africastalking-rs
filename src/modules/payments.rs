/// Payments module implementation

use crate::{client::AfricasTalkingClient, error::Result, AfricasTalkingError, Currency};
use serde::{Deserialize, Serialize};

/// Payments module for handling mobile and bank payments
#[derive(Debug, Clone)]
pub struct PaymentsModule {
    client: AfricasTalkingClient,
}

impl PaymentsModule {
    pub(crate) fn new(client: AfricasTalkingClient) -> Self {
        Self { client }
    }
    
    /// Mobile checkout (B2C)
    pub async fn mobile_checkout(&self, request: MobileCheckoutRequest) -> Result<MobileCheckoutResponse> {
        self.client.post("/version1/payments/mobile/checkout/request", &request).await
    }
    
    /// Mobile B2B payment
    pub async fn mobile_b2b(&self, request: MobileB2BRequest) -> Result<MobileB2BResponse> {
        self.client.post("/version1/payments/mobile/b2b/request", &request).await
    }
    
    /// Bank checkout
    pub async fn bank_checkout(&self, request: BankCheckoutRequest) -> Result<BankCheckoutResponse> {
        self.client.post("/version1/payments/bank/checkout/request", &request).await
    }
    
    /// Bank transfer
    pub async fn bank_transfer(&self, request: BankTransferRequest) -> Result<BankTransferResponse> {
        self.client.post("/version1/payments/bank/transfer", &request).await
    }
    
    /// Card checkout
    pub async fn card_checkout(&self, request: CardCheckoutRequest) -> Result<CardCheckoutResponse> {
        self.client.post("/version1/payments/card/checkout/request", &request).await
    }
    
    /// Validate card checkout
    pub async fn validate_card_checkout(&self, request: ValidateCardCheckoutRequest) -> Result<ValidateCardCheckoutResponse> {
        self.client.post("/version1/payments/card/checkout/validate", &request).await
    }
    
    /// Find transaction
    pub async fn find_transaction(&self, transaction_id: &str) -> Result<FindTransactionResponse> {
        let endpoint = format!("/version1/payments/find?transactionId={}", transaction_id);
        self.client.get(&endpoint).await
    }
    
    /// Get wallet balance
    pub async fn get_wallet_balance(&self) -> Result<WalletBalanceResponse> {
        self.client.get("/version1/payments/balance").await
    }

    /// Get wallet transactions
    pub async fn get_wallet_transactions(&self, request: WalletTransactionsRequest) -> Result<WalletTransactionsResponse> {
        let mut query_params = Vec::new();
        
        if let Some(page) = request.page {
            query_params.push(("page", page.to_string()));
        }
        if let Some(per_page) = request.per_page {
            query_params.push(("perPage", per_page.to_string()));
        }
        if let Some(start_date) = &request.start_date {
            query_params.push(("startDate", start_date.clone()));
        }
        if let Some(end_date) = &request.end_date {
            query_params.push(("endDate", end_date.clone()));
        }

        let qs = serde_urlencoded::to_string(&query_params)
            .map_err(AfricasTalkingError::Serialization)?;
        let endpoint = format!("/version1/payments/transactions?{}", qs);
        self.client.get(&endpoint).await
    }
}

// --- Request and Response types for Payments Module ---

#[derive(Debug, Serialize)]
pub struct MobileCheckoutRequest {
    pub product_name: String,
    pub provider: String,
    pub currency_code: String,
    pub amount: String,
    pub metadata: Option<HashMap<String, String>>,
    pub phone_number: String,
    pub country_code: String,
}

#[derive(Debug, Deserialize)]
pub struct MobileCheckoutResponse {
    pub provider: String,
    pub status: String,
    pub request_id: String,
    pub request_time: String,
    pub receipt: Option<String>,
    pub cost: Option<String>,
}

// (Other request/response structs like MobileB2BRequest, BankCheckoutRequest, etc. would follow here, as per the API specification.)

#[derive(Debug, Serialize)]
pub struct WalletTransactionsRequest {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WalletTransactionsResponse {
    pub transactions: Vec<WalletTransaction>,
    pub total: u32,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Deserialize)]
pub struct WalletTransaction {
    pub transaction_id: String,
    pub amount: String,
    pub status: String,
    pub date: String,
    pub currency: String,
}
