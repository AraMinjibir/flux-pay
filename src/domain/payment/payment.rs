use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    application::payment_orchestrator::OrchestrationResult,
    domain::{
        errors::domain_error::DomainError,
        idempotency::models::StoredResponse,
        payment::{method::PaymentMethod, provider::PaymentProvider, status::PaymentStatus},
        shared::{currency::Currency, money::Money},
    },
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
impl PaymentInitializationResult {
    pub fn from_stored_response(response: StoredResponse) -> Self {
        Self {
            id: Some(response.payment_id),
            merchant_id: None,
            amount: None,
            description: None,
            reference: None,
            selected_provider: None,
            provider_reference: response.provider_reference,
            authorization_url: response.authorization_url,
            client_secret: response.client_secret,
            status: response.status,
            created_at: None,
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

    fn generate_reference_number() -> String {
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

    pub fn apply_initialization(
        &mut self,
        execution: &OrchestrationResult,
    ) -> Result<(), DomainError> {
        self.provider = Some(execution.metadata.selected_provider.clone());

        self.provider_reference = Some(execution.initialization.provider_reference.clone());

        self.retry_count = execution.metadata.retry_count;

        Ok(())
    }

    pub fn apply_failure(&mut self, error: &DomainError) -> Result<(), DomainError> {
        self.mark_failed()?;

        self.failure_reason = Some(error.to_string());

        if let DomainError::PaymentProviderFailed { metadata, .. } = error {
            self.retry_count = metadata.retry_count;
        }

        Ok(())
    }

    pub fn to_initialization_result(
        &self,
        initialization: &PaymentInitializationResult,
    ) -> PaymentInitializationResult {
        PaymentInitializationResult {
            id: Some(self.id()),
            merchant_id: Some(self.merchant_id()),
            amount: Some(self.amount()),
            description: self.description(),
            reference: Some(self.reference()),
            status: self.status(),
            created_at: Some(self.created_at()),
            selected_provider: self.provider(),
            provider_reference: initialization.provider_reference.clone(),
            authorization_url: initialization.authorization_url.clone(),
            client_secret: initialization.client_secret.clone(),
        }
    }
}
