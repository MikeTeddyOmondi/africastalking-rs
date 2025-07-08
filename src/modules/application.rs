// src/modules/application.rs
//! Application module implementation

use crate::{client::AfricasTalkingClient, error::Result};
use serde::{Deserialize, Serialize};

/// Application module for getting app data
#[derive(Debug, Clone)]
pub struct ApplicationModule {
    client: AfricasTalkingClient,
}

impl ApplicationModule {
    pub(crate) fn new(client: AfricasTalkingClient) -> Self {
        Self { client }
    }
    
    /// Get application data
    pub async fn get_data(&self) -> Result<ApplicationDataResponse> {
        self.client.get("/version1/user").await
    }
}

#[derive(Debug, Deserialize)]
pub struct ApplicationDataResponse {
    #[serde(rename = "UserData")]
    pub user_data: UserData,
}

#[derive(Debug, Deserialize)]
pub struct UserData {
    pub balance: String,
}
