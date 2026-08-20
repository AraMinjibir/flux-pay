use serde::Deserialize;

use crate::domain::payment::{payment::PaymentInitializationResult, status::PaymentStatus};

#[derive(Deserialize)]

pub struct ZainpayInitializationResponse {
    pub code: String,
    pub data: String,
    pub decription: String,
}

impl From<ZainpayInitializationResponse> for PaymentInitializationResult {
    fn from(response: ZainpayInitializationResponse) -> Self {
        Self {
            provider_reference: response.code,
            authorization_url: Some(response.data),
            status: PaymentStatus::Processing,
            selected_provider: None,
            client_secret: None,
            amount: None,
            created_at: None,
            id: None,
            merchant_id: None,
            reference: None,
            description: None,
        }
    }
}
