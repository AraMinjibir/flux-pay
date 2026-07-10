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
        config::app_config::InterswitchConfig,
        gateways::interswitch::{
            interswitch_initialization_request::InterswitchInitializeRequest,
            interswitch_initialization_response::InterswitchInitializeResponse,
        },
    },
};

pub struct InterswitchGateway {
    config: InterswitchConfig,
    client: reqwest::Client,
    capabilities: ProviderCapabilities,
}

impl InterswitchGateway {
    pub fn new(config: InterswitchConfig) -> Self {
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
impl PaymentGateway for InterswitchGateway {
    fn provider(&self) -> PaymentProvider {
        PaymentProvider::Interswitch
    }

    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }

    async fn initialize_payment(
        &self,
        payment: &PaymentInitializationRequest,
    ) -> Result<PaymentInitializationResult, DomainError> {
        let request = InterswitchInitializeRequest::from(payment);

        let response = self
            .client
            .post(format!("{}/api/v3/purchases", self.config.base_url,))
            .header("Client-Id", &self.config.client_id)
            .header("Client-Secret", &self.config.secret_key)
            .json(&request)
            .send()
            .await?;
        let response = response.json::<InterswitchInitializeResponse>().await?;
        Ok(response.into())
    }
}
