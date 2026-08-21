#![allow(dead_code)]

use std::{
    collections::HashMap,
    net::TcpListener,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use actix_web::{App, HttpResponse, HttpServer, web};
use async_trait::async_trait;
use flux_pay::{
    application::{circuit_breaker::CircuitBreaker, payment_orchestrator::PaymentOrchestrator},
    config::{app_state::AppState, routes},
    domain::{
        errors::domain_error::DomainError,
        orchestration::{
            capability::ProviderCapabilities, gateway::PaymentGateway,
            routing_request::RoutingRequest, routing_strategy::RoutingStrategy,
        },
        payment::{
            method::PaymentMethod,
            payment::{PaymentInitializationRequest, PaymentInitializationResult},
            provider::PaymentProvider,
        },
        services::{
            idempotency_service_impl::IdempotencyServiceImpl,
            payment_service_impl::PaymentServiceImpl,
        },
        shared::currency::Currency,
    },
    infrastructure::{
        config::app_config::MockConfig, gateways::mock::MockPaymentGateway,
        orchestration::provider_registry::ProviderRegistry,
        redis::redis_idempotency_repository::RedisIdempotencyRepository,
        repositories::postgres_payment_repository::PostgresPaymentRepository,
    },
};
use reqwest::Client;
use sqlx::{PgPool, postgres::PgPoolOptions};
use testcontainers::{ContainerAsync, runners::AsyncRunner};
use testcontainers_modules::{postgres::Postgres, redis::Redis};
use tokio::sync::Mutex;

pub struct TestContext {
    pub address: String,
    pub client: Client,
    pub pool: PgPool,
    pub postgres: Option<ContainerAsync<Postgres>>,
    pub redis: Option<ContainerAsync<Redis>>,
    pub provider: TestProvider,
}

impl TestContext {
    pub async fn create_payment(
        &self,
        payload: &serde_json::Value,
    ) -> reqwest::Result<reqwest::Response> {
        self.client
            .post(format!("{}/api/v1/payments", self.address))
            .json(payload)
            .send()
            .await
    }
}

pub struct TestProvider {
    pub address: String,
    pub requests: Arc<AtomicUsize>,
}

pub async fn spawn_app() -> TestContext {
    spawn_app_with_routing(Arc::new(TestRoutingStrategy)).await
}

pub async fn spawn_app_with_routing(routing: Arc<dyn RoutingStrategy>) -> TestContext {
    // 1. Start PostgreSQL
    let postgres = Postgres::default()
        .start()
        .await
        .expect("failed to start postgres");

    let postgres_port = postgres
        .get_host_port_ipv4(5432)
        .await
        .expect("failed to get postgres port");

    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{postgres_port}/postgres");

    // 2. Start Redis
    let redis = Redis::default()
        .start()
        .await
        .expect("failed to start redis");

    let redis_port = redis
        .get_host_port_ipv4(6379)
        .await
        .expect("failed to get redis port");

    // 3. Start test provider
    let provider = spawn_timeout_provider().await;

    let redis_url = format!("redis://127.0.0.1:{redis_port}");
    let provider_url = provider.address.clone();

    // 4. Connect to PostgreSQL
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to postgres");

    // 5. Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations failed");

    // 6. Build application state
    let state = build_test_app_state(pool.clone(), redis_url, provider_url, routing).await;

    let state = web::Data::new(state);

    // 7. Bind Actix to an available port
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind test server");

    let address = format!(
        "http://{}",
        listener.local_addr().expect("failed to get local address")
    );

    // 8. Start Actix
    let server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .configure(routes::configure)
    })
    .listen(listener)
    .expect("failed to start test server")
    .run();

    tokio::spawn(server);

    // 9. HTTP client
    let client = reqwest::Client::new();

    // 10. Return test context
    TestContext {
        address,
        client,
        pool,
        postgres: Some(postgres),
        redis: Some(redis),
        provider,
    }
}

pub async fn build_test_app_state(
    pool: PgPool,
    redis_url: String,
    provider_url: String,
    routing: Arc<dyn RoutingStrategy>,
) -> AppState {
    let payment_repository = Arc::new(PostgresPaymentRepository::new(pool.clone()));

    // Redis
    let redis_client = redis::Client::open(redis_url).expect("failed to create Redis client");

    let redis_repository = Arc::new(RedisIdempotencyRepository::new(redis_client));

    let idempotency_service = Arc::new(IdempotencyServiceImpl::new(redis_repository));

    // Test providers
    // Paystack -> TimeoutGateway
    // Mock     -> fallback
    let timeout_gateway = Arc::new(TimeoutGateway::new(provider_url));

    let mock = Arc::new(MockPaymentGateway::new(MockConfig {
        base_url: "http://localhost".to_string(),
    }));

    let registry = Arc::new(
        ProviderRegistry::new(vec![timeout_gateway, mock])
            .expect("failed to build provider registry"),
    );

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

    // Orchestrator
    let orchestrator = Arc::new(PaymentOrchestrator::new(
        registry.clone(),
        routing,
        circuit_breakers,
    ));

    // Payment service
    let payment_service = Arc::new(PaymentServiceImpl::new(
        payment_repository.clone(),
        idempotency_service,
        orchestrator,
    ));

    AppState {
        payment_service,
        payment_repository,
        provider_registry: registry,
    }
}

pub struct TestRoutingStrategy;

impl RoutingStrategy for TestRoutingStrategy {
    fn select(&self, _request: &RoutingRequest) -> Vec<PaymentProvider> {
        vec![PaymentProvider::Paystack, PaymentProvider::Mock]
    }
}

pub struct TimeoutGateway {
    client: reqwest::Client,
    base_url: String,
    capabilities: ProviderCapabilities,
}

impl TimeoutGateway {
    pub fn new(base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(100))
            .build()
            .expect("failed to create client");

        Self {
            client,
            base_url,
            capabilities: ProviderCapabilities {
                currencies: vec![Currency::NGN],
                payment_methods: vec![PaymentMethod::Card],
            },
        }
    }
}

#[async_trait]
impl PaymentGateway for TimeoutGateway {
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
        let url = format!("{}/pay/{}", self.base_url, payment.reference);

        self.client
            .post(url)
            .send()
            .await
            .map_err(|error| DomainError::ReqwestError(error.to_string()))?;

        unreachable!()
    }
}

pub async fn spawn_timeout_provider() -> TestProvider {
    let requests = Arc::new(AtomicUsize::new(0));
    let counter = requests.clone();

    let server = HttpServer::new(move || {
        let counter = counter.clone();

        App::new().default_service(web::route().to(move || {
            let counter = counter.clone();

            async move {
                counter.fetch_add(1, Ordering::SeqCst);

                tokio::time::sleep(Duration::from_secs(5)).await;

                HttpResponse::Ok().finish()
            }
        }))
    })
    .bind(("127.0.0.1", 0))
    .expect("failed to bind test provider");

    let address = server
        .addrs()
        .first()
        .expect("provider did not bind")
        .to_string();

    let server = server.run();

    tokio::spawn(server);
    TestProvider {
        address: format!("http://{}", address),
        requests,
    }
}

pub struct RetryExhaustedRoutingStrategy;

impl RoutingStrategy for RetryExhaustedRoutingStrategy {
    fn select(&self, _request: &RoutingRequest) -> Vec<PaymentProvider> {
        vec![PaymentProvider::Paystack]
    }
}
