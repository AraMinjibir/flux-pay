use std::sync::Arc;

use flux_pay::{
    application::payment_orchestrator::PaymentOrchestrator,
    domain::{
        orchestration::gateway::PaymentGateway,
        payment::{
            method::PaymentMethod::Card,
            payment::{CreatePaymentCommand, Payment},
        },
        services::payment_service_impl::PaymentServiceImpl,
        shared::{currency::Currency::NGN, money::Money},
    },
    infrastructure::orchestration::provider_registry::ProviderRegistry,
};
use uuid::Uuid;

use crate::common::mock_repo::{MockGateway, MockIdempotency, MockPaymentRepo, MockRouting};

pub struct TestContext {
    pub gateway: MockGateway,
    pub payment_repo: MockPaymentRepo,
    pub idempotency: MockIdempotency,
    pub routing: MockRouting,
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

        let orchestrator = Arc::new(PaymentOrchestrator::new(
            Arc::new(registry),
            Arc::new(self.routing),
        ));

        PaymentServiceImpl::new(
            Arc::new(self.payment_repo),
            Arc::new(self.idempotency),
            orchestrator,
        )
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
