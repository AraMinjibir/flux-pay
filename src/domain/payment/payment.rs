use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{
    payment::{method::PaymentMethod, provider::PaymentProvider, status::PaymentStatus},
    shared::money::Money,
};

#[derive(Debug, Clone)]
pub struct Payment {
    id: Uuid,
    merchant_id: Uuid,
    amount: Money,
    description: Option<String>,
    reference: String,
    status: PaymentStatus,
    payment_method: PaymentMethod,
    provider: PaymentProvider,
    provider_reference: Option<String>,
    failure_reason: Option<String>,
    retry_count: i16,
    idempotency_key: Option<Uuid>,
    paid_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
}

impl Payment {
    pub fn new(
        id: Uuid,
        merchant_id: Uuid,
        amount: Money,
        description: Option<String>,
        reference: String,
        status: PaymentStatus,
        payment_method: PaymentMethod,
        provider: PaymentProvider,
        provider_reference: Option<String>,
        failure_reason: Option<String>,
        retry_count: i16,
        idempotency_key: Option<Uuid>,
        paid_at: DateTime<Utc>,
        updated_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            merchant_id,
            amount,
            description,
            reference,
            status,
            payment_method,
            provider,
            provider_reference,
            failure_reason,
            retry_count,
            idempotency_key,
            paid_at,
            updated_at,
        }
    }
    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn merchant_id(&self) -> Uuid {
        self.merchant_id
    }
    pub fn amount(&self) -> Money {
        self.amount.clone()
    }
    pub fn description(&self) -> Option<String> {
        self.description.clone()
    }
    pub fn reference(&self) -> String {
        self.reference.clone()
    }
    pub fn status(&self) -> PaymentStatus {
        self.status.clone()
    }
    pub fn payment_method(&self) -> PaymentMethod {
        self.payment_method.clone()
    }
    pub fn provider(&self) -> PaymentProvider {
        self.provider.clone()
    }
    pub fn provider_reference(&self) -> Option<String> {
        self.provider_reference.clone()
    }
    pub fn failure_reason(&self) -> Option<String> {
        self.failure_reason.clone()
    }
    pub fn retry_count(&self) -> i16 {
        self.retry_count
    }
    pub fn idempotency_key(&self) -> Option<Uuid> {
        self.idempotency_key
    }
    pub fn paid_at(&self) -> DateTime<Utc> {
        self.paid_at
    }
    pub fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
    }
}
