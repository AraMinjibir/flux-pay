use serde::{Deserialize, Serialize};

use crate::domain::payment::payment::PaymentInitializationRequest;

#[derive(Serialize, Deserialize)]
pub struct InterswitchInitializeRequest {
    pub amount: i64,
    pub currency: String,
    pub reference: String,
    pub redirect_url: Option<String>,
}

impl From<&PaymentInitializationRequest> for InterswitchInitializeRequest {
    fn from(payment: &PaymentInitializationRequest) -> Self {
        Self {
            amount: payment.amount,
            currency: payment.currency.to_string(),
            reference: payment.reference.to_string(),
            redirect_url: payment.callback_url.clone(),
        }
    }
}
