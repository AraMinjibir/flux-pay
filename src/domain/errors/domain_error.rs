use std::fmt;

#[derive(Debug)]
pub enum DomainError {
    InvalidAmount,
    ValidationError(Vec<String>),
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
        }
    }
}
