use std::str::FromStr;

use crate::domain::ledger::error::LedgerError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryDirection {
    Debit,
    Credit,
}

impl EntryDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Debit => "DEBIT",
            Self::Credit => "CREDIT",
        }
    }
}

impl FromStr for EntryDirection {
    type Err = LedgerError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "DEBIT" => Ok(Self::Debit),
            "CREDIT" => Ok(Self::Credit),
            other => Err(LedgerError::UnknownVariant {
                kind: "entry direction",
                value: other.to_owned(),
            }),
        }
    }
}