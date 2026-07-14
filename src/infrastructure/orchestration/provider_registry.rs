use std::{collections::HashMap, sync::Arc};

use crate::domain::{
    errors::domain_error::DomainError, ochestration::gateway::PaymentGateway,
    payment::provider::PaymentProvider,
};

pub struct ProviderRegistry {
    gateways: HashMap<PaymentProvider, Arc<dyn PaymentGateway>>,
}

impl ProviderRegistry {
    pub fn new(gateways: Vec<Arc<dyn PaymentGateway>>) -> Result<Self, DomainError> {
        let mut map = HashMap::new();

        for gateway in gateways {
            map.insert(gateway.provider(), gateway);
        }

        Ok(Self { gateways: map })
    }

    pub fn get(&self, provider: &PaymentProvider) -> Option<Arc<dyn PaymentGateway>> {
        self.gateways.get(provider).cloned()
    }
}
