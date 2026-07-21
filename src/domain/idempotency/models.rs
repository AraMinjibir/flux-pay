use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::payment::{payment::PaymentInitializationResult, status::PaymentStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReservationResult {
    Reserved,
    InProgress,
    Completed(StoredResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoredResponse {
    pub payment_id: Uuid,
    pub provider_reference: String,
    pub authorization_url: Option<String>,
    pub client_secret: Option<String>,
    pub status: PaymentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyRecord {
    pub status: IdempotencyStatus,
    pub response: Option<StoredResponse>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdempotencyStatus {
    Processing,
    Completed,
}

impl StoredResponse {
    pub fn from_payment(
        payment_id: Uuid,
        response: &PaymentInitializationResult,
    ) -> Self {
        Self {
            payment_id,
            provider_reference: response.provider_reference.clone(),
            authorization_url: response.authorization_url.clone(),
            client_secret: response.client_secret.clone(),
            status: response.status.clone(),
        }
    }
}