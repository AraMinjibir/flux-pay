use async_trait::async_trait;

use crate::domain::{
    errors::domain_error::DomainError,
    orchestration::capability::ProviderCapabilities,
    payment::{
        payment::{PaymentInitializationRequest, PaymentInitializationResult},
        provider::PaymentProvider,
    },
    refund::refund::Refund,
};

#[async_trait]
pub trait PaymentGateway: Send + Sync {
    fn provider(&self) -> PaymentProvider;

    fn capabilities(&self) -> &ProviderCapabilities;

    async fn initialize_payment(
        &self,
        payment: &PaymentInitializationRequest,
    ) -> Result<PaymentInitializationResult, DomainError>;
    async fn refund_payment(&self, refund: &Refund) -> Result<Refund, DomainError>;
}
