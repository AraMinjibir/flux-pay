use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{
    errors::domain_error::DomainError, payment::{method::PaymentMethod, provider::PaymentProvider, status::PaymentStatus}, shared::{currency::Currency, money::Money},
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
    email: String,
    amount: i64,
    currency: Currency,
    reference: String,
    callback_url: String,
    provider_reference: Option<String>,
}
impl PaymentInitializationRequest {
    pub fn email(&self) -> String {
        self.email.clone()
    }
    pub fn amount(&self) -> i64 {
        self.amount
    }
    pub fn currency(&self) -> Currency {
        self.currency.clone()
    }
    pub fn reference(&self) -> String {
        self.reference.clone()
    }
    pub fn callback_url(&self) -> String {
        self.callback_url.clone()
    }
    pub fn provider_reference(&self) -> Option<String> {
        self.provider_reference.clone()
    }
}
#[derive(Debug, Clone)]
pub struct PaymentInitializationResult {
    pub provider_reference: String,
    pub authorization_url: Option<String>,
    pub client_secret: Option<String>,
    pub status: PaymentStatus,
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
    pub fn set__selected_provider(&mut self, provider: Option<PaymentProvider>) {
        self.provider = provider
    }
    pub fn set_retry_count(&mut self, retry_count: i16) {
        self.retry_count = retry_count
    }
}
