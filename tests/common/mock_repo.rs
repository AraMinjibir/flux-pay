use async_trait::async_trait;
use mockall::mock;
use uuid::Uuid;

use flux_pay::domain::{
    errors::{domain_error::DomainError, repository_error::RepositoryError}, idempotency::models::{ReservationResult, StoredResponse}, orchestration::{capability::ProviderCapabilities, gateway::PaymentGateway, routing_request::RoutingRequest, routing_strategy::RoutingStrategy}, payment::{
        method::PaymentMethod, payment::{Payment, PaymentInitializationRequest, PaymentInitializationResult}, provider::PaymentProvider, repository::PaymentRepository, status::PaymentStatus,
    }, services::idempotency_service::IdempotencyService,
};


mock! {
    pub  PaymentRepo {}

    #[async_trait]
    impl PaymentRepository for PaymentRepo {
        async fn save(&self, payment: &Payment) -> Result<(), RepositoryError>;

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Payment>, RepositoryError>;

        async fn find_by_reference(&self, reference: &str) -> Result<Option<Payment>, RepositoryError>;

        async fn find_by_status(&self, status: &PaymentStatus)
        -> Result<Vec<Payment>, RepositoryError>;

        async fn find_by_provider(
            &self,
            provider: &PaymentProvider,
        ) -> Result<Vec<Payment>, RepositoryError>;

        async fn find_by_method(&self, method: &PaymentMethod)
        -> Result<Vec<Payment>, RepositoryError>;

        async fn find_by_merchant(&self, merchant_id: Uuid) -> Result<Vec<Payment>, RepositoryError>;

        async fn find_all(&self) -> Result<Vec<Payment>, RepositoryError>;

        async fn update(&self, payment: &Payment) -> Result<(), RepositoryError>;

        async fn delete(&self, id: Uuid) -> Result<u64, RepositoryError>;
}
}

mock! {
    pub Idempotency{}

    #[async_trait]
    impl IdempotencyService for Idempotency {

        async fn reserve(&self, key: Uuid) -> Result<ReservationResult, DomainError>;

    async fn complete(&self, key: Uuid, response: StoredResponse) -> Result<(), DomainError>;

    }
}

mock! {
  pub Routing {}

    #[async_trait]
    impl RoutingStrategy for Routing {

        fn select(&self, request: &RoutingRequest) -> Vec<PaymentProvider>;
        
    }
}

mock! {
   pub Gateway {}

    #[async_trait]
    impl PaymentGateway for Gateway {

        fn provider(&self) -> PaymentProvider;

    fn capabilities(&self) -> &ProviderCapabilities;

    async fn initialize_payment(
        &self,
        payment: &PaymentInitializationRequest,
    ) -> Result<PaymentInitializationResult, DomainError>;

    }
}