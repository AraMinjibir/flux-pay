use std::str::FromStr;

use crate::domain::errors::ledger_error::LedgerError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnerType {
    Platform,
    Merchant,
    Provider,
}

impl OwnerType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Platform => "PLATFORM",
            Self::Merchant => "MERCHANT",
            Self::Provider => "PROVIDER",
        }
    }
}

impl FromStr for OwnerType {
    type Err = LedgerError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "PLATFORM" => Ok(Self::Platform),
            "MERCHANT" => Ok(Self::Merchant),
            "PROVIDER" => Ok(Self::Provider),
            other => Err(LedgerError::UnknownVariant {
                kind: "owner type",
                value: other.to_owned(),
            }),
        }
    }
}
