use serde::{Deserialize, Serialize};

use crate::domain::payment::payment::PaymentInitializationRequest;

#[derive(Deserialize, Serialize)]

pub struct ZainpayInitializationRequest {
    pub email: Option<String>,
    pub amount: i64,
    pub tnx_ref: String,
    pub mobile_number: Option<String>,
    pub currency: String,
    pub reference: String,
    pub callback_url: Option<String>,
    pub allow_recurring_payment: bool,
}

impl From<&PaymentInitializationRequest> for ZainpayInitializationRequest {
    fn from(payment: &PaymentInitializationRequest) -> Self {
        Self {
            email: payment.email.clone(),
            amount: payment.amount,
            currency: payment.currency.to_string(),
            reference: payment.reference.to_string(),
            callback_url: payment.callback_url.clone(),
            allow_recurring_payment: true,
            mobile_number: None,
            tnx_ref: payment.reference.clone(),
        }
    }
}
