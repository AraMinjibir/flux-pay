use std::str::FromStr;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{
    errors::domain_error::DomainError,
    refund::{refund::Refund, status::RefundStatus},
    shared::{currency::Currency, money::Money},
};

pub struct RefundRow {
    pub id: Uuid,
    pub payment_id: Uuid,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub reason: Option<String>,
    pub failure_reason: Option<String>,
    pub provider_refund_id: Option<String>,
    pub idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl RefundRow {
    pub fn into_refund_domain(self) -> Result<Refund, DomainError> {
        let currency = Currency::from_str(&self.currency)
            .map_err(|e| DomainError::ValidationError(vec![e.to_string()]))?;

        let money = Money::new(self.amount, currency)
            .map_err(|e| DomainError::ValidationError(vec![e.to_string()]))?;

        let status = RefundStatus::from_str(&self.status)
            .map_err(|e| DomainError::ValidationError(vec![e.to_string()]))?;

        Ok(Refund::new(
            self.id,
            self.payment_id,
            money,
            status,
            self.reason,
            self.failure_reason,
            self.provider_refund_id,
            self.idempotency_key,
            self.created_at,
            self.updated_at,
        ))
    }

    pub fn from_refund_domain(refund: &Refund) -> Self {
        Self {
            id: refund.id(),
            payment_id: refund.payment_id(),
            amount: refund.amount().amount(),
            currency: refund.amount().currency().to_string(),
            status: refund.status().to_string(),
            reason: refund.reason(),
            failure_reason: refund.failure_reason(),
            provider_refund_id: refund.provider_refund_id(),
            idempotency_key: refund.idempotency_key(),
            created_at: refund.created_at(),
            updated_at: refund.updated_at(),
        }
    }
}
