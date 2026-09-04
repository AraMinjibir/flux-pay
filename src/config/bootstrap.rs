use std::{collections::HashMap, env, sync::Arc};

use redis::Client;
use sqlx::PgPool;
use tracing::info;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::{
    application::{circuit_breaker::CircuitBreaker, payment_orchestrator::PaymentOrchestrator},
    config::app_state::AppState,
    domain::{
        errors::domain_error::DomainError,
        payment::provider::PaymentProvider,
        services::{
            idempotency_service_impl::IdempotencyServiceImpl,
            payment_service_impl::PaymentServiceImpl,
        },
    },
    infrastructure::{
        config::app_config::{
            IdempotencyConfig, InterswitchConfig, MockConfig, PaystackConfig, StripeConfig,
            ZainpayConfig,
        },
        gateways::{
            interswitch::interswitch::InterswitchGateway, mock::MockPaymentGateway,
            paystack::paystack::PaystackGateway, stripe::stripe::StripeGateway,
            zainpay::zainpay::ZainpayGateway,
        },
        orchestration::{
            currency_rounting_strategy::CurrencyRoutingStrategy,
            provider_registry::ProviderRegistry,
        },
        redis::redis_idempotency_repository::RedisIdempotencyRepository,
        repositories::postgres_payment_repository::PostgresPaymentRepository,
    },
};

pub async fn build_app_state() -> Result<AppState, DomainError> {
    // Load DB connection

    let database_url = std::env::var("DATABASE_URL")
    .map_err(|_|DomainError::ConfigurationError("Database Url must to be set".to_string()))?;

    let pool = PgPool::connect(&database_url)
    .await
    .map_err(|_| DomainError::ConnectionError)?;

    let payment_repository = Arc::new(PostgresPaymentRepository::new(pool.clone()));

    let redis_url = env::var("REDIS_URL")
    .map_err(|_|DomainError::ConfigurationError("Redis Url must to be set".to_string()))?;

    let redis_client = Client::open(redis_url)
    .map_err(|_|DomainError::ConfigurationError("Redis Url must to be set".to_string()))?;
    let idempotency_config = IdempotencyConfig::default();
    let redis_repository = Arc::new(RedisIdempotencyRepository::new(
        redis_client,
        idempotency_config,
    ));

    let idempotency_service = Arc::new(IdempotencyServiceImpl::new(redis_repository));

    let paystack_config = PaystackConfig {
        secret_key: env::var("PAYSTACK_SECRET_KEY").map_err(|_|DomainError::ConfigurationError("PAYSTACK_SECRET_KEY must to be set".to_string()))?,

        base_url: env::var("PAYSTACK_BASE_URL").map_err(|_|DomainError::ConfigurationError("PAYSTACK_BASE_URL must to be set".to_string()))?,
    };
    let mock_config = MockConfig {
        base_url: env::var("MOCK_BASE_URL").map_err(|_|DomainError::ConfigurationError("MOCK_BASE_URL must to be set".to_string()))?,
    };

    let interswitch_config = InterswitchConfig {
        base_url: env::var("INTER_SWITCH_BASE_URL").map_err(|_|DomainError::ConfigurationError("INTER_SWITCH_BASE_URL must to be set".to_string()))?,
        client_id: env::var("INTER_SWITCH_CLIENT_ID").map_err(|_|DomainError::ConfigurationError("INTER_SWITCH_CLIENT_ID must to be set".to_string()))?,
        secret_key: env::var("INTER_SWITCH_SECRET_KEY")
        .map_err(|_|DomainError::ConfigurationError("INTER_SWITCH_SECRET_KEY must to be set".to_string()))?,
    };

    let stripe_config = StripeConfig {
        secret_key: env::var("STRIPE_SECRET_KEY").map_err(|_|DomainError::ConfigurationError("STRIPE_SECRET_KEY must to be set".to_string()))?,

        base_url: env::var("STRIPE_BASE_URL").map_err(|_|DomainError::ConfigurationError("STRIPE_BASE_URL must to be set".to_string()))?,
    };
    let zainpay_config = ZainpayConfig {
        base_url: env::var("ZAINPAY_BASE_URL").map_err(|_|DomainError::ConfigurationError("ZAINPAY_BASE_URL must to be set".to_string()))?,
        secret_key: env::var("ZAINPAY_SECRET_KEY").map_err(|_|DomainError::ConfigurationError("ZAINPAY_SECRET_KEY must to be set".to_string()))?,
    };
    let paystack = Arc::new(PaystackGateway::new(paystack_config));
    let mock = Arc::new(MockPaymentGateway::new(mock_config));
    let stripe = Arc::new(StripeGateway::new(stripe_config));
    let interswitch = Arc::new(InterswitchGateway::new(interswitch_config));
    let zainpay = Arc::new(ZainpayGateway::new(zainpay_config));

    let routing = Arc::new(CurrencyRoutingStrategy::new());

    let provider_registry =
        ProviderRegistry::new(vec![paystack, interswitch, stripe, mock, zainpay])?;
    let circuit_breakers = HashMap::from([
        (
            PaymentProvider::Zainpay,
            Arc::new(Mutex::new(CircuitBreaker::new(3, Duration::from_secs(30)))),
        ),
        (
            PaymentProvider::Paystack,
            Arc::new(Mutex::new(CircuitBreaker::new(3, Duration::from_secs(30)))),
        ),
        (
            PaymentProvider::Interswitch,
            Arc::new(Mutex::new(CircuitBreaker::new(3, Duration::from_secs(30)))),
        ),
        (
            PaymentProvider::Stripe,
            Arc::new(Mutex::new(CircuitBreaker::new(3, Duration::from_secs(30))))
        ),
        (
            PaymentProvider::Mock,
            Arc::new(Mutex::new(CircuitBreaker::new(3, Duration::from_secs(30)))),
        ),
    ]);

    let provider_registry = Arc::new(provider_registry);

    let orchestrator = Arc::new(PaymentOrchestrator::new(
        provider_registry.clone(),
        routing,
        circuit_breakers,
    ));

    let payment_service = Arc::new(PaymentServiceImpl::new(
        payment_repository.clone(),
        idempotency_service,
        orchestrator,
    ));

    info!("FluxPay application state initialized");
        Ok(AppState {
        payment_service,
        payment_repository,
        provider_registry,
    })
}
