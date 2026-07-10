use serde::Serialize;

use crate::domain::payment::payment::PaymentInitializationRequest;

#[derive(Serialize)]
pub struct PaystackInitializeRequest {
    pub email: String,
    pub amount: i64,
    pub currency: String,
    pub reference: String,
    pub callback_url: String,
}

impl From<&PaymentInitializationRequest> for PaystackInitializeRequest {
    fn from(payment: &PaymentInitializationRequest) -> Self {
        Self {
            email: payment.email().to_string(),
            amount: payment.amount(),
            currency: payment.currency().to_string(),
            reference: payment.reference().to_string(),
            callback_url: payment.callback_url().to_string(),
        }
    }
}
