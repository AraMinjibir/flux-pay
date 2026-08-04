use serde::Deserialize;

use crate::domain::payment::{payment::PaymentInitializationResult, status::PaymentStatus};

#[derive(Deserialize)]
pub struct StripePaymentIntentResponse {
    id: String,
    client_secret: String,
    status: PaymentStatus,
}

impl From<StripePaymentIntentResponse> for PaymentInitializationResult {
    fn from(response: StripePaymentIntentResponse) -> Self {
        Self {
            provider_reference: response.id,
            client_secret: Some(response.client_secret),
            status: response.status,
            authorization_url: None,
            amount: None,
            created_at: None,
            id: None,
            merchant_id: None,
            reference: None,
            description: None,
        }
    }
}
