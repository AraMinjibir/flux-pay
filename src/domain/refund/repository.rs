use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{errors::repository_error::RepositoryError, refund::refund::Refund};

#[async_trait]

pub trait RefundRepository: Send + Sync {
    async fn create_refund(&self, refund: &Refund) -> Result<Refund, RepositoryError>;
    async fn update_refund(&self, refund: &Refund) -> Result<Refund, RepositoryError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Refund>, RepositoryError>;
}
