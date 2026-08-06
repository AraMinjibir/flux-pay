use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{
    payment::{
        payment::{Payment, PaymentInitializationResult},
        provider::PaymentProvider,
        status::PaymentStatus,
    },
    shared::money::Money,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentCreationResponse {
    pub id: Option<Uuid>,
    pub merchant_id: Option<Uuid>,
    pub amount: Option<Money>,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub status: PaymentStatus,
    pub selected_provider: Option<PaymentProvider>,
    pub provider_reference: String,
    pub authorization_url: Option<String>,
    pub client_secret: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentResponse {
    id: Uuid,
    merchant_id: Uuid,
    amount: Money,
    description: Option<String>,
    reference: String,
    status: PaymentStatus,
    selected_provider: Option<PaymentProvider>,
    provider_reference: Option<String>,
    created_at: DateTime<Utc>,
}

impl PaymentCreationResponse {
    pub fn from_domain(result: PaymentInitializationResult) -> Self {
        Self {
            id: result.id,
            merchant_id: result.merchant_id,
            amount: result.amount,
            description: result.description,
            reference: result.reference,
            status: result.status,
            selected_provider: result.selected_provider,
            created_at: result.created_at,
            provider_reference: result.provider_reference,
            authorization_url: result.authorization_url,
            client_secret: result.client_secret,
        }
    }
}

impl PaymentResponse {
    pub fn from_domain(payment: &Payment) -> Self {
        Self {
            id: payment.id(),
            merchant_id: payment.merchant_id(),
            amount: payment.amount(),
            description: payment.description(),
            reference: payment.reference(),
            status: payment.status(),
            selected_provider: payment.provider(),
            provider_reference: payment.provider_reference(),
            created_at: payment.created_at(),
        }
    }
}
