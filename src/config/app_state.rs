use std::sync::Arc;

use crate::{
    domain::{payment::repository::PaymentRepository, services::payment_service::PaymentService},
    infrastructure::orchestration::provider_registry::ProviderRegistry,
};

pub struct AppState {
    pub payment_service: Arc<dyn PaymentService>,
    pub payment_repository: Arc<dyn PaymentRepository>,
    pub provider_registry: Arc<ProviderRegistry>,
}
