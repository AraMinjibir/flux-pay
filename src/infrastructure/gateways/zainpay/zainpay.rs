use async_trait::async_trait;

use crate::domain::payment::payment::PaymentInitializationRequest;
use crate::{
    domain::{
        errors::domain_error::DomainError,
        orchestration::{capability::ProviderCapabilities, gateway::PaymentGateway},
        payment::{
            method::PaymentMethod, payment::PaymentInitializationResult, provider::PaymentProvider,
        },
        shared::currency::Currency,
    },
    infrastructure::{
        config::app_config::ZainpayConfig,
        gateways::zainpay::{
            zainpay_initialization_request::ZainpayInitializationRequest,
            zainpay_initialization_response::ZainpayInitializationResponse,
        },
    },
};

pub struct ZainpayGateway {
    config: ZainpayConfig,
    client: reqwest::Client,
    capabilities: ProviderCapabilities,
}

impl ZainpayGateway {
    pub fn new(config: ZainpayConfig) -> Self {
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
impl PaymentGateway for ZainpayGateway {
    fn provider(&self) -> PaymentProvider {
        PaymentProvider::Zainpay
    }

    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }
    async fn initialize_payment(
        &self,
        payment: &PaymentInitializationRequest,
    ) -> Result<PaymentInitializationResult, DomainError> {
        let request = ZainpayInitializationRequest::from(payment);

        let response = self
            .client
            .post(format!("{}/transaction/initialize", self.config.base_url))
            .bearer_auth(&self.config.secret_key)
            .json(&request)
            .send()
            .await?;

        response.status();

        let body = response.text().await?;

        let parsed: ZainpayInitializationResponse = serde_json::from_str(&body)?;

        Ok(parsed.into())
    }
}
