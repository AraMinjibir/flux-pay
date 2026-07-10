use crate::domain::{payment::method::PaymentMethod, shared::currency::Currency};

pub struct ProviderCapabilities {
    pub currencies: Vec<Currency>,
    pub payment_methods: Vec<PaymentMethod>,
}
