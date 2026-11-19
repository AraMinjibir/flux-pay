use std::str::FromStr;

use crate::domain::ledger::error::LedgerError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalStatus {
    Draft,
    Posted,
}

impl JournalStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::Posted => "POSTED",
        }
    }
}

impl FromStr for JournalStatus {
    type Err = LedgerError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "DRAFT" => Ok(Self::Draft),
            "POSTED" => Ok(Self::Posted),
            other => Err(LedgerError::UnknownVariant {
                kind: "journal status",
                value: other.to_owned(),
            }),
        }
    }
}