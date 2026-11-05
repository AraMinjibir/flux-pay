#![allow(dead_code)]

use chrono::Utc;
use flux_pay::{
    application::{
        circuit_breaker::CircuitBreaker,
        payment_orchestrator::{OrchestrationMetadata, OrchestrationResult, PaymentOrchestrator},
    }, domain::{
        idempotency::models::StoredResponse, orchestration::gateway::PaymentGateway, payment::{
            method::PaymentMethod::Card,
            payment::{CreatePaymentCommand, Payment, PaymentInitializationResult},
            provider::PaymentProvider,
            status::PaymentStatus,
        }, refund::{refund::Refund, refund::RefundCreationCommand, status::RefundStatus}, services::{payment_service_impl::PaymentServiceImpl, refund_service::RefundService}, shared::{currency::Currency::{self, NGN}, money::Money},
    }, infrastructure::orchestration::provider_registry::ProviderRegistry,
};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::common::mock_repo::{MockGateway, MockIdempotency, MockIdempotencyRepo, MockPaymentRepo, MockRefundRepo, MockRouting};

pub struct TestContext {
    pub gateway: MockGateway,
    pub payment_repo: MockPaymentRepo,
    pub idempotency: MockIdempotency,
    pub routing: MockRouting,
}
pub struct RefundTestContext {
    pub payment_repo: MockPaymentRepo,
    pub refund_repo: MockRefundRepo,
    pub idempotency: MockIdempotencyRepo,
    pub gateway: MockGateway,
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            gateway: MockGateway::new(),
            payment_repo: MockPaymentRepo::new(),
            idempotency: MockIdempotency::new(),
            routing: MockRouting::new(),
        }
    }

    pub fn command() -> CreatePaymentCommand {
        let payment = test_payment();

        CreatePaymentCommand {
            email: None,
            amount: payment.amount(),
            merchant_id: payment.merchant_id(),
            description: payment.description(),
            payment_method: payment.payment_method(),
            idempotency_key: Uuid::new_v4(),
        }
    }

    pub fn build_service(self) -> PaymentServiceImpl {
        let gateway: Arc<dyn PaymentGateway> = Arc::new(self.gateway);

        let registry = ProviderRegistry::new(vec![gateway]).unwrap();

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
        let orchestrator = Arc::new(PaymentOrchestrator::new(
            Arc::new(registry),
            Arc::new(self.routing),
            circuit_breakers,
        ));

        PaymentServiceImpl::new(
            Arc::new(self.payment_repo),
            Arc::new(self.idempotency),
            orchestrator,
        )
    }
}

impl RefundTestContext {
    pub fn new() -> Self {
        Self {
            payment_repo: MockPaymentRepo::new(),
            refund_repo: MockRefundRepo::new(),
            idempotency: MockIdempotencyRepo::new(),
            gateway: MockGateway::new(),
        }
    }

    pub fn command() -> RefundCreationCommand {
        RefundCreationCommand {
            payment_id: Uuid::new_v4(),
            amount: test_money(5_000),
            reason: Some("customer request".to_string()),
            idempotency_key: Uuid::new_v4(),
        }
    }

    pub fn build_service(self) -> RefundService {
        self.build_service_with_provider(PaymentProvider::Mock)
    }

    pub fn build_service_with_provider(self, provider: PaymentProvider) -> RefundService {
        let mut gateway = self.gateway;
        gateway.expect_provider().return_const(provider);

        let gateway: Arc<dyn PaymentGateway> = Arc::new(gateway);
        let registry = ProviderRegistry::new(vec![gateway]).unwrap();

        RefundService::new(
            Arc::new(self.payment_repo),
            Arc::new(self.refund_repo),
            Arc::new(self.idempotency),
            Arc::new(registry),
        ) 
    }

    pub fn payment(
        status: PaymentStatus,
        currency: Currency,
        provider: Option<PaymentProvider>,
    ) -> Payment {
        let paid_at = matches!(status, PaymentStatus::Success).then(Utc::now);
        let provider_reference = provider.as_ref().map(|_| "PROV-REF-123".to_string());
    
        Payment::new(
            Uuid::new_v4(),
            Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
            Money::new(100_000, currency).unwrap(),
            Some("School fees".to_string()),
            "REF-TEST-001".to_string(),
            status,
            Card,
            provider,
            provider_reference,
            None,
            0,
            None,
            Utc::now(),
            paid_at,
        )
    }
    pub fn pending_refund(payment_id: Uuid) -> Refund {
        Refund::create_refund(payment_id, test_money(5_000), Some("customer request".to_string()))
            .unwrap()
    }

    pub fn stored_response(provider_reference: String) -> StoredResponse {
        StoredResponse {
            payment_id: Uuid::new_v4(),
            provider_reference,
            authorization_url: None,
            client_secret: None,
            status: PaymentStatus::Created,
        }
    }
}
pub fn test_payment() -> Payment {
    let merchant_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
    Payment::generate_payment(
        merchant_id,
        Money::new(100000, NGN).expect("Amout should be positive"),
        Some("School fees".to_string()),
        Card,
    )
    .expect("Test payment should be valid")
}
pub fn mock_execution() -> OrchestrationResult {
    OrchestrationResult {
        initialization: PaymentInitializationResult {
            id: None,
            merchant_id: None,
            amount: None,
            description: None,
            reference: None,
            status: PaymentStatus::Processing,
            selected_provider: Some(PaymentProvider::Mock),
            provider_reference: "MOCK-123".to_string(),
            authorization_url: None,
            client_secret: None,
            created_at: None,
        },
        metadata: OrchestrationMetadata {
            selected_provider: PaymentProvider::Mock,
            retry_count: 0,
            attempted_providers: vec![PaymentProvider::Mock],
        },
    }
}

pub fn test_payload() -> Value {
    json!({
      "email": "test@gmail.com",
      "merchant_id": "550e8400-e29b-41d4-a716-446655443421",
      "amount": {
        "amount": 5500000,
        "currency": "NGN"
      },
      "description": "Gadgets",
      "payment_method": "MobileMoney",
       "idempotency_key": "0c5d0e64-c6b7-3af5-4d43-33c23d8113ae"
    })
}
pub fn test_refund(payment_id: Uuid, amount: i64, idempotency_key: &str) -> Refund {
    let now = Utc::now();
    Refund::new(
        Uuid::new_v4(),
        payment_id,
        test_money(amount),
        RefundStatus::Processing,
        Some("customer request".to_string()),
        None,
        None,
        Some(idempotency_key.to_string()),
        now,
        Some(now),
    )
}
pub fn test_money(amount: i64) -> Money {
    Money::new(amount, Currency::NGN).unwrap()
}
