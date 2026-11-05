use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{
    errors::domain_error::DomainError,
    refund::{
        refund::{Refund, RefundCreationCommand},
        status::RefundStatus,
    },
    shared::money::Money,
};

#[derive(Debug, Deserialize)]
pub struct RefundCreationRequest {
    pub amount: Money,
    pub reason: Option<String>,
}

impl RefundCreationRequest {
    pub fn into_domain(
        self,
        payment_id: Uuid,
        idempotency_key: Uuid,
    ) -> Result<RefundCreationCommand, DomainError> {
        self.amount.validate()?;

        Ok(RefundCreationCommand {
            payment_id,
            amount: self.amount,
            reason: self.reason,
            idempotency_key,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct RefundResponse {
    pub id: Uuid,
    pub payment_id: Uuid,
    pub amount: Money,
    pub status: RefundStatus,
    pub reason: Option<String>,
    pub failure_reason: Option<String>,
    pub provider_refund_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl RefundResponse {
    pub fn from_domain(refund: &Refund) -> Self {
        Self {
            id: refund.id(),
            payment_id: refund.payment_id(),
            amount: refund.amount().clone(),
            status: refund.status().clone(),
            reason: refund.reason(),
            failure_reason: refund.failure_reason(),
            provider_refund_id: refund.provider_refund_id(),
            created_at: refund.created_at(),
            updated_at: refund.updated_at(),
        }
    }
}
