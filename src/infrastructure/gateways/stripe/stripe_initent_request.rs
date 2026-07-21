use serde::Serialize;

use crate::domain::payment::payment::PaymentInitializationRequest;

#[derive(Serialize)]
pub struct StripePaymentIntentRequest {
    pub amount: i64,
    pub currency: String,
}

impl From<&PaymentInitializationRequest> for StripePaymentIntentRequest {
    fn from(request: &PaymentInitializationRequest) -> Self {
        Self {
            amount: request.amount,
            currency: request.currency.to_string().to_lowercase(),
        }
    }
}
