use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{
    errors::ledger_error::LedgerError, ledger::{
        amount::{currency_of, minor},
        entry_direction::EntryDirection,
        jounal_source_type::JournalSourceType,
        journal_entry::NewJournalEntry,
        journal_event_type::JournalEventType,
        journal_status::JournalStatus,
    }, shared::currency::Currency,
};

/// A persisted journal header.
#[derive(Debug, Clone)]
pub struct Journal {
    pub id: Uuid,
    pub reference: String,
    pub source_type: JournalSourceType,
    pub source_id: Uuid,
    pub event_type: JournalEventType,
    pub idempotency_key: String,
    pub description: String,
    pub currency: Currency,
    pub status: JournalStatus,
    pub created_at: DateTime<Utc>,
    pub posted_at: Option<DateTime<Utc>>,
}

/// A journal proposed for posting. `validate` is the single domain-level gate;
/// the database triggers re-check the same invariants as a backstop.
#[derive(Debug, Clone)]
pub struct JournalDraft {
    pub reference: String,
    pub source_type: JournalSourceType,
    pub source_id: Uuid,
    pub event_type: JournalEventType,
    pub idempotency_key: String,
    pub description: String,
    pub currency: Currency,
    pub entries: Vec<NewJournalEntry>,
}

impl JournalDraft {
    /// Checks: at least two entries, positive amounts, one currency, debits == credits.
    pub fn validate(&self) -> Result<(), LedgerError> {
        if self.entries.len() < 2 {
            return Err(LedgerError::JournalTooShort);
        }

        let mut debits: i64 = 0;

        let mut credits: i64 = 0;

        for (index, entry) in self.entries.iter().enumerate() {
            if currency_of(&entry.amount) != self.currency {
                return Err(LedgerError::CurrencyMismatch { line: index + 1 });
            }

            let side = match entry.direction {
                EntryDirection::Debit => &mut debits,

                EntryDirection::Credit => &mut credits,
            };

            *side = side
                .checked_add(minor(&entry.amount))
                .ok_or(LedgerError::AmountOverflow)?;
        }

        if debits != credits {
            return Err(LedgerError::Unbalanced { debits, credits });
        }

        Ok(())
    }
}
