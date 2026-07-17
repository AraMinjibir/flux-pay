use redis::RedisError;
use std::fmt;

use crate::domain::payment::{provider::PaymentProvider, status::PaymentStatus};

#[derive(Debug)]
pub enum DomainError {
    InvalidAmount,
    ValidationError(Vec<String>),

    ReqwestError(String),
    RedisError(String),

    ProviderUnavailable,
    ProviderNotFound(PaymentProvider),
    NoProviderAvailable,

    UnsupportedCurrency,

    InvalidPaymentStatusTransition {
        from: PaymentStatus,
        to: PaymentStatus,
    },

    DatabaseError(String),
}

impl DomainError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, DomainError::ReqwestError(_))
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::InvalidAmount => {
                write!(f, "Amount must not be negative")
            }
            DomainError::ValidationError(causes) => {
                write!(f, "Validation failed: {}", causes.join(", "))
            }
            DomainError::ReqwestError(msg) => {
                write!(f, "Request error: {}", msg)
            }
            DomainError::RedisError(msg) => {
                write!(f, "Redis error: {}", msg)
            }
            DomainError::ProviderUnavailable => {
                write!(f, "Provider is not available")
            }

            DomainError::ProviderNotFound(causes) => {
                write!(f, "Provider not found: {}", causes)
            }
            DomainError::NoProviderAvailable => {
                write!(f, "There is no available provider")
            }

            DomainError::UnsupportedCurrency => {
                write!(f, "The currency provided is Unsupported")
            }
            DomainError::InvalidPaymentStatusTransition { from, to } => {
                write!(
                    f,
                    "Invalid payment status transition from {} to {}",
                    from, to
                )
            }
            DomainError::DatabaseError(msg) => {
                write!(f, "Database error: {}", msg)
            }
        }
    }
}

impl From<reqwest::Error> for DomainError {
    fn from(error: reqwest::Error) -> Self {
        DomainError::ReqwestError(error.to_string())
    }
}
impl From<redis::RedisError> for DomainError {
    fn from(error: redis::RedisError) -> Self {
        DomainError::RedisError(error.to_string())
    }
}
impl From<serde_json::Error> for DomainError {
    fn from(err: serde_json::Error) -> Self {
        DomainError::DatabaseError(err.to_string())
    }
}
