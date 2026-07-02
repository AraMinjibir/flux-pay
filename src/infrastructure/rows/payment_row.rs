use std::str::FromStr;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{
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
    pub payment_provider: String,
    pub provider_reference: Option<String>,
    pub failure_reason: Option<String>,
    pub retry_count: i16,
    pub idempotency_key: Option<Uuid>,
    pub paid_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
impl PaymentRow {
    pub fn into_domain(self) -> Payment {
        let currency = Currency::from_str(&self.currency).expect("invalid currency");

        let money = Money::new(self.amount, currency);

        let status = PaymentStatus::from_str(&self.status)
            .expect("PaymentRow contains an invalid payment status");

        let payment_method = PaymentMethod::from_str(&self.payment_method)
            .expect("PaymentRow contains an invalid payment method");

        let provider = PaymentProvider::from_str(&self.payment_provider)
            .expect("PaymentRow contains an invalid payment provider");

        Payment::new(
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
            self.paid_at,
            self.updated_at,
        )
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
            payment_provider: domain.provider().to_string(),
            provider_reference: domain.provider_reference(),
            failure_reason: domain.failure_reason(),
            retry_count: domain.retry_count(),
            idempotency_key: domain.idempotency_key(),
            paid_at: domain.paid_at(),
            updated_at: domain.updated_at(),
        }
    }
}
