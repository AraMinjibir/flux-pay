use serde::Deserialize;

use crate::domain::payment::{payment::PaymentInitializationResult, status::PaymentStatus};

#[derive(Deserialize)]
pub struct PaystackInitializeResponse {
    status: PaymentStatus,
    message: String,
    data: PaystackData,
}

#[derive(Deserialize)]
pub struct PaystackData {
    authorization_url: String,
    access_code: String,
    reference: String,
}

impl From<PaystackInitializeResponse> for PaymentInitializationResult {
    fn from(response: PaystackInitializeResponse) -> Self {
        Self {
            provider_reference: response.data.reference,
            authorization_url: Some(response.data.authorization_url),
            status: response.status,
            client_secret: None,
            amount: None,
            created_at: None,
            id: None,
            merchant_id: None,
            reference: None,
            description: None
        }
    }
}
