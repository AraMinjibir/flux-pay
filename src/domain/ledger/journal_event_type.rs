use std::str::FromStr;

use crate::domain::ledger::error::LedgerError;

/// The precise accounting event a journal records. Refund, reversal and payout
/// events join this enum as those features reach the ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalEventType {
    PaymentCaptured,
    ProviderSettled,
}

impl JournalEventType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PaymentCaptured => "PAYMENT_CAPTURED",
            Self::ProviderSettled => "PROVIDER_SETTLED",
        }
    }
}

impl FromStr for JournalEventType {
    type Err = LedgerError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "PAYMENT_CAPTURED" => Ok(Self::PaymentCaptured),
            "PROVIDER_SETTLED" => Ok(Self::ProviderSettled),
            other => Err(LedgerError::UnknownVariant {
                kind: "journal event type",
                value: other.to_owned(),
            }),
        }
    }
}
