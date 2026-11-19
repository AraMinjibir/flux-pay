use std::str::FromStr;

use crate::domain::{errors::ledger_error::LedgerError, ledger::entry_direction::EntryDirection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

impl AccountType {
    pub fn normal_balance(self) -> EntryDirection {
        match self {
            Self::Asset | Self::Expense => EntryDirection::Debit,
            Self::Liability | Self::Equity | Self::Revenue => EntryDirection::Credit,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Asset => "ASSET",
            Self::Liability => "LIABILITY",
            Self::Equity => "EQUITY",
            Self::Revenue => "REVENUE",
            Self::Expense => "EXPENSE",
        }
    }
}

impl FromStr for AccountType {
    type Err = LedgerError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "ASSET" => Ok(Self::Asset),
            "LIABILITY" => Ok(Self::Liability),
            "EQUITY" => Ok(Self::Equity),
            "REVENUE" => Ok(Self::Revenue),
            "EXPENSE" => Ok(Self::Expense),
            other => Err(LedgerError::UnknownVariant {
                kind: "account type",
                value: other.to_owned(),
            }),
        }
    }
}