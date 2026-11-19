use std::fmt;

/// Errors raised by ledger domain rules, before anything touches the database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerError {
    /// A journal needs at least two entries.
    JournalTooShort,
    /// An entry amount was zero or negative (direction carries the sign).
    InvalidAmount,
    /// An entry's currency differs from the journal currency.
    CurrencyMismatch { line: usize },
    /// Total debits do not equal total credits.
    Unbalanced { debits: i64, credits: i64 },
    /// Summing amounts overflowed i64.
    AmountOverflow,
    /// The platform fee is larger than the gross amount.
    FeeExceedsGross,
    /// A posting rule received inconsistent input.
    InvalidPostingInput(&'static str),
    /// A stored string did not map to a known enum value.
    UnknownVariant { kind: &'static str, value: String },
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JournalTooShort => write!(f, "journal needs at least two entries"),
            Self::InvalidAmount => write!(f, "amount must be positive"),
            Self::CurrencyMismatch { line } => {
                write!(f, "entry {line} currency differs from the journal currency")
            }
            Self::Unbalanced { debits, credits } => {
                write!(
                    f,
                    "journal is unbalanced (debits {debits}, credits {credits})"
                )
            }
            Self::AmountOverflow => write!(f, "journal amount overflowed"),
            Self::FeeExceedsGross => write!(f, "fee exceeds the gross amount"),
            Self::InvalidPostingInput(reason) => write!(f, "invalid posting input: {reason}"),
            Self::UnknownVariant { kind, value } => write!(f, "unknown {kind}: {value}"),
        }
    }
}

impl std::error::Error for LedgerError {}
