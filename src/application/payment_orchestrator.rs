use std::sync::Arc;

use std::time::Duration;

use rand::Rng;
use tracing::info;

use crate::{
    domain::{
        errors::domain_error::DomainError,
        orchestration::{routing_request::RoutingRequest, routing_strategy::RoutingStrategy},
        payment::{
            payment::{PaymentInitializationRequest, PaymentInitializationResult},
            provider::PaymentProvider,
        },
    },
    infrastructure::orchestration::provider_registry::ProviderRegistry,
};
pub struct PaymentOrchestrator {
    registry: Arc<ProviderRegistry>,
    routing_strategy: Arc<dyn RoutingStrategy>,
}
#[derive(Debug, Clone)]
pub struct OrchestrationMetadata {
    pub selected_provider: PaymentProvider,
    pub retry_count: i16,
    pub attempted_providers: Vec<PaymentProvider>,
}
#[derive(Debug, Clone)]
pub struct OrchestrationFailureMetadata {
    pub retry_count: i16,
    pub attempted_providers: Vec<PaymentProvider>,
}
#[derive(Debug, Clone)]
pub struct OrchestrationResult {
    pub initialization: PaymentInitializationResult,
    pub metadata: OrchestrationMetadata,
}

impl PaymentOrchestrator {
    const MAX_RETRIES: u32 = 3;
    const BASE_RETRY_DELAY_MS: u64 = 500;
    const MAX_RETRY_DELAY_MS: u64 = 10_000;

    pub fn new(
        registry: Arc<ProviderRegistry>,
        routing_strategy: Arc<dyn RoutingStrategy>,
    ) -> Self {
        Self {
            registry,
            routing_strategy,
        }
    }

    pub async fn initialize_payment(
        &self,
        request: &PaymentInitializationRequest,
    ) -> Result<OrchestrationResult, DomainError> {
        let routing_request = RoutingRequest {
            currency: request.currency.clone(),
        };

        let providers = self.routing_strategy.select(&routing_request);

        if providers.is_empty() {
            return Err(DomainError::NoProviderAvailable);
        }

        let mut retry_count = 0;
        let mut last_error = None;
        let mut attempted_providers = Vec::new();

        for provider in &providers {
            attempted_providers.push(provider.clone());

            let gateway = self
                .registry
                .get(provider)
                .ok_or_else(|| DomainError::ProviderNotFound(provider.clone()))?;

            for attempt in 1..=Self::MAX_RETRIES {
                info!("Trying provider {:?}, attempt {}", provider, attempt);

                match gateway.initialize_payment(request).await {
                    Ok(initialization) => {
                        return Ok(OrchestrationResult {
                            initialization,
                            metadata: OrchestrationMetadata {
                                selected_provider: provider.clone(),
                                retry_count,
                                attempted_providers,
                            },
                        });
                    }

                    Err(error) => {
                        info!("Provider failed: {:?}", provider);
                        info!("Error: {:?}", error);

                        let retryable = error.is_retryable();

                        info!("Retryable: {}", retryable);

                        last_error = Some(error);

                        if retryable && attempt < Self::MAX_RETRIES {
                            retry_count += 1;

                            let delay = Self::calculate_retry_delay(attempt);

                            info!("Retrying provider {:?} after {:?}", provider, delay);

                            tokio::time::sleep(delay).await;

                            continue;
                        }

                        break;
                    }
                }
            }
        }

        Err(DomainError::PaymentProviderFailed {
            error: Box::new(last_error.unwrap_or(DomainError::NoProviderAvailable)),
            metadata: OrchestrationFailureMetadata {
                retry_count,
                attempted_providers,
            },
        })
    }

    fn calculate_retry_delay(attempt: u32) -> Duration {
        let exponential_delay = Self::BASE_RETRY_DELAY_MS
            .saturating_mul(2u64.saturating_pow(attempt.saturating_sub(1)));

        let capped_delay = exponential_delay.min(Self::MAX_RETRY_DELAY_MS);

        let jitter = rand::rng().random_range(0..=capped_delay);

        Duration::from_millis(jitter)
    }
}
