use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{
    errors::domain_error::DomainError,
    idempotency::models::{ReservationResult, StoredResponse},
};

#[async_trait]
pub trait IdempotencyRepository: Send + Sync {
    async fn reserve(&self, key: Uuid) -> Result<ReservationResult, DomainError>;

    async fn complete(&self, key: Uuid, response: StoredResponse) -> Result<(), DomainError>;
}
