use async_trait::async_trait;

use crate::{
    domain::{
        errors::domain_error::DomainError,
        orchestration::{capability::ProviderCapabilities, gateway::PaymentGateway},
        payment::{
            method::PaymentMethod,
            payment::{PaymentInitializationRequest, PaymentInitializationResult},
            provider::PaymentProvider,
        },
        shared::currency::Currency,
    },
    infrastructure::{
        config::app_config::PaystackConfig,
        gateways::paystack::{
            paystack_initialization_request::PaystackInitializeRequest,
            paystack_initialization_response::PaystackInitializeResponse,
        },
    },
};
pub struct PaystackGateway {
    config: PaystackConfig,
    client: reqwest::Client,
    capabilities: ProviderCapabilities,
}

impl PaystackGateway {
    pub fn new(config: PaystackConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
            capabilities: ProviderCapabilities {
                currencies: vec![Currency::NGN],
                payment_methods: vec![PaymentMethod::Card, PaymentMethod::BankTransfer],
            },
        }
    }
}
#[async_trait]
impl PaymentGateway for PaystackGateway {
    fn provider(&self) -> PaymentProvider {
        PaymentProvider::Paystack
    }

    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }
    async fn initialize_payment(
        &self,
        payment: &PaymentInitializationRequest,
    ) -> Result<PaymentInitializationResult, DomainError> {
        let request = PaystackInitializeRequest::from(payment);

        let response = self
            .client
            .post(format!("{}/transaction/initialize", self.config.base_url))
            .bearer_auth(&self.config.secret_key)
            .json(&request)
            .send()
            .await?;

        let response = response.json::<PaystackInitializeResponse>().await?;

        Ok(response.into())
    }
}
