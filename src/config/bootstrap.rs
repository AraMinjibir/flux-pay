use std::{collections::HashMap, env, sync::Arc};

use redis::Client;
use sqlx::PgPool;
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
            InterswitchConfig, MockConfig, PaystackConfig, StripeConfig, ZainpayConfig,
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

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to DB");

    let payment_repository = Arc::new(PostgresPaymentRepository::new(pool.clone()));

    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");

    let redis_client = Client::open(redis_url).expect("Failed to create Redis client");
    let redis_repository = Arc::new(RedisIdempotencyRepository::new(redis_client));

    let idempotency_service = Arc::new(IdempotencyServiceImpl::new(redis_repository));

    let paystack_config = PaystackConfig {
        secret_key: env::var("PAYSTACK_SECRET_KEY").expect("PAYSTACK_SECRET_KEY must be set"),

        base_url: env::var("PAYSTACK_BASE_URL").expect("PAYSTACK_BASE_URL must be set"),
    };
    let mock_config = MockConfig {
        base_url: env::var("MOCK_BASE_URL").expect("MOCK_BASE_URL must be set"),
    };

    let interswitch_config = InterswitchConfig {
        base_url: env::var("INTER_SWITCH_BASE_URL").expect("INTER_SWITCH_BASE_URL must be set"),
        client_id: env::var("INTER_SWITCH_CLIENT_ID").expect("INTER_SWITCH_CLIENT_ID must be set"),
        secret_key: env::var("INTER_SWITCH_SECRET_KEY")
            .expect("INTER_SWITCH_SECRET_KEY must be set"),
    };

    let stripe_config = StripeConfig {
        secret_key: env::var("STRIPE_SECRET_KEY").expect("STRIPE_SECRET_KEY must be set"),

        base_url: env::var("STRIPE_BASE_URL").expect("STRIPE_BASE_URL must be set"),
    };
    let zainpay_config = ZainpayConfig {
        base_url: env::var("ZAINPAY_BASE_URL").expect("ZAINPAY_BASE_URL must be set"),
        secret_key: env::var("ZAINPAY_SECRET_KEY").expect("ZAINPAY_SECRET_KEY must be set"),
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

    println!("App state created.");
    Ok(AppState {
        payment_service,
        payment_repository,
        provider_registry,
    })
}
