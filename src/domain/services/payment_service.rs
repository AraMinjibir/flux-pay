use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::payment::{method::PaymentMethod, status::PaymentStatus};
use crate::domain::{
    errors::domain_error::DomainError,
    payment::{
        payment::{CreatePaymentCommand, Payment, PaymentInitializationResult},
        provider::PaymentProvider,
    },
};

#[async_trait]
pub trait PaymentService: Send + Sync {
    async fn generate_payment(
        &self,
        command: CreatePaymentCommand,
    ) -> Result<PaymentInitializationResult, DomainError>;

    async fn find_payment_by_id(&self, id: Uuid) -> Result<Payment, DomainError>;
    async fn find_payment_by_reference(&self, reference: &str) -> Result<Payment, DomainError>;
    async fn find_payment_by_status(
        &self,
        status: &PaymentStatus,
    ) -> Result<Vec<Payment>, DomainError>;

    async fn find_by_provider(
        &self,
        provider: &PaymentProvider,
    ) -> Result<Vec<Payment>, DomainError>;

    async fn find_payment_by_method(
        &self,
        method: &PaymentMethod,
    ) -> Result<Vec<Payment>, DomainError>;

    async fn find_payment_by_merchant(
        &self,
        merchant_id: Uuid,
    ) -> Result<Vec<Payment>, DomainError>;

    async fn find_all_payments(&self) -> Result<Vec<Payment>, DomainError>;
    async fn delete(&self, id: Uuid) -> Result<u64, DomainError>;
}
