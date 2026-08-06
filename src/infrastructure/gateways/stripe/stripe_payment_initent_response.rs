use serde::Deserialize;

use crate::domain::payment::{payment::PaymentInitializationResult, status::PaymentStatus};

#[derive(Deserialize)]
pub struct StripePaymentIntentResponse {
    id: String,
    client_secret: String,
    status: StripeStatus,
    authorization_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StripeStatus {
    RequiresPaymentMethod,
    RequiresConfirmation,
    RequiresAction,
    Processing,
    Succeeded,
    Canceled,
}

impl From<StripeStatus> for PaymentStatus {
    fn from(status: StripeStatus) -> Self {
        match status {
            StripeStatus::RequiresPaymentMethod => PaymentStatus::Processing,
            StripeStatus::RequiresConfirmation => PaymentStatus::Processing,
            StripeStatus::RequiresAction => PaymentStatus::Processing,
            StripeStatus::Processing => PaymentStatus::Processing,
            StripeStatus::Succeeded => PaymentStatus::Success,
            StripeStatus::Canceled => PaymentStatus::Failed,
        }
    }
}

impl From<StripePaymentIntentResponse> for PaymentInitializationResult {
    fn from(response: StripePaymentIntentResponse) -> Self {
        Self {
            provider_reference: response.id,
            client_secret: Some(response.client_secret),
            status: response.status.into(),
            authorization_url: response.authorization_url,
            selected_provider: None,
            amount: None,
            created_at: None,
            id: None,
            merchant_id: None,
            reference: None,
            description: None,
        }
    }
}
