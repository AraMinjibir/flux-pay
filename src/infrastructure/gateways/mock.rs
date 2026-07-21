use async_trait::async_trait;

use crate::{
    domain::{
        errors::domain_error::DomainError,
        orchestration::{capability::ProviderCapabilities, gateway::PaymentGateway},
        payment::{
            method::PaymentMethod,
            payment::{PaymentInitializationRequest, PaymentInitializationResult},
            provider::PaymentProvider,
            status::PaymentStatus,
        },
        shared::currency::Currency,
    },
    infrastructure::config::app_config::MockConfig,
};

pub struct MockPaymentGateway {
    capabilities: ProviderCapabilities,
    config: MockConfig,
}

impl MockPaymentGateway {
    pub fn new(config: MockConfig) -> Self {
        Self {
            capabilities: ProviderCapabilities {
                currencies: vec![
                    Currency::NGN,
                    Currency::USD,
                    Currency::EUR,
                    Currency::GBP,
                    Currency::AED,
                    Currency::SAR,
                ],
                payment_methods: vec![PaymentMethod::Card, PaymentMethod::BankTransfer],
            },
            config,
        }
    }
}

#[async_trait]
impl PaymentGateway for MockPaymentGateway {
    fn provider(&self) -> PaymentProvider {
        PaymentProvider::Mock
    }

    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }

    async fn initialize_payment(
        &self,
        payment: &PaymentInitializationRequest,
    ) -> Result<PaymentInitializationResult, DomainError> {
        let fake_url = format!("{}/pay/{}", self.config.base_url, payment.reference);

        Ok(PaymentInitializationResult {
            authorization_url: Some(fake_url),
            provider_reference: payment
                .provider_reference
                .clone()
                .expect("No provider reference"),
            status: PaymentStatus::Processing,
            client_secret: None,
        })
    }
}
