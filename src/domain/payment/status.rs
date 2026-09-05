use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

use crate::domain::errors::domain_error::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, PartialEq)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentStatus {
    Created,
    Processing,
    Success,
    Failed,
    Deleted,
}

impl PaymentStatus {
    pub fn transition(current: &mut PaymentStatus, next: PaymentStatus) -> Result<(), DomainError> {
        Self::validate_transition(current, &next)?;
        *current = next;
        Ok(())
    }

    pub fn validate_transition(
        current: &PaymentStatus,
        next: &PaymentStatus,
    ) -> Result<(), DomainError> {
        if current.can_transition_to(next) {
            Ok(())
        } else {
            Err(DomainError::InvalidPaymentStatusTransition {
                from: current.clone(),
                to: next.clone(),
            })
        }
    }

    pub fn can_transition_to(&self, next: &PaymentStatus) -> bool {
        use PaymentStatus::*;

        matches!(
            (self, next),
            (Created, Processing) | (Processing, Success) | (Processing, Failed)
        )
    }
}
