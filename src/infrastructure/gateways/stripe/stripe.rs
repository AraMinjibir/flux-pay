use async_trait::async_trait;

use crate::{
    domain::{
        errors::domain_error::DomainError,
        ochestration::{capability::ProviderCapabilities, gateway::PaymentGateway},
        payment::{
            method::PaymentMethod,
            payment::{PaymentInitializationRequest, PaymentInitializationResult},
            provider::PaymentProvider,
        },
        shared::currency::Currency,
    },
    infrastructure::{
        config::app_config::StripeConfig,
        gateways::stripe::{
            stripe_initent_request::StripePaymentIntentRequest,
            stripe_payment_initent_response::StripePaymentIntentResponse,
        },
    },
};
pub struct StripeGateway {
    config: StripeConfig,
    client: reqwest::Client,
    capabilities: ProviderCapabilities,
}
impl StripeGateway {
    pub fn new(config: StripeConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
            capabilities: ProviderCapabilities {
                currencies: vec![Currency::USD, Currency::EUR, Currency::GBP],
                payment_methods: vec![PaymentMethod::Card],
            },
        }
    }
}

#[async_trait]
impl PaymentGateway for StripeGateway {
    fn provider(&self) -> PaymentProvider {
        PaymentProvider::Stripe
    }

    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }
    async fn initialize_payment(
        &self,
        payment: &PaymentInitializationRequest,
    ) -> Result<PaymentInitializationResult, DomainError> {
        let request = StripePaymentIntentRequest::from(payment);

        let response = self
            .client
            .post(format!("{}/v1/payment_intents", self.config.base_url,))
            .bearer_auth(&self.config.secret_key)
            .form(&request)
            .send()
            .await?;

        let response = response.json::<StripePaymentIntentResponse>().await?;
        Ok(response.into())
    }
}
