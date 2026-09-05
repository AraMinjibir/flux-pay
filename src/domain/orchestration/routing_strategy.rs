use crate::domain::{
    orchestration::routing_request::RoutingRequest, payment::provider::PaymentProvider,
};

pub trait RoutingStrategy: Send + Sync {
    fn select(&self, request: &RoutingRequest) -> Vec<PaymentProvider>;
}
