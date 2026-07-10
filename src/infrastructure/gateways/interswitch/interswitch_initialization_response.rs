use serde::Deserialize;

use crate::domain::payment::{payment::PaymentInitializationResult, status::PaymentStatus};

#[derive(Deserialize)]
pub struct InterswitchInitializeResponse {
    authorization_url: String,
    status: PaymentStatus,
    reference: String,
}

impl From<InterswitchInitializeResponse> for PaymentInitializationResult {
    fn from(response: InterswitchInitializeResponse) -> Self {
        Self {
            authorization_url: Some(response.authorization_url),
            status: response.status,
            provider_reference: response.reference,
            client_secret: None,
        }
    }
}
