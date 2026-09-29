use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

use crate::domain::errors::domain_error::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, PartialEq)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum RefundStatus {
    Pending,
    Processing,
    Refunded,
    Failed,
}

impl RefundStatus {
    pub fn transition(current: &mut RefundStatus, next: RefundStatus) -> Result<(), DomainError> {
        Self::validate_transition(current, &next)?;
        *current = next;
        Ok(())
    }

    fn validate_transition(current: &RefundStatus, next: &RefundStatus) -> Result<(), DomainError> {
        if current.can_transition_to(next) {
            Ok(())
        } else {
            Err(DomainError::InvalidRefundStatusTransition {
                from: current.clone(),
                to: next.clone(),
            })
        }
    }

    fn can_transition_to(&self, next: &RefundStatus) -> bool {
        use RefundStatus::*;

        matches!(
            (self, next),
            (Pending, Processing)
                | (Pending, Failed)
                | (Processing, Refunded)
                | (Processing, Failed)
        )
    }
}
