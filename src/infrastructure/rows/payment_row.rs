use std::str::FromStr;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{
    errors::domain_error::DomainError,
    payment::{
        method::PaymentMethod, payment::Payment, provider::PaymentProvider, status::PaymentStatus,
    },
    shared::{currency::Currency, money::Money},
};

#[derive(Debug, Clone)]
pub struct PaymentRow {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub amount: i64,
    pub currency: String,
    pub description: Option<String>,
    pub reference: String,
    pub status: String,
    pub payment_method: String,
    pub payment_provider: Option<String>,
    pub provider_reference: Option<String>,
    pub failure_reason: Option<String>,
    pub retry_count: i16,
    pub idempotency_key: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
}
impl PaymentRow {
    pub fn into_domain(self) -> Result<Payment, DomainError> {
        let currency = Currency::from_str(&self.currency)
            .map_err(|e| DomainError::ValidationError(vec![e.to_string()]))?;

        let money = Money::new(self.amount, currency)
            .map_err(|e| DomainError::ValidationError(vec![e.to_string()]))?;

        let status = PaymentStatus::from_str(&self.status)
            .map_err(|e| DomainError::ValidationError(vec![e.to_string()]))?;

        let payment_method = PaymentMethod::from_str(&self.payment_method)
            .map_err(|e| DomainError::ValidationError(vec![e.to_string()]))?;

        let provider = self
            .payment_provider
            .as_deref()
            .map(PaymentProvider::from_str)
            .transpose()
            .map_err(|e| DomainError::ValidationError(vec![e.to_string()]))?;

        Ok(Payment::new(
            self.id,
            self.merchant_id,
            money,
            self.description,
            self.reference,
            status,
            payment_method,
            provider,
            self.provider_reference,
            self.failure_reason,
            self.retry_count,
            self.idempotency_key,
            self.created_at,
            self.paid_at,
        ))
    }
    pub fn from_domain(domain: &Payment) -> Self {
        Self {
            id: domain.id(),
            merchant_id: domain.merchant_id(),
            amount: domain.amount().amount(),
            currency: domain.amount().currency().to_string(),
            description: domain.description(),
            reference: domain.reference(),
            status: domain.status().to_string(),
            payment_method: domain.payment_method().to_string(),
            payment_provider: domain.provider().map(|p| p.to_string()),
            provider_reference: domain.provider_reference(),
            failure_reason: domain.failure_reason(),
            retry_count: domain.retry_count(),
            idempotency_key: domain.idempotency_key(),
            created_at: domain.created_at(),
            paid_at: Some(domain.created_at()),
        }
    }
}
