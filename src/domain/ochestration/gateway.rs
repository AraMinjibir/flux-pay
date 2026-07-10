use async_trait::async_trait;

use crate::domain::{
    errors::domain_error::DomainError,
    ochestration::capability::ProviderCapabilities,
    payment::{
        payment::{PaymentInitializationRequest, PaymentInitializationResult},
        provider::PaymentProvider,
    },
};

#[async_trait]
pub trait PaymentGateway: Send + Sync {
    fn provider(&self) -> PaymentProvider;

    fn capabilities(&self) -> &ProviderCapabilities;

    async fn initialize_payment(
        &self,
        payment: &PaymentInitializationRequest,
    ) -> Result<PaymentInitializationResult, DomainError>;
}
