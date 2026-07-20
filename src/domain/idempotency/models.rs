use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::payment::status::PaymentStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReservationResult {
    Reserved,
    InProgress,
    Completed(StoredResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoredResponse {
    pub payment_id: Uuid,
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
