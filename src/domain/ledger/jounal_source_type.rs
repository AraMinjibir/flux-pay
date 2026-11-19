use std::str::FromStr;

use crate::domain::ledger::error::LedgerError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalSourceType {
    Payment,
    Refund,
    Payout,
    Settlement,
}

impl JournalSourceType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Payment => "PAYMENT",
            Self::Refund => "REFUND",
            Self::Payout => "PAYOUT",
            Self::Settlement => "SETTLEMENT",
        }
    }
}

impl FromStr for JournalSourceType {
    type Err = LedgerError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "PAYMENT" => Ok(Self::Payment),
            "REFUND" => Ok(Self::Refund),
            "PAYOUT" => Ok(Self::Payout),
            "SETTLEMENT" => Ok(Self::Settlement),
            other => Err(LedgerError::UnknownVariant {
                kind: "journal source type",
                value: other.to_owned(),
            }),
        }
    }
}