use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{
    errors::repository_error::RepositoryError,
    payment::{
        method::PaymentMethod, payment::Payment, provider::PaymentProvider, status::PaymentStatus,
    },
};

#[async_trait]
pub trait PaymentRepository {
    async fn save(&self, payment: &Payment) -> Result<(), RepositoryError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Payment>, RepositoryError>;

    async fn find_by_reference(&self, reference: &str) -> Result<Option<Payment>, RepositoryError>;

    async fn find_by_status(&self, status: &PaymentStatus)
    -> Result<Vec<Payment>, RepositoryError>;

    async fn find_by_provider(
        &self,
        provider: PaymentProvider,
    ) -> Result<Vec<Payment>, RepositoryError>;

    async fn find_by_method(&self, method: PaymentMethod) -> Result<Vec<Payment>, RepositoryError>;

    async fn find_by_merchant(&self, merchant_id: Uuid) -> Result<Vec<Payment>, RepositoryError>;

    async fn find_all(&self) -> Result<Vec<Payment>, RepositoryError>;

    async fn update(&self, payment: &Payment) -> Result<(), RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<u64, RepositoryError>;
}
