use std::fmt::{Display, Formatter};

use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde::Serialize;

use crate::domain::errors::domain_error::DomainError;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

#[derive(Debug)]
pub enum ApiError {
    Validation(String),
    NotFound(String),
    Conflict(String),
    Internal(String),
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let response = match self {
            Self::Validation(message) => ErrorResponse {
                code: "VALIDATION_ERROR".to_string(),
                message: message.clone(),
            },

            Self::NotFound(message) => ErrorResponse {
                code: "NOT_FOUND".to_string(),
                message: message.clone(),
            },

            Self::Conflict(message) => ErrorResponse {
                code: "CONFLICT".to_string(),
                message: message.clone(),
            },

            Self::Internal(message) => ErrorResponse {
                code: "INTERNAL_ERROR".to_string(),
                message: message.clone(),
            },
        };

        HttpResponse::build(self.status_code()).json(response)
    }
}
impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Validation(message) => write!(f, "{message}"),
            ApiError::NotFound(message) => write!(f, "{message}"),
            ApiError::Conflict(message) => write!(f, "{message}"),
            ApiError::Internal(message) => write!(f, "{message}"),
        }
    }
}
impl From<DomainError> for ApiError {
    fn from(error: DomainError) -> Self {
        match error {
            // Validation errors
            DomainError::InvalidAmount => Self::Validation("Invalid amount".into()),

            DomainError::ValidationError(errors) => Self::Validation(errors.join(", ")),

            DomainError::UnsupportedCurrency => Self::Validation("Unsupported currency".into()),

            DomainError::InvalidStoredResponse => {
                Self::Validation("Invalid stored response".into())
            }

            DomainError::InvalidPaymentStatusTransition { from, to } => Self::Validation(format!(
                "Invalid state transition from {:?} to {:?}",
                from, to
            )),

            // Not found
            DomainError::PaymentNotFound(id) => {
                Self::NotFound(format!("Payment {id} was not found"))
            }

            DomainError::PaymentWithRefNotFound(reference) => {
                Self::NotFound(format!("Payment reference {reference} was not found"))
            }

            DomainError::EntityNotFound => Self::NotFound("Entity not found".into()),

            // Conflict
            DomainError::RequestInProgress => Self::Conflict("Request already in progress".into()),

            DomainError::DuplicateEntity => Self::Conflict("Duplicate entity".into()),

            // Internal errors
            DomainError::ReqwestError(error)
            | DomainError::RedisError(error)
            | DomainError::DatabaseError(error)
            | DomainError::ConfigurationError(error)
            | DomainError::SerializationError(error)
            | DomainError::Internal(error) => Self::Internal(error),

            DomainError::ProviderUnavailable => Self::Internal("Provider unavailable".into()),

            DomainError::ProviderNotFound(provider) => {
                Self::Internal(format!("Provider {:?} was not found", provider))
            }

            DomainError::NoProviderAvailable => Self::Internal("No provider available".into()),

            DomainError::PaymentProviderFailed { error, .. } => Self::Internal(error.to_string()),

            DomainError::ForeignKeyViolation
            | DomainError::ConnectionError
            | DomainError::PoolTimeout
            | DomainError::NullConstraintViolation
            | DomainError::CheckConstraintViolation
            | DomainError::DataTooLong
            | DomainError::InvalidDataFormat
            | DomainError::NumericOverflow
            | DomainError::DeadlockDetected
            | DomainError::TransactionTimeout
            | DomainError::SerializationFailure => Self::Internal(error.to_string()),
        }
    }
}
