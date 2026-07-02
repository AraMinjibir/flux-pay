use crate::domain::shared::currency::Currency;

#[derive(Debug, Clone)]
pub struct Money {
    amount: i64,
    currency: Currency,
}

impl Money {
    pub fn new(amount: i64, currency: Currency) -> Self {
        Self { amount, currency }
    }

    pub fn amount(&self) -> i64 {
        self.amount
    }

    pub fn currency(&self) -> Currency {
        self.currency.clone()
    }
}
