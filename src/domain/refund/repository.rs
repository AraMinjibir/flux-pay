use async_trait::async_trait;

use crate::domain::{errors::repository_error::RepositoryError, refund::refund::Refund};

#[async_trait]

pub trait RefundRepository: Send + Sync {
    async fn create_refund(&self, refund: &Refund) -> Result<Refund, RepositoryError>;
}
