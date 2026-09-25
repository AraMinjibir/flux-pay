use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{errors::domain_error::DomainError, refund::status::RefundStatus, shared::money::Money};

#[derive(Debug, Clone)]
pub struct Refund {
    id: Uuid,
    payment_id: Uuid,
    amount: Money,
    status: RefundStatus,
    reason: Option<String>,
    provider_refund_id:Option<String>,
    idempotency_key: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
}

impl Refund {
    pub fn new(
        id: Uuid,
        payment_id: Uuid,
        amount: Money,
        status: RefundStatus,
        reason: Option<String>,
        provider_refund_id: Option<String>,
        idempotency_key: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            payment_id,
            amount,
            status,
           reason,
            provider_refund_id,
            idempotency_key,
            created_at,
            updated_at,
        }
    }

    pub fn create_refund(
        payment_id:Uuid,
        amount: Money,
        reason: Option<String>,
    ) -> Result<Self, DomainError> {

        let mut errors = Vec::new();

        if payment_id.is_nil(){
            errors.push("Payment id must be provided".to_string())
        }
        if !errors.is_empty() {
            return Err(DomainError::ValidationError(errors));
        }

        let now = Utc::now();
        Ok(Self { 
            id: Uuid::new_v4(),
             payment_id, amount, 
             status: RefundStatus::Processing, 
             reason: reason, 
             provider_refund_id: None,
              idempotency_key: None, 
              created_at: now, 
              updated_at: None
             })
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn payment_id(&self) -> Uuid {
        self.payment_id
    }

    pub fn amount(&self) -> Money {
        self.amount.clone()
    }
    pub fn status(&self) -> RefundStatus {
        self.status.clone()
    }
    pub fn reason(&self) -> Option<String> {
        self.reason.clone()
    }
    pub fn provider_refund_id(&self) -> Option<String>{
        self.provider_refund_id.clone()
    }

    pub fn idempotency_key(&self) -> Option<String>{
        self.idempotency_key.clone()
    }

    pub fn created_at(&self) ->  DateTime<Utc>{
        self.created_at
    }
    pub fn updated_at(&self) ->  Option<DateTime<Utc>>{
        self.updated_at
    }
}
