use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{
    errors::domain_error::DomainError,
    idempotency::{
        models::{ReservationResult, StoredResponse},
        repository::IdempotencyRepository,
    },
    services::idempotency_service::IdempotencyService,
};

pub struct IdempotencyServiceImpl {
    repository: Arc<dyn IdempotencyRepository>,
}

impl IdempotencyServiceImpl {
    pub fn new(repository: Arc<dyn IdempotencyRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl IdempotencyService for IdempotencyServiceImpl {
    async fn reserve(&self, key: Uuid) -> Result<ReservationResult, DomainError> {
        self.repository.reserve(key).await
    }

    async fn complete(&self, key: Uuid, response: StoredResponse) -> Result<(), DomainError> {
        self.repository.complete(key, response).await
    }
}
