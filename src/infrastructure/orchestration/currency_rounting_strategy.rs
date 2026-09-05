use std::collections::HashMap;

use crate::domain::{
    orchestration::{routing_request::RoutingRequest, routing_strategy::RoutingStrategy},
    payment::provider::PaymentProvider,
    shared::currency::Currency,
};

pub struct CurrencyRoutingStrategy {
    routes: HashMap<Currency, Vec<PaymentProvider>>,
}

impl CurrencyRoutingStrategy {
    pub fn new() -> Self {
        let mut routes = HashMap::new();

        routes.insert(
            Currency::NGN,
            vec![
                PaymentProvider::Zainpay,
                PaymentProvider::Paystack,
                PaymentProvider::Interswitch,
                PaymentProvider::Mock,
            ],
        );

        routes.insert(
            Currency::USD,
            vec![PaymentProvider::Stripe, PaymentProvider::Mock],
        );

        routes.insert(
            Currency::EUR,
            vec![PaymentProvider::Stripe, PaymentProvider::Mock],
        );

        routes.insert(
            Currency::GBP,
            vec![PaymentProvider::Stripe, PaymentProvider::Mock],
        );

        Self { routes }
    }
}

impl RoutingStrategy for CurrencyRoutingStrategy {
    fn select(&self, request: &RoutingRequest) -> Vec<PaymentProvider> {
        self.routes
            .get(&request.currency)
            .cloned()
            .unwrap_or_default()
    }
}
