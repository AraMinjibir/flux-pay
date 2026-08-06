use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::{
    errors::domain_error::DomainError,
    payment::{method::PaymentMethod, provider::PaymentProvider, status::PaymentStatus},
    shared::{currency::Currency, money::Money},
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
    provider: Option<PaymentProvider>,
    provider_reference: Option<String>,
    failure_reason: Option<String>,
    retry_count: i16,
    idempotency_key: Option<Uuid>,
    created_at: DateTime<Utc>,
    paid_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct PaymentInitializationRequest {
    pub email: Option<String>,
    pub amount: i64,
    pub currency: Currency,
    pub reference: String,
    pub callback_url: Option<String>,
    pub provider_reference: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaymentInitializationResult {
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
#[derive(Debug, Clone)]
pub struct CreatePaymentCommand {
    pub email: Option<String>,
    pub merchant_id: Uuid,
    pub amount: Money,
    pub description: Option<String>,
    pub payment_method: PaymentMethod,
    pub idempotency_key: Uuid,
}

impl PaymentInitializationRequest {
    pub fn converted_request(command: &CreatePaymentCommand, payment: &Payment) -> Self {
        Self {
            email: command.email.clone(),
            amount: payment.amount().amount(),
            currency: payment.amount().currency(),
            reference: payment.reference(),
            callback_url: None,
            provider_reference: payment.provider_reference(),
        }
    }
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
        provider: Option<PaymentProvider>,
        provider_reference: Option<String>,
        failure_reason: Option<String>,
        retry_count: i16,
        idempotency_key: Option<Uuid>,
        created_at: DateTime<Utc>,
        paid_at: Option<DateTime<Utc>>,
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
            created_at,
            paid_at,
        }
    }
    pub fn generate_payment(
        merchant_id: Uuid,
        amount: Money,
        description: Option<String>,
        payment_method: PaymentMethod,
    ) -> Result<Self, DomainError> {
        let mut errors = Vec::new();

        if merchant_id.is_nil() {
            errors.push("Merchant id must not be empty".to_string());
        }
        if !errors.is_empty() {
            return Err(DomainError::ValidationError(errors));
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            merchant_id,
            amount,
            description,
            status: PaymentStatus::Created,
            reference: Self::generate_reference_number(),
            payment_method,
            provider: None,
            provider_reference: None,
            failure_reason: None,
            retry_count: 0,
            idempotency_key: None,
            created_at: now,
            paid_at: None,
        })
    }

    pub fn generate_reference_number() -> String {
        let id = Uuid::new_v4().to_string().replace("-", "");
        format!("RF-FluxPay-{}", &id[..10].to_uppercase())
    }

    pub fn mark_processing(&mut self) -> Result<(), DomainError> {
        PaymentStatus::transition(&mut self.status, PaymentStatus::Processing)
    }
    pub fn mark_success(&mut self) -> Result<(), DomainError> {
        PaymentStatus::transition(&mut self.status, PaymentStatus::Success)
    }
    pub fn mark_failed(&mut self) -> Result<(), DomainError> {
        PaymentStatus::transition(&mut self.status, PaymentStatus::Failed)
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
    pub fn provider(&self) -> Option<PaymentProvider> {
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
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn paid_at(&self) -> Option<DateTime<Utc>> {
        self.paid_at
    }

    pub fn set_status(&mut self, status: PaymentStatus) {
        self.status = status
    }
    pub fn set_provider_reference(&mut self, provider_reference: Option<String>) {
        self.provider_reference = provider_reference
    }
    pub fn set_failure_reason(&mut self, failure_reason: Option<String>) {
        self.failure_reason = failure_reason
    }

    pub fn set_paid_at(&mut self, paid_at: Option<DateTime<Utc>>) {
        self.paid_at = paid_at
    }
    pub fn set_selected_provider(&mut self, provider: Option<PaymentProvider>) {
        self.provider = provider
    }
    pub fn set_retry_count(&mut self, retry_count: i16) {
        self.retry_count = retry_count
    }
}
