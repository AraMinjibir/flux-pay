use serde::Deserialize;

use crate::domain::payment::{payment::PaymentInitializationResult, status::PaymentStatus};

#[derive(Deserialize)]
pub struct PaystackInitializeResponse {
    data: PaystackData,
}

#[derive(Deserialize)]
pub struct PaystackData {
    authorization_url: String,
    reference: String,
}

impl From<PaystackInitializeResponse> for PaymentInitializationResult {
    fn from(response: PaystackInitializeResponse) -> Self {
        Self {
            provider_reference: response.data.reference,
            authorization_url: Some(response.data.authorization_url),
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
