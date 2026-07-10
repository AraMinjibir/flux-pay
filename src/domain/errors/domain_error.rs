use std::fmt;

#[derive(Debug)]
pub enum DomainError {
    InvalidAmount,
    ValidationError(Vec<String>),
    ReqwestError(String)
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
        }
    }
}

impl From<reqwest::Error> for DomainError {
    fn from(error:reqwest::Error) -> Self {
        DomainError::ReqwestError(error.to_string())
    }
}