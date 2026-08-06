use serde::{Deserialize, Serialize};

use crate::domain::{errors::domain_error::DomainError, shared::currency::Currency};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Money {
    amount: i64,
    currency: Currency,
}

impl Money {
    pub fn new(amount: i64, currency: Currency) -> Result<Self, DomainError> {
        if amount <= 0 {
            return Err(DomainError::InvalidAmount);
        }
        Ok(Self { amount, currency })
    }
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.amount <= 0 {
            return Err(DomainError::InvalidAmount);
        }

        Ok(())
    }

    pub fn amount(&self) -> i64 {
        self.amount
    }

    pub fn currency(&self) -> Currency {
        self.currency.clone()
    }
}
