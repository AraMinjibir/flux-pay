use serde::Deserialize;
use uuid::Uuid;

use crate::domain::{
    errors::domain_error::DomainError,
    payment::{method::PaymentMethod, payment::CreatePaymentCommand},
    shared::money::Money,
};

#[derive(Debug, Deserialize)]
pub struct PaymentCreationRequest {
    email: Option<String>,
    merchant_id: Uuid,
    amount: Money,
    description: Option<String>,
    payment_method: PaymentMethod,
    idempotency_key: Uuid,
}

impl PaymentCreationRequest {
    pub fn into_domain(self) -> Result<CreatePaymentCommand, DomainError> {
        self.amount.validate()?;

        Ok(CreatePaymentCommand {
            email: self.email,
            merchant_id: self.merchant_id,
            amount: self.amount,
            description: self.description,
            payment_method: self.payment_method,
            idempotency_key: self.idempotency_key,
        })
    }
}
