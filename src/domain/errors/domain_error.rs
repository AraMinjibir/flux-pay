use std::fmt;

use crate::{
    application::payment_orchestrator::OrchestrationFailureMetadata,
    domain::{
        errors::repository_error::RepositoryError,
        payment::{provider::PaymentProvider, status::PaymentStatus},
    },
};

#[derive(Debug)]
pub enum DomainError {
    InvalidAmount,
    ValidationError(Vec<String>),

    ReqwestError(String),
    RedisError(String),

    ProviderUnavailable,
    ProviderNotFound(PaymentProvider),
    NoProviderAvailable,
    PaymentProviderFailed {
        error: Box<DomainError>,
        metadata: OrchestrationFailureMetadata,
    },

    UnsupportedCurrency,
    RequestInProgress,
    InvalidStoredResponse,

    InvalidPaymentStatusTransition {
        from: PaymentStatus,
        to: PaymentStatus,
    },

    DatabaseError(String),
    DuplicateEntity,
    ForeignKeyViolation,
    EntityNotFound,
    ConnectionError,
    PoolTimeout,
    NullConstraintViolation,
    CheckConstraintViolation,
    DataTooLong,
    InvalidDataFormat,
    NumericOverflow,
    DeadlockDetected,
    TransactionTimeout,
    SerializationFailure,
    Internal(String),
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
            DomainError::RequestInProgress => {
                write!(
                    f,
                    "This request is already received and currently Processing"
                )
            }
            DomainError::InvalidStoredResponse => {
                write!(f, "Invalid stored response")
            }
            DomainError::DatabaseError(msg) => {
                write!(f, "Database error: {}", msg)
            }

            DomainError::ForeignKeyViolation => {
                write!(f, "Foreign key violation")
            }

            DomainError::NullConstraintViolation => {
                write!(f, "Null constraint violation")
            }

            DomainError::CheckConstraintViolation => {
                write!(f, "Check constraint violation")
            }

            DomainError::DataTooLong => {
                write!(f, "Data too long")
            }

            DomainError::InvalidDataFormat => {
                write!(f, "Invalid data format")
            }

            DomainError::NumericOverflow => {
                write!(f, "Numeric overflow")
            }

            DomainError::DeadlockDetected => {
                write!(f, "Deadlock detected")
            }

            DomainError::TransactionTimeout => {
                write!(f, "Transaction timeout")
            }

            DomainError::SerializationFailure => {
                write!(f, "Serialization failure")
            }

            DomainError::DuplicateEntity => {
                write!(f, "Duplicate Entity")
            }
            DomainError::EntityNotFound => {
                write!(f, "Entity Not Found")
            }
            DomainError::ConnectionError => {
                write!(f, "Connection Error")
            }
            DomainError::PoolTimeout => {
                write!(f, "Pool Timeout")
            }
            DomainError::Internal(msg) => {
                write!(f, "Internal system error: {}", msg)
            }

            DomainError::PaymentProviderFailed { error, .. } => {
                write!(f, "Payment provider failed: {error}")
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
impl From<RepositoryError> for DomainError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::DuplicateEntity => DomainError::DuplicateEntity,
            RepositoryError::ForeignKeyViolation => DomainError::ForeignKeyViolation,
            RepositoryError::NullConstraintViolation => DomainError::NullConstraintViolation,
            RepositoryError::CheckConstraintViolation => DomainError::CheckConstraintViolation,
            RepositoryError::DataTooLong => DomainError::DataTooLong,
            RepositoryError::EntityNotFound => DomainError::EntityNotFound,
            RepositoryError::PoolTimeout => DomainError::PoolTimeout,
            RepositoryError::InvalidDataFormat => DomainError::InvalidDataFormat,
            RepositoryError::ConnectionError => DomainError::ConnectionError,
            RepositoryError::NumericOverflow => DomainError::NumericOverflow,
            RepositoryError::DeadlockDetected => DomainError::DeadlockDetected,
            RepositoryError::TransactionTimeout => DomainError::TransactionTimeout,
            RepositoryError::SerializationFailure => DomainError::SerializationFailure,
            RepositoryError::DatabaseError(msg) => DomainError::DatabaseError(msg),
        }
    }
}
